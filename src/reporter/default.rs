//! Default reporter implementation

use chrono::Utc;
use std::path::Path;
use std::sync::Arc;

use crate::core::error::TranslateError;
use crate::reporter::r#trait::{ReportFormat, Reporter};
use crate::reporter::stats::SharedStats;
use tracing::{debug, info, warn};

/// Default reporter implementation
#[derive(Debug, Clone)]
pub struct DefaultReporter {
    stats: SharedStats,
}

impl DefaultReporter {
    pub fn new() -> Self {
        Self {
            stats: SharedStats::new(),
        }
    }

    fn generate_text_report(
        &self,
        stats: &crate::reporter::stats::TranslationStats,
    ) -> Result<String, TranslateError> {
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
            report.push_str(&format!("Errors ({}):\n", stats.error_count));
            for (i, err) in stats.errors.iter().enumerate() {
                report.push_str(&format!("  {}. {}: {}\n", i + 1, err.file_path, err.error));
            }
            report.push('\n');
        }

        report.push_str(&format!("{}\n", "=".repeat(60)));

        Ok(report)
    }

    fn generate_json_report(
        &self,
        stats: &crate::reporter::stats::TranslationStats,
    ) -> Result<String, TranslateError> {
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
    fn report_file(&self, path: &Path, units: usize) {
        debug!(
            file = %path.display(),
            units = units,
            "Reporting file processed"
        );
        self.stats.record_units(units);
        self.stats.record_processed();
    }

    fn report_progress(&self, current: usize, total: usize) {
        debug!(current = current, total = total, "Reporting progress");
        self.stats.record_progress(current, total);
    }

    fn report_error(&self, path: &Path, error: &TranslateError) {
        warn!(
            file = %path.display(),
            error = %error,
            "Reporting error"
        );
        self.stats.record_failed(&path.to_string_lossy(), &error.to_string());
    }

    fn report_skipped(&self, path: &Path) {
        debug!(
            file = %path.display(),
            "Reporting skipped file"
        );
        self.stats.record_skipped();
    }

    fn report_api_call(&self, count: usize) {
        debug!(count = count, "Reporting API call");
        self.stats.record_api_call(count);
    }

    fn report_cache_hit(&self) {
        debug!("Reporting cache hit");
        self.stats.record_cache_hit();
    }

    fn report_cache_miss(&self) {
        debug!("Reporting cache miss");
        self.stats.record_cache_miss();
    }

    fn final_report(&self, format: ReportFormat) -> Result<String, TranslateError> {
        debug!(
            format = ?format,
            "Generating final report"
        );
        let stats = self.stats.get_stats();

        match format {
            ReportFormat::Text => self.generate_text_report(&stats),
            ReportFormat::Json => self.generate_json_report(&stats),
        }
    }

    fn get_stats(&self) -> crate::reporter::stats::TranslationStats {
        self.stats.get_stats()
    }

    fn has_errors(&self) -> bool {
        self.stats.has_errors()
    }

    fn get_progress(&self) -> f64 {
        self.stats.get_progress()
    }

    fn finalize(&self) {
        info!("Finalizing reporter statistics");
        self.stats.finalize();
    }

    fn save_report(&self, path: &Path, format: ReportFormat) -> Result<(), TranslateError> {
        debug!(
            path = %path.display(),
            format = ?format,
            "Saving report to file"
        );
        let report = self.final_report(format)?;

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

        self.save_report(&path, format)?;

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
        let stats = reporter.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.processed_files, 0);
    }

    #[test]
    fn test_default_reporter_default() {
        let reporter = DefaultReporter::default();
        let stats = reporter.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.processed_files, 0);
    }

    #[test]
    fn test_default_reporter_report_file() {
        let reporter = DefaultReporter::new();
        reporter.report_file(Path::new("test.rs"), 5);
        let stats = reporter.get_stats();
        assert_eq!(stats.processed_files, 1);
        assert_eq!(stats.total_units, 5);
    }

    #[test]
    fn test_default_reporter_report_skipped() {
        let reporter = DefaultReporter::new();
        reporter.report_skipped(Path::new("test.rs"));
        let stats = reporter.get_stats();
        assert_eq!(stats.skipped_files, 1);
    }

    #[test]
    fn test_default_reporter_report_api_call() {
        let reporter = DefaultReporter::new();
        reporter.report_api_call(5);
        let stats = reporter.get_stats();
        assert_eq!(stats.api_call_count, 5);
    }

    #[test]
    fn test_default_reporter_report_cache_hit() {
        let reporter = DefaultReporter::new();
        reporter.report_cache_hit();
        let stats = reporter.get_stats();
        assert_eq!(stats.cache_hit_count, 1);
    }

    #[test]
    fn test_default_reporter_report_cache_miss() {
        let reporter = DefaultReporter::new();
        reporter.report_cache_miss();
        let stats = reporter.get_stats();
        assert_eq!(stats.cache_miss_count, 1);
    }

    #[test]
    fn test_default_reporter_has_errors() {
        let reporter = DefaultReporter::new();
        assert!(!reporter.has_errors());
        reporter.report_error(Path::new("test.txt"), &TranslateError::Parse("test error".to_string()));
        assert!(reporter.has_errors());
    }

    #[test]
    fn test_default_reporter_get_progress() {
        let reporter = DefaultReporter::new();
        assert_eq!(reporter.get_progress(), 0.0);
    }

    #[test]
    fn test_default_reporter_finalize() {
        let reporter = DefaultReporter::new();
        reporter.finalize();
        let stats = reporter.get_stats();
        assert!(stats.end_time.is_some());
    }

    #[test]
    fn test_default_reporter_save_report() {
        let reporter = DefaultReporter::new();
        reporter.finalize();
        let temp_dir = std::env::temp_dir();
        let report_path = temp_dir.join("test_report.txt");

        let result = reporter.save_report(&report_path, ReportFormat::Text);
        assert!(result.is_ok());
        assert!(report_path.exists());

        std::fs::remove_file(&report_path).expect("Failed to remove test file");
    }

    #[test]
    fn test_default_reporter_save_report_with_template() {
        let reporter = DefaultReporter::new();
        reporter.finalize();
        let temp_dir = std::env::temp_dir();

        let result = reporter.save_report_with_template(
            &temp_dir,
            "report_{timestamp}.{format}",
            ReportFormat::Text,
        );
        assert!(result.is_ok());

        let report_path = result.expect("Failed to get report path");
        assert!(report_path.exists());

        std::fs::remove_file(&report_path).expect("Failed to remove test file");
    }

    #[test]
    fn test_create_reporter() {
        let reporter = create_reporter();
        let stats = reporter.get_stats();
        assert_eq!(stats.total_files, 0);
    }
}
