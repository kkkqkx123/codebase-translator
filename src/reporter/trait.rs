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

/// Reporter trait for progress and statistics
///
/// Note: This trait is intentionally synchronous to avoid async complexity
/// in the reporting layer. All methods should return immediately.
pub trait Reporter: Send + Sync {
    /// Report file processed
    fn report_file(&self, path: &Path, units: usize);

    /// Report translation progress
    fn report_progress(&self, current: usize, total: usize);

    /// Report error
    fn report_error(&self, error: &TranslateError);

    /// Report skipped file
    fn report_skipped(&self, path: &Path);

    /// Report API call
    fn report_api_call(&self, count: usize);

    /// Report cache hit
    fn report_cache_hit(&self);

    /// Report cache miss
    fn report_cache_miss(&self);

    /// Get final report in specified format
    fn final_report(&self, format: ReportFormat) -> Result<String, TranslateError>;

    /// Get statistics
    fn get_stats(&self) -> TranslationStats;

    /// Check if there are any errors
    fn has_errors(&self) -> bool;

    /// Get progress percentage
    fn get_progress(&self) -> f64;

    /// Finalize the reporter
    fn finalize(&self);

    /// Save report to file
    fn save_report(&self, path: &Path, format: ReportFormat) -> Result<(), TranslateError>;

    /// Save report with filename template
    fn save_report_with_template(
        &self,
        dir: &Path,
        template: &str,
        format: ReportFormat,
    ) -> Result<std::path::PathBuf, TranslateError>;
}
