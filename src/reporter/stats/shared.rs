use super::provider::{LLMProviderStats, TranslatorStats};
use super::translation::TranslationStats;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct SharedStats {
    inner: Arc<RwLock<TranslationStats>>,
}

impl SharedStats {
    pub fn new() -> Self {
        debug!("Creating new shared statistics");
        Self {
            inner: Arc::new(RwLock::new(TranslationStats::new())),
        }
    }

    pub fn record_processed(&self) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_processed(),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_processed"),
        }
    }

    pub fn record_skipped(&self) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_skipped(),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_skipped"),
        }
    }

    pub fn record_failed(&self, file_path: &str, error: &str) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_failed(file_path, error),
            Err(e) => {
                warn!(error = %e, file_path = %file_path, "Failed to acquire write lock for record_failed")
            }
        }
    }

    pub fn record_total_files(&self, count: usize) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_total_files(count),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_total_files"),
        }
    }

    pub fn record_units(&self, count: usize) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_units(count),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_units"),
        }
    }

    pub fn record_translated(&self, count: usize) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_translated(count),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_translated"),
        }
    }

    pub fn record_api_call(&self, count: usize) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_api_call(count),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_api_call"),
        }
    }

    pub fn record_cache_hit(&self) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_cache_hit(),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_cache_hit"),
        }
    }

    pub fn record_cache_miss(&self) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_cache_miss(),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_cache_miss"),
        }
    }

    pub fn record_progress(&self, current: usize, total: usize) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_progress(current, total),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_progress"),
        }
    }

    pub fn finalize(&self) {
        match self.inner.write() {
            Ok(mut stats) => stats.finalize(),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for finalize"),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.inner.read().map(|s| s.has_errors()).unwrap_or(false)
    }

    pub fn get_progress(&self) -> f64 {
        self.inner.read().map(|s| s.get_progress()).unwrap_or(0.0)
    }

    pub fn get_translation_progress(&self) -> f64 {
        self.inner
            .read()
            .map(|s| s.get_translation_progress())
            .unwrap_or(0.0)
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        self.inner
            .read()
            .map(|s| s.get_cache_hit_rate())
            .unwrap_or(0.0)
    }

    pub fn get_stats(&self) -> TranslationStats {
        self.inner.read().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn reset(&self) {
        debug!("Resetting shared statistics");
        match self.inner.write() {
            Ok(mut stats) => *stats = TranslationStats::new(),
            Err(e) => warn!(error = %e, "Failed to acquire write lock for reset"),
        }
    }

    pub fn record_translator_call(
        &self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        match self.inner.write() {
            Ok(mut stats) => {
                stats.record_translator_call(translator_type, latency_ms, success, chars)
            }
            Err(e) => warn!(error = %e, "Failed to acquire write lock for record_translator_call"),
        }
    }

    pub fn record_llm_provider_call(
        &self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        match self.inner.write() {
            Ok(mut stats) => stats.record_llm_provider_call(
                provider_id,
                provider_name,
                model,
                latency_ms,
                success,
                chars,
            ),
            Err(e) => {
                warn!(error = %e, "Failed to acquire write lock for record_llm_provider_call")
            }
        }
    }

    pub fn get_translator_stats(&self, translator_type: &str) -> Option<TranslatorStats> {
        self.inner
            .read()
            .ok()
            .and_then(|s| s.get_translator_stats(translator_type).cloned())
    }

    pub fn get_llm_provider_stats(&self, provider_id: &str) -> Option<LLMProviderStats> {
        self.inner
            .read()
            .ok()
            .and_then(|s| s.get_llm_provider_stats(provider_id).cloned())
    }

    pub fn get_all_translator_stats(&self) -> Vec<TranslatorStats> {
        self.inner
            .read()
            .map(|s| s.get_all_translator_stats().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_all_llm_provider_stats(&self) -> Vec<LLMProviderStats> {
        self.inner
            .read()
            .map(|s| {
                s.get_all_llm_provider_stats()
                    .into_iter()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for SharedStats {
    fn default() -> Self {
        Self::new()
    }
}
