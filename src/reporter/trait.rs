//! Reporter trait definition
//!
//! This module defines the Reporter trait for progress and statistics reporting.

use std::path::Path;

use crate::core::error::TranslateError;
use crate::reporter::stats::TranslationStats;

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Text,
    Json,
}

/// Reporter trait for progress and statistics reporting
///
/// This trait provides methods for reporting progress and generating reports.
/// Statistics are collected externally and passed to the reporter for report generation.
pub trait Reporter: Send + Sync {
    /// Report total files to process (for progress tracking only)
    fn report_total_files(&self, count: usize);

    /// Report file processed (for progress tracking only, does not affect stats)
    fn report_file(&self, path: &Path, units: usize);

    /// Report translation progress (for progress tracking only)
    fn report_progress(&self, current: usize, total: usize);

    /// Report error (for logging only)
    fn report_error(&self, path: &Path, error: &TranslateError);

    /// Report skipped file (for logging only)
    fn report_skipped(&self, path: &Path);

    /// Report API call (for logging only)
    fn report_api_call(&self, count: usize);

    /// Report cache hit (for logging only)
    fn report_cache_hit(&self);

    /// Report cache miss (for logging only)
    fn report_cache_miss(&self);

    /// Report translator call statistics
    fn report_translator_call(
        &self,
        translator_type: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    );

    /// Report LLM provider call statistics
    fn report_llm_provider_call(
        &self,
        provider_id: &str,
        provider_name: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
        chars: usize,
    );

    /// Generate final report in specified format using the provided stats
    fn final_report(
        &self,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<String, TranslateError>;

    /// Get statistics (returns the stats passed to finalize)
    fn get_stats(&self) -> Option<TranslationStats>;

    /// Check if there are any errors
    fn has_errors(&self) -> bool;

    /// Get progress percentage
    fn get_progress(&self) -> f64;

    /// Finalize the reporter with the complete statistics
    fn finalize(&self, stats: &TranslationStats);

    /// Save report to file using the provided stats
    fn save_report(
        &self,
        path: &Path,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<(), TranslateError>;

    /// Save report with filename template using the provided stats
    fn save_report_with_template(
        &self,
        dir: &Path,
        template: &str,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<std::path::PathBuf, TranslateError>;
}
