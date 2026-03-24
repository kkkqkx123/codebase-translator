use std::fs;
use tempfile::TempDir;

use codebase_translate::{
    core::error::Result,
    reporter::{create_reporter, ReportFormat, TranslationStats},
};

#[test]
fn test_reporter_progress_tracking() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 10;
    stats.processed_files = 5;
    stats.total_units = 50;
    stats.translated_units = 25;
    stats.api_call_count = 5;
    stats.cache_miss_count = 5;

    stats.record_translator_call("deeplx", 150, true, 100);
    stats.record_translator_call("tencent", 200, true, 120);

    stats.record_llm_provider_call("openai-gpt4", "openai", "gpt-4", 300, true, 100);

    stats.finalize();
    reporter.finalize(&stats);

    let progress = reporter.get_progress();
    assert!(progress >= 0.0, "Should have progress >= 0");
    assert!(progress <= 100.0, "Progress should be <= 100");

    let report_path = temp_dir_path.join("progress_tracking_report.txt");
    reporter.save_report(&report_path, &stats, ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
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
