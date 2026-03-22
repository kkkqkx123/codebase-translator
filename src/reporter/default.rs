//! Default reporter implementation

use chrono::Utc;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::core::error::TranslateError;
use crate::reporter::r#trait::{ReportFormat, Reporter};
use crate::reporter::stats::TranslationStats;
use tracing::{debug, info, warn};

/// Default reporter implementation
///
/// This reporter only tracks progress and errors for logging purposes.
/// Statistics are collected externally and passed in when generating reports.
#[derive(Debug, Clone)]
pub struct DefaultReporter {
    /// Total files to process (for progress tracking)
    total_files: Arc<RwLock<usize>>,
    /// Current progress (for progress tracking)
    current_progress: Arc<RwLock<usize>>,
    /// Error flag
    has_errors: Arc<RwLock<bool>>,
    /// Final stats (set during finalize)
    final_stats: Arc<RwLock<Option<TranslationStats>>>,
}

impl DefaultReporter {
    pub fn new() -> Self {
        Self {
            total_files: Arc::new(RwLock::new(0)),
            current_progress: Arc::new(RwLock::new(0)),
            has_errors: Arc::new(RwLock::new(false)),
            final_stats: Arc::new(RwLock::new(None)),
        }
    }

    fn generate_text_report(&self, stats: &TranslationStats) -> Result<String, TranslateError> {
        info!(
            files = stats.processed_files,
            units = stats.translated_units,
            errors = stats.error_count,
            "Generating translation report"
        );

        let end_time = stats
            .end_time
            .ok_or_else(|| TranslateError::Parse("Stats should be finalized".to_string()))?;

        let duration = end_time.signed_duration_since(stats.start_time);

        let mut report = String::new();
        report.push_str(&format!("\n{}\n", "=".repeat(60)));
        report.push_str("Translation Report\n");
        report.push_str(&format!("{}\n\n", "=".repeat(60)));

        report.push_str("Time:\n");
        report.push_str(&format!(
            "  Start:      {}\n",
            stats.start_time.format("%Y-%m-%d %H:%M:%S")
        ));
        report.push_str(&format!(
            "  End:        {}\n",
            end_time.format("%Y-%m-%d %H:%M:%S")
        ));
        report.push_str(&format!(
            "  Duration:   {:.3}s\n",
            duration.num_milliseconds() as f64 / 1000.0
        ));
        report.push_str(&format!(
            "  Speed:      {:.1} files/s\n\n",
            stats.avg_speed_files_per_sec
        ));

        report.push_str("Files:\n");
        report.push_str(&format!("  Total:      {}\n", stats.total_files));
        report.push_str(&format!("  Processed:  {}\n", stats.processed_files));
        report.push_str(&format!("  Skipped:    {}\n", stats.skipped_files));
        report.push_str(&format!("  Failed:     {}\n\n", stats.failed_files));

        report.push_str("Translation Units:\n");
        report.push_str(&format!("  Total:      {}\n", stats.total_units));
        report.push_str(&format!("  Translated: {}\n", stats.translated_units));

        if stats.total_units > 0 {
            let percentage = stats.get_translation_progress();
            report.push_str(&format!("  Progress:   {:.1}%\n\n", percentage));
        }

        report.push_str("API Calls:\n");
        report.push_str(&format!("  Total:      {}\n\n", stats.api_call_count));

        report.push_str("Cache:\n");
        report.push_str(&format!("  Hits:       {}\n", stats.cache_hit_count));
        report.push_str(&format!("  Misses:     {}\n", stats.cache_miss_count));

        if stats.cache_hit_count + stats.cache_miss_count > 0 {
            let hit_rate = stats.get_cache_hit_rate();
            report.push_str(&format!("  Hit Rate:   {:.1}%\n\n", hit_rate));
        }

        if stats.error_count > 0 {
            report.push_str(&format!("Errors ({})\n", stats.error_count));
            for (i, err) in stats.errors.iter().enumerate() {
                report.push_str(&format!("  {}. {}: {}\n", i + 1, err.file_path, err.error));
            }
            report.push('\n');
        }

        report.push_str(&format!("{}\n", "=".repeat(60)));

        Ok(report)
    }

    fn generate_json_report(&self, stats: &TranslationStats) -> Result<String, TranslateError> {
        debug!("Generating JSON report");
        serde_json::to_string_pretty(stats)
            .map_err(|e| TranslateError::Parse(format!("Failed to serialize JSON report: {}", e)))
    }
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for DefaultReporter {
    fn report_total_files(&self, count: usize) {
        debug!(count = count, "Reporting total files");
        if let Ok(mut total) = self.total_files.write() {
            *total = count;
        }
    }

    fn report_file(&self, path: &Path, units: usize) {
        debug!(
            file = %path.display(),
            units = units,
            "Reporting file processed"
        );
        // Only for logging, does not affect stats
    }

    fn report_progress(&self, current: usize, total: usize) {
        debug!(current = current, total = total, "Reporting progress");
        if let Ok(mut progress) = self.current_progress.write() {
            *progress = current;
        }
    }

    fn report_error(&self, path: &Path, error: &TranslateError) {
        warn!(
            file = %path.display(),
            error = %error,
            "Reporting error"
        );
        if let Ok(mut has_errors) = self.has_errors.write() {
            *has_errors = true;
        }
    }

    fn report_skipped(&self, path: &Path) {
        debug!(
            file = %path.display(),
            "Reporting skipped file"
        );
        // Only for logging, does not affect stats
    }

    fn report_api_call(&self, count: usize) {
        debug!(count = count, "Reporting API call");
        // Only for logging, does not affect stats
    }

    fn report_cache_hit(&self) {
        debug!("Reporting cache hit");
        // Only for logging, does not affect stats
    }

    fn report_cache_miss(&self) {
        debug!("Reporting cache miss");
        // Only for logging, does not affect stats
    }

    fn final_report(
        &self,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<String, TranslateError> {
        debug!(
            format = ?format,
            "Generating final report"
        );

        match format {
            ReportFormat::Text => self.generate_text_report(stats),
            ReportFormat::Json => self.generate_json_report(stats),
        }
    }

    fn get_stats(&self) -> Option<TranslationStats> {
        self.final_stats.read().ok().and_then(|s| s.clone())
    }

    fn has_errors(&self) -> bool {
        self.has_errors.read().map(|e| *e).unwrap_or(false)
    }

    fn get_progress(&self) -> f64 {
        let total = self.total_files.read().map(|t| *t).unwrap_or(0);
        let current = self.current_progress.read().map(|c| *c).unwrap_or(0);
        if total == 0 {
            0.0
        } else {
            (current as f64 / total as f64) * 100.0
        }
    }

    fn finalize(&self, stats: &TranslationStats) {
        info!("Finalizing reporter with external statistics");
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
        debug!(
            path = %path.display(),
            format = ?format,
            "Saving report to file"
        );
        let report = self.final_report(stats, format)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TranslateError::Io(e.to_string()))?;
        }

        std::fs::write(path, report).map_err(|e| TranslateError::Io(e.to_string()))?;

        info!(
            path = %path.display(),
            "Report saved successfully"
        );
        Ok(())
    }

    fn save_report_with_template(
        &self,
        dir: &Path,
        template: &str,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<std::path::PathBuf, TranslateError> {
        debug!(
            dir = %dir.display(),
            template = template,
            format = ?format,
            "Saving report with template"
        );
        std::fs::create_dir_all(dir).map_err(|e| TranslateError::Io(e.to_string()))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let ext = match format {
            ReportFormat::Text => "txt",
            ReportFormat::Json => "json",
        };
        let filename = template
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{format}", ext);
        let path = dir.join(filename);

        self.save_report(&path, stats, format)?;

        info!(
            path = %path.display(),
            "Report saved with template"
        );
        Ok(path)
    }
}

/// Create a new default reporter
pub fn create_reporter() -> Arc<dyn Reporter> {
    Arc::new(DefaultReporter::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
    fn test_default_reporter_report_error() {
        let reporter = DefaultReporter::new();
        assert!(!reporter.has_errors());
        let error = TranslateError::Io("test error".to_string());
        reporter.report_error(Path::new("test.rs"), &error);
        assert!(reporter.has_errors());
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
