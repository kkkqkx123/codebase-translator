use std::fs;
use tempfile::TempDir;

use codebase_translate::{
    core::error::Result,
    reporter::{create_reporter, ReportFormat, TranslationStats},
};

#[test]
fn test_reporter_finalization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 8;
    stats.processed_files = 8;
    stats.total_units = 40;
    stats.translated_units = 40;
    stats.api_call_count = 8;
    stats.cache_miss_count = 8;

    stats.record_translator_call("deeplx", 150, true, 100);
    stats.record_translator_call("deeplx", 180, true, 120);
    stats.record_translator_call("tencent", 200, true, 150);

    stats.record_llm_provider_call("openai-gpt4", "openai", "gpt-4", 300, true, 100);
    stats.record_llm_provider_call(
        "anthropic-claude3",
        "anthropic",
        "claude-3-opus",
        350,
        true,
        120,
    );

    stats.finalize();
    reporter.finalize(&stats);

    assert!(
        stats.end_time.is_some(),
        "Should have end time after finalization"
    );

    let report_path = temp_dir_path.join("finalization_report.txt");
    reporter.save_report(&report_path, &stats, ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
    assert!(
        report_content.contains("Start:"),
        "Report should contain start time"
    );
    assert!(
        report_content.contains("End:"),
        "Report should contain end time"
    );
    assert!(
        report_content.contains("Duration:"),
        "Report should contain duration"
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
    assert!(
        report_content.contains("anthropic-claude3"),
        "Report should contain anthropic-claude3 provider"
    );

    Ok(())
}
