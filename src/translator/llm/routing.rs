//! LLM Provider weighted routing
//!
//! This module provides intelligent routing between LLM providers based on their capacity.
//! Each provider represents a single model/endpoint with a fixed max_tokens capacity.
//!
//! Routing logic:
//! 1. **Short texts** (below threshold): Weighted distribution among all providers
//! 2. **Long texts** (above threshold): Weighted distribution among capable providers
//!
//! The threshold is set to the minimum capacity among all providers,
//! ensuring short texts can be handled by any provider.

use std::sync::Arc;

use tracing::{debug, info, trace, warn};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::llm::provider::LLMProvider;

/// Weighted router for LLM providers
///
/// Routes texts to providers based on:
/// 1. Capacity (must fit within provider's max_tokens limit)
/// 2. Weight (higher weight = more likely to be selected)
#[derive(Debug)]
pub struct ProviderRouter {
    providers: Vec<Arc<LLMProvider>>,
    /// Threshold for long text routing (minimum capacity among all providers)
    capacity_threshold: usize,
}

impl ProviderRouter {
    /// Create a new provider router from configurations
    pub fn new(configs: &[LLMProviderConfig]) -> Result<Self> {
        // Validate input configurations
        if configs.is_empty() {
            return Err(TranslateError::Config(
                "At least one LLM provider configuration is required".to_string(),
            ));
        }

        // Check for duplicate provider IDs
        let mut seen_ids = std::collections::HashSet::new();
        for config in configs {
            if config.id.is_empty() {
                return Err(TranslateError::Config(
                    "Provider ID cannot be empty".to_string(),
                ));
            }
            if !seen_ids.insert(&config.id) {
                return Err(TranslateError::Config(format!(
                    "Duplicate provider ID: {}",
                    config.id
                )));
            }
        }

        let mut providers: Vec<Arc<LLMProvider>> = Vec::new();
        let mut total_weight = 0u32;

        for config in configs {
            // Validate individual configuration
            if config.base_url.is_empty() {
                warn!("Provider {} has empty base_url, skipping", config.id);
                continue;
            }

            if config.api_keys.is_empty() {
                warn!("Provider {} has no API keys, skipping", config.id);
                continue;
            }

            match LLMProvider::new(config) {
                Ok(provider) => {
                    let max_chars = provider.max_input_chars();
                    let weight = provider.weight();
                    debug!(
                        "Added provider {} with capacity {} chars, weight {}",
                        config.id, max_chars, weight
                    );
                    total_weight += weight;
                    providers.push(Arc::new(provider));
                }
                Err(e) => {
                    warn!("Failed to create provider {}: {}. Skipping.", config.id, e);
                }
            }
        }

        if providers.is_empty() {
            return Err(TranslateError::Config(
                "No valid LLM providers configured. Please check your configuration for missing required fields (id, base_url, api_keys)".to_string(),
            ));
        }

        // Warn if all weights are zero
        if total_weight == 0 {
            warn!(
                "All providers have weight 0. Weighted selection will fall back to random selection."
            );
        }

        // Calculate capacity threshold (minimum capacity among all providers)
        let capacity_threshold = providers
            .iter()
            .map(|p| p.max_input_chars())
            .filter(|&c| c > 0)
            .min()
            .unwrap_or(0);

        info!(
            "Created ProviderRouter with {} providers, capacity_threshold: {}, total_weight: {}",
            providers.len(),
            capacity_threshold,
            total_weight
        );

        Ok(Self {
            providers,
            capacity_threshold,
        })
    }

    /// Select provider based on text length using weighted distribution
    pub fn select_provider(&self, text_len: usize) -> Option<&Arc<LLMProvider>> {
        trace!(
            "Selecting provider for text length: {} (threshold: {})",
            text_len,
            self.capacity_threshold
        );

        // For long texts, filter to capable providers only
        let candidates: Vec<&Arc<LLMProvider>> = if text_len < self.capacity_threshold {
            self.providers.iter().collect()
        } else {
            self.providers
                .iter()
                .filter(|p| p.can_handle(text_len))
                .collect()
        };

        if candidates.is_empty() {
            warn!(
                "No provider can handle text of length {}. Maximum capacity: {}",
                text_len,
                self.max_capacity()
            );
            return None;
        }

        self.select_weighted(&candidates)
    }

    /// Select provider by weighted strategy using random selection
    fn select_weighted<'a>(
        &self,
        candidates: &[&'a Arc<LLMProvider>],
    ) -> Option<&'a Arc<LLMProvider>> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        // Calculate total weight of candidates
        let total_weight: u32 = candidates.iter().map(|p| p.weight()).sum();
        if total_weight == 0 {
            // Fall back to random selection if all weights are 0
            let index = rand::random::<usize>() % candidates.len();
            trace!(
                "All weights are 0, randomly selected provider at index {}",
                index
            );
            return Some(candidates[index]);
        }

        // Use random selection for weighted distribution
        let target_weight = rand::random::<u32>() % total_weight;

        let mut current_weight = 0u32;
        for provider in candidates {
            current_weight += provider.weight();
            if target_weight < current_weight {
                trace!(
                    "Weighted selected provider {} with weight {} (target: {})",
                    provider.id(),
                    provider.weight(),
                    target_weight
                );
                return Some(provider);
            }
        }

        // Fallback to last candidate
        let provider = candidates.last().copied();
        if let Some(p) = provider {
            trace!(
                "Weighted selection fell back to last provider {}",
                p.id()
            );
        }
        provider
    }

    /// Get capacity threshold (minimum capacity among all providers)
    pub fn capacity_threshold(&self) -> usize {
        self.capacity_threshold
    }

    /// Get maximum capacity among all providers
    pub fn max_capacity(&self) -> usize {
        self.providers
            .iter()
            .map(|p| p.max_input_chars())
            .max()
            .unwrap_or(0)
    }

    /// Get all providers
    pub fn providers(&self) -> &[Arc<LLMProvider>] {
        &self.providers
    }

    /// Check if any provider can handle the given text length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.providers.iter().any(|p| p.can_handle(text_len))
    }

    /// Route and translate a single text
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let text_len = text.len();

        let provider = self.select_provider(text_len).ok_or_else(|| {
            TranslateError::Translation(format!(
                "No provider can handle text of length {}. Maximum capacity: {}",
                text_len,
                self.max_capacity()
            ))
        })?;

        let response = provider
            .translate(text, source_lang, target_lang)
            .await?;

        Ok(response.translated_text)
    }
}
