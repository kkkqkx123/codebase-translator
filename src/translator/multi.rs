//! Multi-translator with load balancing and failover
//!
//! This module provides a multi-translator implementation that supports
//! load balancing across multiple translation providers with automatic failover.
//! Uses static dispatch via TranslatorImpl enum for better performance.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::{Translator, TranslatorImpl};

/// Selection strategy for multi-translator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    RoundRobin,
    Weighted,
    Priority,
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
        if count >= 3 {
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
        if translators.is_empty() {
            return Err(TranslateError::Config(
                "At least one translator is required".to_string(),
            ));
        }

        let wrappers: Vec<TranslatorWrapper> = translators
            .into_iter()
            .map(|(t, w)| TranslatorWrapper::new(t, w))
            .collect();

        let max_retries = if max_retries == 0 { 3 } else { max_retries };

        info!(
            "Multi-translator created with {} translators, strategy: {:?}",
            wrappers.len(),
            strategy
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
            SelectionStrategy::Priority => self.select_priority(attempted),
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

        for i in 0..total {
            if !attempted.get(&i).copied().unwrap_or(false) {
                return Some(i);
            }
        }

        None
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

    /// Priority selection (by index order)
    fn select_priority(&self, attempted: &HashMap<usize, bool>) -> Option<usize> {
        for i in 0..self.translators.len() {
            if self.translators[i].is_healthy() && !attempted.get(&i).copied().unwrap_or(false) {
                return Some(i);
            }
        }

        for i in 0..self.translators.len() {
            if !attempted.get(&i).copied().unwrap_or(false) {
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
        let mut last_error = None;
        let mut attempted = HashMap::new();

        for attempt in 0..=self.max_retries {
            let index = match self.select_translator(&attempted) {
                Some(i) => i,
                None => {
                    break;
                }
            };

            attempted.insert(index, true);

            debug!(
                "Attempting translation with translator {} (attempt {}/{})",
                self.translators[index].translator.name(),
                attempt + 1,
                self.max_retries + 1
            );

            match self.translators[index]
                .translator
                .translate_single(text, source_lang, target_lang)
                .await
            {
                Ok(translated_text) => {
                    self.translators[index].reset_failure();
                    info!(
                        "Translation succeeded with translator {}",
                        self.translators[index].translator.name()
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
                    error!(
                        "Translation failed with translator {}: {}",
                        self.translators[index].translator.name(),
                        e
                    );
                    self.translators[index].increment_failure();
                    last_error = Some(e);
                }
            }
        }

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
