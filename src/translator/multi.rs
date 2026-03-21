//! Multi-translator with load balancing and failover
//!
//! This module provides a multi-translator implementation that supports
//! load balancing across multiple translation providers with automatic failover.
//! Uses static dispatch via TranslatorImpl enum for better performance.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::{Translator, TranslatorImpl};

/// Selection strategy for multi-translator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    RoundRobin,
    Weighted,
}

impl std::str::FromStr for SelectionStrategy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "round_robin" | "roundrobin" => Ok(SelectionStrategy::RoundRobin),
            "weighted" => Ok(SelectionStrategy::Weighted),
            _ => Err(format!("Unknown selection strategy: {}", s)),
        }
    }
}

/// Translator wrapper with health tracking
/// Uses static dispatch via TranslatorImpl for better performance.
struct TranslatorWrapper {
    translator: Arc<TranslatorImpl>,
    weight: u32,
    healthy: Arc<AtomicBool>,
    failure_count: Arc<AtomicU32>,
}

impl std::fmt::Debug for TranslatorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranslatorWrapper")
            .field("translator", &self.translator)
            .field("weight", &self.weight)
            .field("healthy", &self.healthy.load(Ordering::Relaxed))
            .field("failure_count", &self.failure_count.load(Ordering::Relaxed))
            .finish()
    }
}

impl TranslatorWrapper {
    fn new(translator: Arc<TranslatorImpl>, weight: u32) -> Self {
        Self {
            translator,
            weight,
            healthy: Arc::new(AtomicBool::new(true)),
            failure_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed)
    }

    fn mark_healthy(&self) {
        self.healthy.store(true, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
    }

    fn mark_unhealthy(&self) {
        self.healthy.store(false, Ordering::Relaxed);
    }

    fn increment_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        // Immediately mark as unhealthy on first failure for real-time health update
        if count >= 1 {
            self.mark_unhealthy();
        }
    }

    fn reset_failure(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.mark_healthy();
    }
}

/// Multi-translator with load balancing and failover
/// Uses static dispatch via TranslatorImpl for better performance.
#[derive(Debug)]
pub struct MultiTranslator {
    translators: Vec<TranslatorWrapper>,
    strategy: SelectionStrategy,
    current_index: Arc<AtomicU64>,
    max_retries: usize,
}

impl MultiTranslator {
    /// Create a new multi-translator
    /// Uses static dispatch via TranslatorImpl for better performance.
    pub fn new(
        translators: Vec<(Arc<TranslatorImpl>, u32)>,
        strategy: SelectionStrategy,
        max_retries: usize,
    ) -> Result<Self> {
        // Validate input
        if translators.is_empty() {
            return Err(TranslateError::Config(
                "At least one translator is required".to_string(),
            ));
        }

        // Check for duplicate translators by name
        let mut seen_names = std::collections::HashSet::new();
        for (translator, _) in &translators {
            if !seen_names.insert(translator.name()) {
                warn!(
                    "Duplicate translator name detected: {}. This may affect routing behavior.",
                    translator.name()
                );
            }
        }

        let wrappers: Vec<TranslatorWrapper> = translators
            .into_iter()
            .map(|(t, w)| TranslatorWrapper::new(t, w))
            .collect();

        // Validate that at least one translator is available
        if wrappers.is_empty() {
            return Err(TranslateError::Config(
                "No valid translators configured".to_string(),
            ));
        }

        // Validate max_retries
        let max_retries = if max_retries == 0 {
            3
        } else if max_retries > 10 {
            warn!("max_retries {} is too high, limiting to 10", max_retries);
            10
        } else {
            max_retries
        };

        info!(
            "Multi-translator created with {} translators, strategy: {:?}, max_retries: {}",
            wrappers.len(),
            strategy,
            max_retries
        );

        Ok(Self {
            translators: wrappers,
            strategy,
            current_index: Arc::new(AtomicU64::new(0)),
            max_retries,
        })
    }

    /// Select translator based on strategy
    fn select_translator(&self, attempted: &HashMap<usize, bool>) -> Option<usize> {
        match self.strategy {
            SelectionStrategy::RoundRobin => self.select_round_robin(attempted),
            SelectionStrategy::Weighted => self.select_weighted(attempted),
        }
    }

    /// Round-robin selection
    fn select_round_robin(&self, attempted: &HashMap<usize, bool>) -> Option<usize> {
        let total = self.translators.len();

        for _ in 0..total {
            let index = self.current_index.fetch_add(1, Ordering::Relaxed) as usize % total;

            if self.translators[index].is_healthy()
                && !attempted.get(&index).copied().unwrap_or(false)
            {
                return Some(index);
            }
        }

        (0..total).find(|&i| !attempted.get(&i).copied().unwrap_or(false))
    }

    /// Weighted selection
    fn select_weighted(&self, attempted: &HashMap<usize, bool>) -> Option<usize> {
        let total_weight: u32 = self
            .translators
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                self.translators[*i].is_healthy() && !attempted.get(i).copied().unwrap_or(false)
            })
            .map(|(_, w)| w.weight)
            .sum();

        if total_weight == 0 {
            for i in 0..self.translators.len() {
                if !attempted.get(&i).copied().unwrap_or(false) {
                    return Some(i);
                }
            }
            return None;
        }

        let target = self.current_index.fetch_add(1, Ordering::Relaxed) as u32 % total_weight;
        let mut current_weight = 0u32;

        for (i, wrapper) in self.translators.iter().enumerate() {
            if !wrapper.is_healthy() || attempted.get(&i).copied().unwrap_or(false) {
                continue;
            }

            current_weight += wrapper.weight;
            if target < current_weight {
                return Some(i);
            }
        }

        None
    }

    /// Translate with failover
    async fn translate_with_failover(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = std::time::Instant::now();
        let mut last_error = None;
        let mut attempted = HashMap::new();
        let text_len = text.len();

        for attempt in 0..=self.max_retries {
            let index = match self.select_translator(&attempted) {
                Some(i) => i,
                None => {
                    error!("No more translators available for selection");
                    break;
                }
            };

            attempted.insert(index, true);

            let attempt_start = std::time::Instant::now();
            let translator_name = self.translators[index].translator.name();
            let translator_healthy = self.translators[index].is_healthy();

            debug!(
                "Attempting translation with translator {} (attempt {}/{}, healthy: {}, text: {} chars)",
                translator_name,
                attempt + 1,
                self.max_retries + 1,
                translator_healthy,
                text_len
            );

            match self.translators[index]
                .translator
                .translate_single(text, source_lang, target_lang)
                .await
            {
                Ok(translated_text) => {
                    let latency = attempt_start.elapsed();
                    self.translators[index].reset_failure();
                    info!(
                        "Translation succeeded with translator {} in {:?} (attempt {}/{})",
                        translator_name,
                        latency,
                        attempt + 1,
                        self.max_retries + 1
                    );
                    return Ok(TranslateResponse {
                        original_text: text.to_string(),
                        translated_text,
                        source_lang: source_lang.to_string(),
                        target_lang: target_lang.to_string(),
                        ..Default::default()
                    });
                }
                Err(e) => {
                    let latency = attempt_start.elapsed();
                    error!(
                        "Translation failed with translator {} in {:?}: {}",
                        translator_name, latency, e
                    );
                    self.translators[index].increment_failure();
                    last_error = Some(e);
                }
            }
        }

        let total_latency = start_time.elapsed();
        error!(
            "All translators failed after {:?} ({} attempts made)",
            total_latency,
            attempted.len()
        );
        Err(last_error
            .unwrap_or_else(|| TranslateError::Translation("All translators failed".to_string())))
    }
}

#[async_trait]
impl Translator for MultiTranslator {
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let response = self
                .translate_with_failover(text, "AUTO", target_lang)
                .await?;
            results.push(response.translated_text);
        }

        Ok(results)
    }

    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let response = self
            .translate_with_failover(text, source_lang, target_lang)
            .await?;
        Ok(response.translated_text)
    }

    fn name(&self) -> &str {
        "multi"
    }

    async fn is_available(&self) -> bool {
        self.translators.iter().any(|t| t.is_healthy())
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        vec!["AUTO"]
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        vec!["EN", "ZH", "JA", "KO", "FR", "DE", "ES", "RU"]
    }

    fn max_input_chars(&self) -> usize {
        // Return the maximum capacity among all translators
        self.translators
            .iter()
            .map(|t| t.translator.max_input_chars())
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_selection_strategy_from_str() {
        assert_eq!(
            SelectionStrategy::from_str("round_robin").unwrap(),
            SelectionStrategy::RoundRobin
        );
        assert_eq!(
            SelectionStrategy::from_str("roundrobin").unwrap(),
            SelectionStrategy::RoundRobin
        );
        assert_eq!(
            SelectionStrategy::from_str("weighted").unwrap(),
            SelectionStrategy::Weighted
        );
        assert!(SelectionStrategy::from_str("unknown").is_err());
    }
}
