//! LLM Provider rate-based routing
//!
//! This module provides intelligent routing between LLM providers based on their capacity.
//! Each provider represents a single model/endpoint with a fixed max_tokens capacity.
//!
//! Routing logic:
//! 1. **Short texts** (below threshold): Rate-based distribution among all providers
//! 2. **Long texts** (above threshold): Rate-based distribution among capable providers
//!
//! The threshold is set to the minimum capacity among all providers,
//! ensuring short texts can be handled by any provider.
//!
//! Selection strategies:
//! - **RateBasedRandom**: Pure random selection weighted by rate_limit
//! - **SmoothRateBasedRoundRobin**: Smooth round-robin weighted by rate_limit for better distribution

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tracing::{debug, info, trace, warn};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::llm::provider::LLMProvider;

/// Selection strategy for provider routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum SelectionStrategy {
    /// Pure random selection weighted by rate_limit
    RateBasedRandom,
    /// Smooth round-robin weighted by rate_limit (better distribution over time)
    #[default]
    SmoothRateBasedRoundRobin,
}


/// Provider entry with selection state for smooth rate-based round-robin
#[derive(Debug)]
struct ProviderEntry {
    provider: Arc<LLMProvider>,
    #[allow(dead_code)]
    current_weight: AtomicU32,
    effective_weight: AtomicU32,
}

impl ProviderEntry {
    fn new(provider: Arc<LLMProvider>) -> Self {
        let rate_limit = provider.rate_limit();
        Self {
            provider,
            current_weight: AtomicU32::new(0),
            effective_weight: AtomicU32::new(rate_limit.max(1)),
        }
    }

    #[allow(dead_code)]
    fn rate_limit(&self) -> u32 {
        self.provider.rate_limit()
    }

    fn effective_weight(&self) -> u32 {
        self.effective_weight.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    fn current_weight(&self) -> u32 {
        self.current_weight.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    fn update_effective_weight(&self, is_healthy: bool) {
        let base_weight = self.rate_limit().max(1);
        let new_weight = if is_healthy { base_weight } else { 0 };
        self.effective_weight.store(new_weight, Ordering::Relaxed);
    }
}

/// Rate-based router for LLM providers
///
/// Routes texts to providers based on:
/// 1. Capacity (must fit within provider's max_tokens limit)
/// 2. Rate limit (higher rate_limit = more likely to be selected)
/// 3. Health status (unhealthy providers are excluded)
#[derive(Debug)]
pub struct ProviderRouter {
    providers: Vec<ProviderEntry>,
    #[allow(dead_code)]
    capacity_threshold: usize,
    #[allow(dead_code)]
    strategy: SelectionStrategy,
    #[allow(dead_code)]
    total_effective_weight: AtomicU32,
}

impl ProviderRouter {
    /// Create a new provider router from configurations
    pub fn new(configs: &[LLMProviderConfig]) -> Result<Self> {
        Self::new_with_strategy(configs, SelectionStrategy::default())
    }

    /// Create a new provider router with specific strategy
    pub fn new_with_strategy(
        configs: &[LLMProviderConfig],
        strategy: SelectionStrategy,
    ) -> Result<Self> {
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

        let mut providers: Vec<ProviderEntry> = Vec::new();
        let mut total_rate_limit = 0u32;

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
                    let rate_limit = provider.rate_limit();
                    debug!(
                        "Added provider {} with capacity {} chars, rate_limit {}",
                        config.id, max_chars, rate_limit
                    );
                    total_rate_limit += rate_limit;
                    providers.push(ProviderEntry::new(Arc::new(provider)));
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

        // Warn if all rate limits are zero
        if total_rate_limit == 0 {
            warn!(
                "All providers have rate_limit 0. Rate-based selection will use equal distribution."
            );
        }

        // Calculate capacity threshold (minimum capacity among all providers)
        let capacity_threshold = providers
            .iter()
            .map(|p| p.provider.max_input_chars())
            .filter(|&c| c > 0)
            .min()
            .unwrap_or(0);

        info!(
            "Created ProviderRouter with {} providers, capacity_threshold: {}, total_rate_limit: {}, strategy: {:?}",
            providers.len(),
            capacity_threshold,
            total_rate_limit,
            strategy
        );

        Ok(Self {
            providers,
            capacity_threshold,
            strategy,
            total_effective_weight: AtomicU32::new(total_rate_limit.max(1)),
        })
    }

    /// Select provider based on text length and configured strategy
    pub fn select_provider(&self, text_len: usize) -> Option<&Arc<LLMProvider>> {
        trace!(
            "Selecting provider for text length: {} (threshold: {})",
            text_len,
            self.capacity_threshold
        );

        // Update effective weights based on current health status
        self.update_effective_weights();

        // Filter candidates based on capacity
        let candidates: Vec<&ProviderEntry> = if text_len < self.capacity_threshold {
            self.providers.iter().collect()
        } else {
            self.providers
                .iter()
                .filter(|p| p.provider.can_handle(text_len))
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

        match self.strategy {
            SelectionStrategy::RateBasedRandom => self.select_rate_based_random(&candidates),
            SelectionStrategy::SmoothRateBasedRoundRobin => {
                self.select_smooth_rate_based_rr(&candidates)
            }
        }
    }

    /// Update effective weights based on provider health
    /// Note: This is a placeholder for future async health checking
    /// Currently, health is managed internally by each provider
    fn update_effective_weights(&self) {
        // Health is managed internally by LLMProvider with threshold-based tracking
        // The effective weight is updated when provider health changes
    }

    /// Select provider using rate-based random strategy
    fn select_rate_based_random<'a>(
        &self,
        candidates: &[&'a ProviderEntry],
    ) -> Option<&'a Arc<LLMProvider>> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(&candidates[0].provider);
        }

        // Calculate total effective weight (based on rate_limit)
        let total_weight: u32 = candidates.iter().map(|p| p.effective_weight()).sum();
        
        if total_weight == 0 {
            // All rate limits are 0, use equal distribution
            let index = rand::random::<usize>() % candidates.len();
            trace!(
                "All rate limits are 0, randomly selected provider at index {}",
                index
            );
            return Some(&candidates[index].provider);
        }

        // Use random selection for rate-based distribution
        let target_weight = rand::random::<u32>() % total_weight;

        let mut current_weight = 0u32;
        for entry in candidates {
            current_weight += entry.effective_weight();
            if target_weight < current_weight {
                trace!(
                    "Rate-based selected provider {} with weight {} (target: {})",
                    entry.provider.id(),
                    entry.effective_weight(),
                    target_weight
                );
                return Some(&entry.provider);
            }
        }

        // Fallback to last candidate
        candidates.last().map(|e| &e.provider)
    }

    /// Select provider using smooth rate-based round-robin (Nginx algorithm)
    ///
    /// Algorithm:
    /// 1. On each request, increase current_weight by effective_weight (based on rate_limit)
    /// 2. Select provider with maximum current_weight
    /// 3. Decrease selected provider's current_weight by total_weight
    fn select_smooth_rate_based_rr<'a>(
        &self,
        candidates: &[&'a ProviderEntry],
    ) -> Option<&'a Arc<LLMProvider>> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(&candidates[0].provider);
        }

        let total_weight: u32 = candidates.iter().map(|p| p.effective_weight()).sum();
        
        if total_weight == 0 {
            // All rate limits are 0, use round-robin
            static INDEX: AtomicU32 = AtomicU32::new(0);
            let idx = INDEX.fetch_add(1, Ordering::Relaxed) as usize % candidates.len();
            return Some(&candidates[idx].provider);
        }

        // Find provider with max current_weight + effective_weight
        let mut best_entry: Option<&ProviderEntry> = None;
        let mut max_weight = 0u32;

        for entry in candidates {
            let effective = entry.effective_weight();
            let current = entry.current_weight.load(Ordering::Relaxed);
            let new_weight = current + effective;

            if new_weight > max_weight {
                max_weight = new_weight;
                best_entry = Some(entry);
            }
        }

        if let Some(entry) = best_entry {
            // Update current_weight: add effective_weight, then subtract total_weight
            let current = entry.current_weight.load(Ordering::Relaxed);
            let effective = entry.effective_weight();
            entry.current_weight.store(
                current + effective - total_weight,
                Ordering::Relaxed,
            );

            // Reset other providers' current_weight (they just had effective_weight added)
            for other in candidates {
                if !Arc::ptr_eq(&other.provider, &entry.provider) {
                    let effective = other.effective_weight();
                    other.current_weight.fetch_add(effective, Ordering::Relaxed);
                }
            }

            trace!(
                "Smooth rate-based RR selected provider {} (rate_limit: {}, total: {})",
                entry.provider.id(),
                effective,
                total_weight
            );

            return Some(&entry.provider);
        }

        // Fallback
        candidates.first().map(|e| &e.provider)
    }

    /// Get capacity threshold (minimum capacity among all providers)
    pub fn capacity_threshold(&self) -> usize {
        self.capacity_threshold
    }

    /// Get maximum capacity among all providers
    pub fn max_capacity(&self) -> usize {
        self.providers
            .iter()
            .map(|p| p.provider.max_input_chars())
            .max()
            .unwrap_or(0)
    }

    /// Get all providers
    pub fn providers(&self) -> Vec<Arc<LLMProvider>> {
        self.providers.iter().map(|p| p.provider.clone()).collect()
    }

    /// Get selection strategy
    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }

    /// Check if any provider can handle the given text length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.providers.iter().any(|p| p.provider.can_handle(text_len))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_strategy_default() {
        let strategy = SelectionStrategy::default();
        assert_eq!(strategy, SelectionStrategy::SmoothRateBasedRoundRobin);
    }

    #[test]
    fn test_selection_strategy_variants() {
        // Verify the strategy variants exist and can be compared
        let random = SelectionStrategy::RateBasedRandom;
        let round_robin = SelectionStrategy::SmoothRateBasedRoundRobin;
        assert_ne!(random as u8, round_robin as u8);
    }
}
