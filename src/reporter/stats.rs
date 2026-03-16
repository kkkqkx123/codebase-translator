//! Statistics collection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Translation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationStats {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_files: usize,
    pub processed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub total_units: usize,
    pub translated_units: usize,
    pub api_call_count: usize,
    pub error_count: usize,
    pub cache_hit_count: usize,
    pub cache_miss_count: usize,
    pub errors: Vec<ErrorRecord>,
}

/// Error record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub file_path: String,
    pub error: String,
    pub time: DateTime<Utc>,
}

impl Default for TranslationStats {
    fn default() -> Self {
        Self {
            start_time: Utc::now(),
            end_time: None,
            total_files: 0,
            processed_files: 0,
            skipped_files: 0,
            failed_files: 0,
            total_units: 0,
            translated_units: 0,
            api_call_count: 0,
            error_count: 0,
            cache_hit_count: 0,
            cache_miss_count: 0,
            errors: Vec::new(),
        }
    }
}

impl TranslationStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_processed(&mut self) {
        self.processed_files += 1;
    }

    pub fn record_skipped(&mut self) {
        self.skipped_files += 1;
    }

    pub fn record_failed(&mut self, file_path: &str, error: &str) {
        self.failed_files += 1;
        self.error_count += 1;
        self.errors.push(ErrorRecord {
            file_path: file_path.to_string(),
            error: error.to_string(),
            time: Utc::now(),
        });
    }

    pub fn record_total_files(&mut self, count: usize) {
        self.total_files = count;
    }

    pub fn record_units(&mut self, count: usize) {
        self.total_units += count;
    }

    pub fn record_translated(&mut self, count: usize) {
        self.translated_units += count;
    }

    pub fn record_api_call(&mut self, count: usize) {
        self.api_call_count += count;
    }

    pub fn record_cache_hit(&mut self) {
        self.cache_hit_count += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_miss_count += 1;
    }

    pub fn finalize(&mut self) {
        self.end_time = Some(Utc::now());
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn get_progress(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }
        (self.processed_files as f64 / self.total_files as f64) * 100.0
    }

    pub fn get_translation_progress(&self) -> f64 {
        if self.total_units == 0 {
            return 0.0;
        }
        (self.translated_units as f64 / self.total_units as f64) * 100.0
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let total = self.cache_hit_count + self.cache_miss_count;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hit_count as f64 / total as f64) * 100.0
    }
}

/// Thread-safe statistics wrapper using synchronous RwLock
#[derive(Debug, Clone)]
pub struct SharedStats {
    inner: Arc<RwLock<TranslationStats>>,
}

impl SharedStats {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TranslationStats::new())),
        }
    }

    pub fn record_processed(&self) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_processed();
        }
    }

    pub fn record_skipped(&self) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_skipped();
        }
    }

    pub fn record_failed(&self, file_path: &str, error: &str) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_failed(file_path, error);
        }
    }

    pub fn record_total_files(&self, count: usize) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_total_files(count);
        }
    }

    pub fn record_units(&self, count: usize) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_units(count);
        }
    }

    pub fn record_translated(&self, count: usize) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_translated(count);
        }
    }

    pub fn record_api_call(&self, count: usize) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_api_call(count);
        }
    }

    pub fn record_cache_hit(&self) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_cache_hit();
        }
    }

    pub fn record_cache_miss(&self) {
        if let Ok(mut stats) = self.inner.write() {
            stats.record_cache_miss();
        }
    }

    pub fn finalize(&self) {
        if let Ok(mut stats) = self.inner.write() {
            stats.finalize();
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
        if let Ok(mut stats) = self.inner.write() {
            *stats = TranslationStats::new();
        }
    }
}

impl Default for SharedStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_stats_default() {
        let stats = TranslationStats::new();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.processed_files, 0);
        assert_eq!(stats.skipped_files, 0);
        assert_eq!(stats.failed_files, 0);
        assert_eq!(stats.total_units, 0);
        assert_eq!(stats.translated_units, 0);
        assert_eq!(stats.api_call_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.cache_hit_count, 0);
        assert_eq!(stats.cache_miss_count, 0);
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_translation_stats_record_processed() {
        let mut stats = TranslationStats::new();
        stats.record_processed();
        assert_eq!(stats.processed_files, 1);
    }

    #[test]
    fn test_translation_stats_record_skipped() {
        let mut stats = TranslationStats::new();
        stats.record_skipped();
        assert_eq!(stats.skipped_files, 1);
    }

    #[test]
    fn test_translation_stats_record_failed() {
        let mut stats = TranslationStats::new();
        stats.record_failed("test.rs", "error message");
        assert_eq!(stats.failed_files, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.errors.len(), 1);
        assert_eq!(stats.errors[0].file_path, "test.rs");
        assert_eq!(stats.errors[0].error, "error message");
    }

    #[test]
    fn test_translation_stats_record_total_files() {
        let mut stats = TranslationStats::new();
        stats.record_total_files(100);
        assert_eq!(stats.total_files, 100);
    }

    #[test]
    fn test_translation_stats_record_units() {
        let mut stats = TranslationStats::new();
        stats.record_units(50);
        assert_eq!(stats.total_units, 50);
    }

    #[test]
    fn test_translation_stats_record_translated() {
        let mut stats = TranslationStats::new();
        stats.record_translated(30);
        assert_eq!(stats.translated_units, 30);
    }

    #[test]
    fn test_translation_stats_record_api_call() {
        let mut stats = TranslationStats::new();
        stats.record_api_call(5);
        assert_eq!(stats.api_call_count, 5);
    }

    #[test]
    fn test_translation_stats_record_cache_hit() {
        let mut stats = TranslationStats::new();
        stats.record_cache_hit();
        assert_eq!(stats.cache_hit_count, 1);
    }

    #[test]
    fn test_translation_stats_record_cache_miss() {
        let mut stats = TranslationStats::new();
        stats.record_cache_miss();
        assert_eq!(stats.cache_miss_count, 1);
    }

    #[test]
    fn test_translation_stats_finalize() {
        let mut stats = TranslationStats::new();
        assert!(stats.end_time.is_none());
        stats.finalize();
        assert!(stats.end_time.is_some());
    }

    #[test]
    fn test_translation_stats_has_errors() {
        let mut stats = TranslationStats::new();
        assert!(!stats.has_errors());
        stats.record_failed("test.rs", "error");
        assert!(stats.has_errors());
    }

    #[test]
    fn test_translation_stats_get_progress() {
        let mut stats = TranslationStats::new();
        assert_eq!(stats.get_progress(), 0.0);
        stats.record_total_files(100);
        stats.record_processed();
        stats.record_processed();
        assert_eq!(stats.get_progress(), 2.0);
    }

    #[test]
    fn test_translation_stats_get_translation_progress() {
        let mut stats = TranslationStats::new();
        assert_eq!(stats.get_translation_progress(), 0.0);
        stats.record_units(100);
        stats.record_translated(50);
        assert_eq!(stats.get_translation_progress(), 50.0);
    }

    #[test]
    fn test_translation_stats_get_cache_hit_rate() {
        let mut stats = TranslationStats::new();
        assert_eq!(stats.get_cache_hit_rate(), 0.0);
        stats.record_cache_hit();
        stats.record_cache_hit();
        stats.record_cache_miss();
        let hit_rate = stats.get_cache_hit_rate();
        assert!(
            (hit_rate - 66.66666666666667).abs() < 0.001,
            "Expected ~66.67%, got {}",
            hit_rate
        );
    }

    #[test]
    fn test_shared_stats_new() {
        let shared = SharedStats::new();
        let stats = shared.get_stats();
        assert_eq!(stats.total_files, 0);
    }

    #[test]
    fn test_shared_stats_default() {
        let shared: SharedStats = Default::default();
        let stats = shared.get_stats();
        assert_eq!(stats.total_files, 0);
    }

    #[test]
    fn test_shared_stats_record_processed() {
        let shared = SharedStats::new();
        shared.record_processed();
        let stats = shared.get_stats();
        assert_eq!(stats.processed_files, 1);
    }

    #[test]
    fn test_shared_stats_record_skipped() {
        let shared = SharedStats::new();
        shared.record_skipped();
        let stats = shared.get_stats();
        assert_eq!(stats.skipped_files, 1);
    }

    #[test]
    fn test_shared_stats_record_failed() {
        let shared = SharedStats::new();
        shared.record_failed("test.rs", "error message");
        let stats = shared.get_stats();
        assert_eq!(stats.failed_files, 1);
        assert_eq!(stats.errors.len(), 1);
    }

    #[test]
    fn test_shared_stats_record_total_files() {
        let shared = SharedStats::new();
        shared.record_total_files(100);
        let stats = shared.get_stats();
        assert_eq!(stats.total_files, 100);
    }

    #[test]
    fn test_shared_stats_record_units() {
        let shared = SharedStats::new();
        shared.record_units(50);
        let stats = shared.get_stats();
        assert_eq!(stats.total_units, 50);
    }

    #[test]
    fn test_shared_stats_record_translated() {
        let shared = SharedStats::new();
        shared.record_translated(30);
        let stats = shared.get_stats();
        assert_eq!(stats.translated_units, 30);
    }

    #[test]
    fn test_shared_stats_record_api_call() {
        let shared = SharedStats::new();
        shared.record_api_call(5);
        let stats = shared.get_stats();
        assert_eq!(stats.api_call_count, 5);
    }

    #[test]
    fn test_shared_stats_record_cache_hit() {
        let shared = SharedStats::new();
        shared.record_cache_hit();
        let stats = shared.get_stats();
        assert_eq!(stats.cache_hit_count, 1);
    }

    #[test]
    fn test_shared_stats_record_cache_miss() {
        let shared = SharedStats::new();
        shared.record_cache_miss();
        let stats = shared.get_stats();
        assert_eq!(stats.cache_miss_count, 1);
    }

    #[test]
    fn test_shared_stats_finalize() {
        let shared = SharedStats::new();
        shared.finalize();
        let stats = shared.get_stats();
        assert!(stats.end_time.is_some());
    }

    #[test]
    fn test_shared_stats_has_errors() {
        let shared = SharedStats::new();
        assert!(!shared.has_errors());
        shared.record_failed("test.rs", "error");
        assert!(shared.has_errors());
    }

    #[test]
    fn test_shared_stats_get_progress() {
        let shared = SharedStats::new();
        assert_eq!(shared.get_progress(), 0.0);
        shared.record_total_files(100);
        shared.record_processed();
        shared.record_processed();
        assert_eq!(shared.get_progress(), 2.0);
    }

    #[test]
    fn test_shared_stats_get_translation_progress() {
        let shared = SharedStats::new();
        assert_eq!(shared.get_translation_progress(), 0.0);
        shared.record_units(100);
        shared.record_translated(50);
        assert_eq!(shared.get_translation_progress(), 50.0);
    }

    #[test]
    fn test_shared_stats_get_cache_hit_rate() {
        let shared = SharedStats::new();
        assert_eq!(shared.get_cache_hit_rate(), 0.0);
        shared.record_cache_hit();
        shared.record_cache_hit();
        shared.record_cache_miss();
        let hit_rate = shared.get_cache_hit_rate();
        assert!(
            (hit_rate - 66.66666666666667).abs() < 0.001,
            "Expected ~66.67%, got {}",
            hit_rate
        );
    }

    #[test]
    fn test_shared_stats_reset() {
        let shared = SharedStats::new();
        shared.record_processed();
        shared.record_cache_hit();
        shared.reset();
        let stats = shared.get_stats();
        assert_eq!(stats.processed_files, 0);
        assert_eq!(stats.cache_hit_count, 0);
    }

    #[test]
    fn test_shared_stats_clone() {
        let shared = SharedStats::new();
        shared.record_processed();
        let cloned = shared.clone();
        let stats = cloned.get_stats();
        assert_eq!(stats.processed_files, 1);
    }

    #[test]
    fn test_error_record_clone() {
        let record = ErrorRecord {
            file_path: "test.rs".to_string(),
            error: "error".to_string(),
            time: Utc::now(),
        };
        let cloned = record.clone();
        assert_eq!(cloned.file_path, "test.rs");
    }
}
