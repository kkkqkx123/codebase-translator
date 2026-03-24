use std::fs;
use tempfile::TempDir;

use codebase_translate::{
    core::error::Result,
    reporter::{create_reporter, ReportFormat, TranslationStats},
};

#[test]
fn test_reporter_records_cache_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let reporter = create_reporter();

    let mut stats = TranslationStats::new();
    stats.total_files = 5;
    stats.processed_files = 5;
    stats.total_units = 20;
    stats.translated_units = 15;
    stats.api_call_count = 3;
    stats.cache_hit_count = 2;
    stats.cache_miss_count = 3;

    stats.record_translator_call("deeplx", 150, true, 100);
    stats.record_translator_call("deeplx", 180, true, 120);

    stats.record_llm_provider_call("openai-gpt4", "openai", "gpt-4", 300, true, 100);

    stats.finalize();
    reporter.finalize(&stats);

    let report_path = temp_dir_path.join("cache_operations_report.txt");
    reporter.save_report(&report_path, &stats, ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
    let total_cache_ops = stats.cache_hit_count + stats.cache_miss_count;
    assert!(
        total_cache_ops > 0,
        "Should record cache operations"
    );
    assert!(
        report_content.contains("Hits:       2"),
        "Should record cache hits"
    );
    assert!(
        report_content.contains("Misses:     3"),
        "Should record cache misses"
    );
    assert!(
        report_content.contains("Hit Rate:   40.0%"),
        "Should calculate correct hit rate"
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
