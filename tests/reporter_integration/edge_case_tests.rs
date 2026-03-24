use std::fs;
use tempfile::TempDir;

use codebase_translate::{
    core::error::Result,
    reporter::{create_reporter, ReportFormat, TranslationStats},
};

#[test]
fn test_reporter_empty_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 0;
    stats.processed_files = 0;
    stats.total_units = 0;
    stats.translated_units = 0;
    stats.api_call_count = 0;
    stats.cache_miss_count = 0;

    stats.finalize();
    reporter.finalize(&stats);

    let report_path = temp_dir_path.join("empty_directory_report.txt");
    reporter.save_report(&report_path, &stats, ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
    assert!(
        report_content.contains("Total:      0"),
        "Should show 0 files"
    );
    assert!(
        report_content.contains("Processed:  0"),
        "Should show 0 processed files"
    );
    assert!(
        report_content.contains("Total:      0"),
        "Should show 0 units"
    );

    Ok(())
}
