use super::error::ErrorRecord;
use super::provider::{LLMProviderStats, TranslatorStats};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

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
    pub total_duration_ms: u64,
    pub avg_speed_files_per_sec: f64,
    pub translator_stats: HashMap<String, TranslatorStats>,
    pub llm_provider_stats: HashMap<String, LLMProviderStats>,
    /// Current progress tracking
    pub current_progress: usize,
    pub total_progress: usize,
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
            total_duration_ms: 0,
            avg_speed_files_per_sec: 0.0,
            translator_stats: HashMap::new(),
            llm_provider_stats: HashMap::new(),
            current_progress: 0,
            total_progress: 0,
        }
    }
}

impl TranslationStats {
    pub fn new() -> Self {
        debug!("Creating new translation statistics");
        Self::default()
    }

    pub fn record_processed(&mut self) {
        debug!(
            processed = self.processed_files + 1,
            "Recording file processed"
        );
        self.processed_files += 1;
    }

    pub fn record_skipped(&mut self) {
        debug!(skipped = self.skipped_files + 1, "Recording file skipped");
        self.skipped_files += 1;
    }

    pub fn record_failed(&mut self, file_path: &str, error: &str) {
        debug!(
            file = %file_path,
            error = %error,
            "Recording file failed"
        );
        self.failed_files += 1;
        self.error_count += 1;
        self.errors.push(ErrorRecord {
            file_path: file_path.to_string(),
            error: error.to_string(),
            time: Utc::now(),
        });
    }

    pub fn record_total_files(&mut self, count: usize) {
        debug!(total_files = count, "Recording total files");
        self.total_files = count;
    }

    pub fn record_units(&mut self, count: usize) {
        debug!(
            units = count,
            total = self.total_units + count,
            "Recording translation units"
        );
        self.total_units += count;
    }

    pub fn record_translated(&mut self, count: usize) {
        debug!(
            translated = count,
            total = self.translated_units + count,
            "Recording translated units"
        );
        self.translated_units += count;
    }

    pub fn record_api_call(&mut self, count: usize) {
        debug!(
            api_calls = count,
            total = self.api_call_count + count,
            "Recording API calls"
        );
        self.api_call_count += count;
    }

    pub fn record_cache_hit(&mut self) {
        debug!(cache_hits = self.cache_hit_count + 1, "Recording cache hit");
        self.cache_hit_count += 1;
    }

    pub fn record_cache_miss(&mut self) {
        debug!(
            cache_misses = self.cache_miss_count + 1,
            "Recording cache miss"
        );
        self.cache_miss_count += 1;
    }

    pub fn record_progress(&mut self, current: usize, total: usize) {
        debug!(current = current, total = total, "Recording progress");
        self.current_progress = current;
        self.total_progress = total;
    }

    pub fn finalize(&mut self) {
        info!(
            total_files = self.total_files,
            processed_files = self.processed_files,
            total_units = self.total_units,
            translated_units = self.translated_units,
            api_calls = self.api_call_count,
            cache_hits = self.cache_hit_count,
            cache_misses = self.cache_miss_count,
            "Finalizing translation statistics"
        );
        self.end_time = Some(Utc::now());

        if let Some(end_time) = self.end_time {
            let duration = end_time.signed_duration_since(self.start_time);
            self.total_duration_ms = duration.num_milliseconds() as u64;

            if self.total_duration_ms > 0 {
                self.avg_speed_files_per_sec =
                    (self.processed_files as f64) / (self.total_duration_ms as f64 / 1000.0);
            }
        }

        debug!(
            duration_ms = self.total_duration_ms,
            avg_speed = self.avg_speed_files_per_sec,
            "Statistics finalized"
        );
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

    pub fn record_translator_call(
        &mut self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        let stats = self
            .translator_stats
            .entry(translator_type.to_string())
            .or_insert_with(|| TranslatorStats::new(translator_type.to_string()));
        stats.record_call(latency_ms, success, chars);
    }

    pub fn record_llm_provider_call(
        &mut self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        let stats = self
            .llm_provider_stats
            .entry(provider_id.to_string())
            .or_insert_with(|| {
                LLMProviderStats::new(
                    provider_id.to_string(),
                    provider_name.to_string(),
                    model.to_string(),
                )
            });
        stats.record_call(latency_ms, success, chars);
    }

    pub fn get_translator_stats(&self, translator_type: &str) -> Option<&TranslatorStats> {
        self.translator_stats.get(translator_type)
    }

    pub fn get_llm_provider_stats(&self, provider_id: &str) -> Option<&LLMProviderStats> {
        self.llm_provider_stats.get(provider_id)
    }

    pub fn get_all_translator_stats(&self) -> Vec<&TranslatorStats> {
        self.translator_stats.values().collect()
    }

    pub fn get_all_llm_provider_stats(&self) -> Vec<&LLMProviderStats> {
        self.llm_provider_stats.values().collect()
    }

    /// Merge another stats into this one
    pub fn merge(&mut self, other: &TranslationStats) {
        self.total_files += other.total_files;
        self.processed_files += other.processed_files;
        self.skipped_files += other.skipped_files;
        self.failed_files += other.failed_files;
        self.total_units += other.total_units;
        self.translated_units += other.translated_units;
        self.api_call_count += other.api_call_count;
        self.error_count += other.error_count;
        self.cache_hit_count += other.cache_hit_count;
        self.cache_miss_count += other.cache_miss_count;
        // Note: translator_stats and llm_provider_stats are not merged
        // as they require more complex merging logic
    }
}
