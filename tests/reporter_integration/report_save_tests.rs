use std::fs;
use tempfile::TempDir;

use codebase_translate::{
    core::error::Result,
    reporter::{create_reporter, ReportFormat, TranslationStats},
};

#[test]
fn test_reporter_save_report() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 1;
    stats.processed_files = 1;
    stats.total_units = 4;
    stats.translated_units = 4;
    stats.api_call_count = 1;
    stats.cache_miss_count = 1;

    stats.record_translator_call("deeplx", 150, true, 100);
    stats.record_llm_provider_call("openai-gpt4", "openai", "gpt-4", 200, true, 100);

    stats.finalize();
    reporter.finalize(&stats);

    let report_path = temp_dir_path.join("report.txt");
    reporter.save_report(&report_path, &stats, ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
    assert!(
        report_content.contains("Translation Report"),
        "Report should contain title"
    );
    assert!(
        report_content.contains("Files:"),
        "Report should contain files section"
    );
    assert!(
        report_content.contains("Translation Units:"),
        "Report should contain units section"
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
        report_content.contains("LLM Provider Statistics:"),
        "Report should contain LLM provider statistics section"
    );
    assert!(
        report_content.contains("openai-gpt4"),
        "Report should contain openai-gpt4 provider"
    );

    Ok(())
}

#[test]
fn test_reporter_save_report_with_template() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 1;
    stats.processed_files = 1;
    stats.total_units = 4;
    stats.translated_units = 4;
    stats.api_call_count = 1;
    stats.cache_miss_count = 1;

    stats.record_translator_call("deeplx", 150, true, 100);
    stats.record_llm_provider_call("openai-gpt4", "openai", "gpt-4", 200, true, 100);

    stats.finalize();
    reporter.finalize(&stats);

    let output_dir = temp_dir_path.join("reports");
    fs::create_dir_all(&output_dir)?;

    let saved_path = reporter.save_report_with_template(
        &output_dir,
        "translation_report_{timestamp}.txt",
        &stats,
        ReportFormat::Text,
    )?;

    assert!(saved_path.exists(), "Report file should be created");
    assert!(
        saved_path.starts_with(&output_dir),
        "Report should be in output directory"
    );

    let report_content = fs::read_to_string(&saved_path)?;
    assert!(
        report_content.contains("Translator Statistics:"),
        "Report should contain translator statistics section"
    );
    assert!(
        report_content.contains("LLM Provider Statistics:"),
        "Report should contain LLM provider statistics section"
    );

    Ok(())
}
