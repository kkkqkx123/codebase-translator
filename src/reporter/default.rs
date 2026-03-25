//! Default reporter implementation

use std::path::Path;
use std::sync::Arc;

use crate::core::error::TranslateError;
use crate::reporter::generator::{DefaultReportGenerator, ReportGenerator};
use crate::reporter::logger::EventLogger;
use crate::reporter::progress::ProgressTracker;
use crate::reporter::r#trait::{ReportFormat, Reporter};
use crate::reporter::stats::{SharedStats, TranslationStats};

#[derive(Debug, Clone)]
pub struct DefaultReporter {
    progress_tracker: ProgressTracker,
    event_logger: EventLogger,
    report_generator: DefaultReportGenerator,
    shared_stats: Option<Arc<SharedStats>>,
    final_stats: Arc<std::sync::RwLock<Option<TranslationStats>>>,
}

impl DefaultReporter {
    pub fn new() -> Self {
        Self {
            progress_tracker: ProgressTracker::new(),
            event_logger: EventLogger::new(),
            report_generator: DefaultReportGenerator::new(),
            shared_stats: None,
            final_stats: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn with_shared_stats(shared_stats: Arc<SharedStats>) -> Self {
        Self {
            progress_tracker: ProgressTracker::new(),
            event_logger: EventLogger::new(),
            report_generator: DefaultReportGenerator::new(),
            shared_stats: Some(shared_stats),
            final_stats: Arc::new(std::sync::RwLock::new(None)),
        }
    }
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for DefaultReporter {
    fn report_total_files(&self, count: usize) {
        self.progress_tracker.set_total(count);
        self.event_logger.log_total_files(count);
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_total_files(count);
        }
    }

    fn report_file(&self, path: &Path, units: usize) {
        self.event_logger.log_file_processed(path, units);
    }

    fn report_progress(&self, current: usize, total: usize) {
        self.progress_tracker.update(current);
        self.event_logger.log_progress(current, total);
    }

    fn report_error(&self, path: &Path, error: &TranslateError) {
        self.event_logger.log_error(path, error);
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_failed(&path.to_string_lossy(), &error.to_string());
        }
    }

    fn report_skipped(&self, path: &Path) {
        self.event_logger.log_skipped(path);
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_skipped();
        }
    }

    fn report_api_call(&self, count: usize) {
        self.event_logger.log_api_call(count);
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_api_call(count);
        }
    }

    fn report_cache_hit(&self) {
        self.event_logger.log_cache_hit();
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_cache_hit();
        }
    }

    fn report_cache_miss(&self) {
        self.event_logger.log_cache_miss();
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_cache_miss();
        }
    }

    fn report_translator_call(
        &self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        self.event_logger
            .log_translator_call(translator_type, latency_ms, success, chars);
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_translator_call(translator_type, latency_ms, success, chars);
        }
    }

    fn report_llm_provider_call(
        &self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    ) {
        self.event_logger.log_llm_provider_call(
            provider_id,
            provider_name,
            model,
            latency_ms,
            success,
            chars,
        );
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.record_llm_provider_call(
                provider_id,
                provider_name,
                model,
                latency_ms,
                success,
                chars,
            );
        }
    }

    fn final_report(
        &self,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<String, TranslateError> {
        self.event_logger
            .log_report_generation(&format!("{:?}", format));
        self.report_generator.generate(stats, format)
    }

    fn get_stats(&self) -> Option<TranslationStats> {
        self.final_stats.read().ok().and_then(|s| s.clone())
    }

    fn has_errors(&self) -> bool {
        if let Some(ref shared_stats) = self.shared_stats {
            shared_stats.has_errors()
        } else {
            false
        }
    }

    fn get_progress(&self) -> f64 {
        self.progress_tracker.get_percentage()
    }

    fn finalize(&self, stats: &TranslationStats) {
        if let Ok(mut final_stats) = self.final_stats.write() {
            *final_stats = Some(stats.clone());
        }
    }

    fn save_report(
        &self,
        path: &Path,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<(), TranslateError> {
        self.report_generator.save(path, stats, format)?;
        self.event_logger.log_report_saved(path);
        Ok(())
    }

    fn save_report_with_template(
        &self,
        dir: &Path,
        template: &str,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<std::path::PathBuf, TranslateError> {
        let path = self
            .report_generator
            .save_with_template(dir, template, stats, format)?;
        self.event_logger.log_report_saved(&path);
        Ok(path)
    }
}

pub fn create_reporter() -> Arc<dyn Reporter> {
    Arc::new(DefaultReporter::new())
}

pub fn create_reporter_with_stats(shared_stats: Arc<SharedStats>) -> Arc<dyn Reporter> {
    Arc::new(DefaultReporter::with_shared_stats(shared_stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_reporter_new() {
        let reporter = DefaultReporter::new();
        assert!(!reporter.has_errors());
        assert_eq!(reporter.get_progress(), 0.0);
    }

    #[test]
    fn test_default_reporter_default() {
        let reporter = DefaultReporter::default();
        assert!(!reporter.has_errors());
    }

    #[test]
    fn test_default_reporter_report_total_files() {
        let reporter = DefaultReporter::new();
        reporter.report_total_files(100);
        assert_eq!(reporter.get_progress(), 0.0);
    }

    #[test]
    fn test_default_reporter_report_progress() {
        let reporter = DefaultReporter::new();
        reporter.report_total_files(100);
        reporter.report_progress(50, 100);
        assert_eq!(reporter.get_progress(), 50.0);
    }

    #[test]
    fn test_default_reporter_finalize() {
        let reporter = DefaultReporter::new();
        let mut stats = TranslationStats::new();
        stats.total_files = 10;
        stats.processed_files = 5;
        stats.finalize();

        reporter.finalize(&stats);
        let retrieved = reporter.get_stats();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().total_files, 10);
    }
}
