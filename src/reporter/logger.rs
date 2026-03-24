use std::path::Path;
use crate::core::error::TranslateError;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Default)]
pub struct EventLogger;

impl EventLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn log_total_files(&self, count: usize) {
        debug!(count = count, "Total files to process");
    }

    pub fn log_file_processed(&self, path: &Path, units: usize) {
        debug!(
            file = %path.display(),
            units = units,
            "File processed"
        );
    }

    pub fn log_progress(&self, current: usize, total: usize) {
        debug!(current = current, total = total, "Progress update");
    }

    pub fn log_error(&self, path: &Path, error: &TranslateError) {
        warn!(
            file = %path.display(),
            error = %error,
            "Error occurred"
        );
    }

    pub fn log_skipped(&self, path: &Path) {
        debug!(
            file = %path.display(),
            "File skipped"
        );
    }

    pub fn log_api_call(&self, count: usize) {
        debug!(count = count, "API call made");
    }

    pub fn log_cache_hit(&self) {
        debug!("Cache hit");
    }

    pub fn log_cache_miss(&self) {
        debug!("Cache miss");
    }

    pub fn log_translator_call(
        &self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        debug!(
            translator = translator_type,
            latency_ms = latency_ms,
            success = success,
            chars = chars,
            "Translator call"
        );
    }

    pub fn log_llm_provider_call(
        &self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        debug!(
            provider_id = provider_id,
            provider_name = provider_name,
            model = model,
            latency_ms = latency_ms,
            success = success,
            chars = chars,
            "LLM provider call"
        );
    }

    pub fn log_report_generation(&self, format: &str) {
        info!(format = format, "Generating report");
    }

    pub fn log_report_saved(&self, path: &Path) {
        info!(
            path = %path.display(),
            "Report saved"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_logger_new() {
        EventLogger::new();
    }
}
