use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use codebase_translate::{
    core::error::Result,
    reporter::{create_reporter, ReportFormat, TranslationStats},
};

#[test]
fn test_reporter_records_file_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 2;
    stats.processed_files = 2;
    stats.total_units = 8;
    stats.translated_units = 8;
    stats.api_call_count = 2;
    stats.cache_miss_count = 2;

    stats.record_translator_call("deeplx", 150, true, 100);
    stats.record_translator_call("tencent", 200, true, 120);

    stats.record_llm_provider_call("openai-gpt4", "openai", "gpt-4", 300, true, 100);

    stats.finalize();
    reporter.finalize(&stats);

    let report_path = temp_dir_path.join("file_processing_report.txt");
    reporter.save_report(&report_path, &stats, ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
    assert!(
        report_content.contains("Processed:  2"),
        "Should record processed files"
    );
    assert!(
        report_content.contains("Total:      8"),
        "Should record total units"
    );
    assert!(
        report_content.contains("Translator Statistics:"),
        "Report should contain translator statistics section"
    );
    assert!(
        report_content.contains("deeplx"),
        "Report should contain deeplx translator"
    );
    assert!(
        report_content.contains("tencent"),
        "Report should contain tencent translator"
    );
    assert!(
        report_content.contains("LLM Provider Statistics:"),
        "Report should contain LLM provider statistics section"
    );
    assert!(
        report_content.contains("openai-gpt4"),
        "Report should contain openai-gpt4 provider"
    );

    Ok(())
}
