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

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tracing::{debug, trace, warn};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::llm::provider::{LLMProvider, Provider, ProviderImpl};

/// Provider with capacity information for routing
#[derive(Debug, Clone)]
pub struct CapacityProvider {
    provider: Arc<ProviderImpl>,
    max_chars: usize,
    weight: u32,
}

impl CapacityProvider {
    /// Create a new capacity-aware provider wrapper
    pub fn new(config: &LLMProviderConfig) -> Result<Self> {
        let provider = Arc::new(ProviderImpl::LLM(LLMProvider::new(config)?));
        let max_chars = provider.translator().max_input_chars();

        Ok(Self {
            provider,
            max_chars,
            weight: config.weight,
        })
    }

    /// Check if this provider can handle the given text length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.max_chars == 0 || text_len <= self.max_chars
    }

    /// Get the underlying provider
    pub fn provider(&self) -> &Arc<ProviderImpl> {
        &self.provider
    }

    /// Get maximum characters capacity
    pub fn max_chars(&self) -> usize {
        self.max_chars
    }

    /// Get weight (higher = more likely to be selected)
    pub fn weight(&self) -> u32 {
        self.weight
    }
}

/// Weighted router for LLM providers
///
/// Routes texts to providers based on:
/// 1. Capacity (must fit within provider's max_tokens limit)
/// 2. Weight (higher weight = more likely to be selected)
pub struct ProviderRouter {
    providers: Vec<CapacityProvider>,
    /// Threshold for long text routing (minimum capacity among all providers)
    capacity_threshold: usize,
    /// Current index for weighted selection
    current_index: AtomicU64,
}

impl ProviderRouter {
    /// Create a new provider router from configurations
    pub fn new(configs: &[LLMProviderConfig]) -> Result<Self> {
        let mut providers = Vec::new();

        for config in configs {
            match CapacityProvider::new(config) {
                Ok(provider) => {
                    debug!(
                        "Added provider {} with capacity {} chars, weight {}",
                        config.id, provider.max_chars, provider.weight
                    );
                    providers.push(provider);
                }
                Err(e) => {
                    warn!("Failed to create provider {}: {}. Skipping.", config.id, e);
                }
            }
        }

        if providers.is_empty() {
            return Err(TranslateError::Config(
                "No valid LLM providers configured".to_string(),
            ));
        }

        // Calculate capacity threshold (minimum capacity among all providers)
        let capacity_threshold = providers
            .iter()
            .map(|p| p.max_chars())
            .filter(|&c| c > 0)
            .min()
            .unwrap_or(0);

        info!(
            "Created ProviderRouter with {} providers, capacity_threshold: {}",
            providers.len(),
            capacity_threshold
        );

        Ok(Self {
            providers,
            capacity_threshold,
            current_index: AtomicU64::new(0),
        })
    }

    /// Select provider based on text length using weighted distribution
    pub fn select_provider(&self, text_len: usize) -> Option<&CapacityProvider> {
        trace!(
            "Selecting provider for text length: {} (threshold: {})",
            text_len,
            self.capacity_threshold
        );

        // For long texts, filter to capable providers only
        let candidates: Vec<&CapacityProvider> = if text_len < self.capacity_threshold {
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

    /// Select provider by weighted strategy
    fn select_weighted<'a>(
        &self,
        candidates: &[&'a CapacityProvider],
    ) -> Option<&'a CapacityProvider> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        // Calculate total weight of candidates
        let total_weight: u32 = candidates.iter().map(|p| p.weight()).sum();
        if total_weight == 0 {
            // Fall back to round-robin if all weights are 0
            let index =
                self.current_index.fetch_add(1, Ordering::Relaxed) as usize % candidates.len();
            return Some(candidates[index]);
        }

        let target_weight =
            self.current_index.fetch_add(1, Ordering::Relaxed) as u32 % total_weight;

        let mut current_weight = 0u32;
        for provider in candidates {
            current_weight += provider.weight();
            if target_weight < current_weight {
                trace!(
                    "Weighted selected provider with weight {} (target: {})",
                    provider.weight(),
                    target_weight
                );
                return Some(provider);
            }
        }

        // Fallback to last candidate
        candidates.last().copied()
    }

    /// Get capacity threshold (minimum capacity among all providers)
    pub fn capacity_threshold(&self) -> usize {
        self.capacity_threshold
    }

    /// Get maximum capacity among all providers
    pub fn max_capacity(&self) -> usize {
        self.providers
            .iter()
            .map(|p| p.max_chars())
            .max()
            .unwrap_or(0)
    }

    /// Get all providers
    pub fn providers(&self) -> &[CapacityProvider] {
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
            .provider()
            .translate(text, source_lang, target_lang)
            .await?;

        Ok(response.translated_text)
    }
}

use tracing::info;

#[cfg(test)]
mod tests {
    // Tests would require mock providers
}
