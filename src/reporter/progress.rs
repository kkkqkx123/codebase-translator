//! Progress reporter with real-time progress bar

use std::path::Path;
use std::sync::Arc;

use crate::core::error::TranslateError;
use crate::reporter::r#trait::{ReportFormat, Reporter};
use crate::reporter::stats::{SharedStats, TranslationStats};
use tracing::{debug, info};

#[cfg(feature = "progress")]
use indicatif::{ProgressBar, ProgressStyle};

/// Progress reporter with real-time progress bar
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    stats: SharedStats,
    #[cfg(feature = "progress")]
    progress_bar: Option<ProgressBar>,
    current_file: Arc<std::sync::RwLock<Option<String>>>,
}

impl ProgressReporter {
    pub fn new() -> Self {
        info!("Creating progress reporter");
        #[cfg(feature = "progress")]
        let progress_bar = {
            let bar = ProgressBar::new(0);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .expect("Failed to set progress bar style")
                    .progress_chars("#>-"),
            );
            Some(bar)
        };

        Self {
            stats: SharedStats::new(),
            #[cfg(feature = "progress")]
            progress_bar,
            current_file: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    #[cfg(feature = "progress")]
    fn update_progress_bar(&self) {
        if let Some(bar) = &self.progress_bar {
            let stats = self.stats.get_stats();
            bar.set_length(stats.total_files as u64);
            bar.set_position(stats.processed_files as u64);

            let elapsed = if let Some(end_time) = stats.end_time {
                end_time
                    .signed_duration_since(stats.start_time)
                    .num_milliseconds() as f64
                    / 1000.0
            } else {
                chrono::Utc::now()
                    .signed_duration_since(stats.start_time)
                    .num_milliseconds() as f64
                    / 1000.0
            };
            let speed = if elapsed > 0.0 {
                stats.processed_files as f64 / elapsed
            } else {
                0.0
            };

            let current_file = self
                .current_file
                .read()
                .expect("Failed to read current file");
            bar.set_message(format!(
                "{} | {:.1} files/s",
                current_file.as_deref().unwrap_or("Initializing..."),
                speed
            ));

            debug!(
                processed = stats.processed_files,
                total = stats.total_files,
                speed = speed,
                "Progress bar updated"
            );
        }
    }

    fn set_current_file(&self, path: &Path) {
        if let Ok(mut current) = self.current_file.write() {
            *current = Some(path.display().to_string());
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for ProgressReporter {
    fn report_file(&self, path: &Path, units: usize) {
        debug!(
            file = %path.display(),
            units = units,
            "Reporting file processed"
        );
        self.set_current_file(path);
        self.stats.record_units(units);
        self.stats.record_processed();
        #[cfg(feature = "progress")]
        self.update_progress_bar();
    }

    fn report_progress(&self, _current: usize, total: usize) {
        debug!(current = _current, total = total, "Reporting progress");
        self.stats.record_total_files(total);
        #[cfg(feature = "progress")]
        self.update_progress_bar();
    }

    fn report_error(&self, error: &TranslateError) {
        debug!(
            error = %error,
            "Reporting error"
        );
        self.stats.record_failed("unknown", &error.to_string());
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
        #[cfg(feature = "progress")]
        if let Some(bar) = &self.progress_bar {
            bar.finish();
        }

        info!("Generating final report");
        let stats = self.stats.get_stats();

        match format {
            ReportFormat::Text => self.generate_text_report(&stats),
            ReportFormat::Json => self.generate_json_report(&stats),
        }
    }

    fn get_stats(&self) -> TranslationStats {
        self.stats.get_stats()
    }

    fn has_errors(&self) -> bool {
        self.stats.has_errors()
    }

    fn get_progress(&self) -> f64 {
        self.stats.get_progress()
    }

    fn finalize(&self) {
        info!("Finalizing progress reporter statistics");
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

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
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

impl ProgressReporter {
    fn generate_text_report(&self, stats: &TranslationStats) -> Result<String, TranslateError> {
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
                if i >= 10 {
                    report.push_str(&format!(
                        "  ... and {} more errors\n",
                        stats.error_count - 10
                    ));
                    break;
                }
                report.push_str(&format!("  {}. {}: {}\n", i + 1, err.file_path, err.error));
            }
            report.push('\n');
        }

        report.push_str(&format!("{}\n", "=".repeat(60)));

        Ok(report)
    }

    fn generate_json_report(&self, stats: &TranslationStats) -> Result<String, TranslateError> {
        serde_json::to_string_pretty(stats)
            .map_err(|e| TranslateError::Parse(format!("Failed to serialize JSON report: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_new() {
        let reporter = ProgressReporter::new();
        let stats = reporter.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.processed_files, 0);
    }

    #[test]
    fn test_progress_reporter_default() {
        let reporter = ProgressReporter::default();
        let stats = reporter.get_stats();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.processed_files, 0);
    }

    #[test]
    fn test_progress_reporter_report_file() {
        let reporter = ProgressReporter::new();
        reporter.report_file(Path::new("test.rs"), 5);
        let stats = reporter.get_stats();
        assert_eq!(stats.processed_files, 1);
        assert_eq!(stats.total_units, 5);
    }

    #[test]
    fn test_progress_reporter_report_skipped() {
        let reporter = ProgressReporter::new();
        reporter.report_skipped(Path::new("test.rs"));
        let stats = reporter.get_stats();
        assert_eq!(stats.skipped_files, 1);
    }

    #[test]
    fn test_progress_reporter_report_api_call() {
        let reporter = ProgressReporter::new();
        reporter.report_api_call(5);
        let stats = reporter.get_stats();
        assert_eq!(stats.api_call_count, 5);
    }

    #[test]
    fn test_progress_reporter_report_cache_hit() {
        let reporter = ProgressReporter::new();
        reporter.report_cache_hit();
        let stats = reporter.get_stats();
        assert_eq!(stats.cache_hit_count, 1);
    }

    #[test]
    fn test_progress_reporter_report_cache_miss() {
        let reporter = ProgressReporter::new();
        reporter.report_cache_miss();
        let stats = reporter.get_stats();
        assert_eq!(stats.cache_miss_count, 1);
    }

    #[test]
    fn test_progress_reporter_has_errors() {
        let reporter = ProgressReporter::new();
        assert!(!reporter.has_errors());
        reporter.report_error(&TranslateError::Parse("test error".to_string()));
        assert!(reporter.has_errors());
    }

    #[test]
    fn test_progress_reporter_get_progress() {
        let reporter = ProgressReporter::new();
        assert_eq!(reporter.get_progress(), 0.0);
    }

    #[test]
    fn test_progress_reporter_finalize() {
        let reporter = ProgressReporter::new();
        reporter.finalize();
        let stats = reporter.get_stats();
        assert!(stats.end_time.is_some());
    }

    #[test]
    fn test_progress_reporter_save_report() {
        let reporter = ProgressReporter::new();
        reporter.finalize();
        let temp_dir = std::env::temp_dir();
        let report_path = temp_dir.join("test_report.txt");

        let result = reporter.save_report(&report_path, ReportFormat::Text);
        assert!(result.is_ok());
        assert!(report_path.exists());

        std::fs::remove_file(&report_path).expect("Failed to remove test file");
    }

    #[test]
    fn test_progress_reporter_save_report_with_template() {
        let reporter = ProgressReporter::new();
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
}
