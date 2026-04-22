//! Output Integration Tests
//!
//! Tests for OutputFormatter component that formats verification results.

use std::path::PathBuf;

use codebase_translate::commands::verify::{
    OutputFormat, OutputFormatter, VerifyMatch, VerifySummary,
};
use codebase_translate::core::models::PatternType;
use codebase_translate::core::models::Position;

fn create_test_match(
    file_path: &str,
    pattern_name: &str,
    pattern_type: PatternType,
    category: &str,
    extracted_text: &str,
    line: usize,
    col: usize,
) -> VerifyMatch {
    VerifyMatch {
        file_path: PathBuf::from(file_path),
        pattern_name: pattern_name.to_string(),
        pattern_type,
        category: category.to_string(),
        extracted_text: extracted_text.to_string(),
        position: Position::new(line, col, 0),
        raw_match: None,
        metadata: Default::default(),
    }
}

fn create_test_summary(total_files: usize, total_matches: usize) -> VerifySummary {
    VerifySummary {
        total_files,
        total_matches,
        patterns_used: vec![("comment".to_string(), 1), ("error".to_string(), 1)]
            .into_iter()
            .collect(),
        by_category: vec![("other".to_string(), 1), ("error_handling".to_string(), 1)]
            .into_iter()
            .collect(),
        by_file_type: vec![("rs".to_string(), 1), ("js".to_string(), 1)]
            .into_iter()
            .collect(),
        by_pattern_type: vec![("Builtin".to_string(), 1), ("CustomRegex".to_string(), 1)]
            .into_iter()
            .collect(),
    }
}

#[test]
fn test_output_formatter_table_format() {
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
    ];
    let summary = create_test_summary(2, 2);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, true, true)
        .expect("Failed to format output");

    assert!(output.contains("test.rs"));
    assert!(output.contains("test.js"));
    assert!(output.contains("comment"));
    assert!(output.contains("error"));
    assert!(output.contains("Builtin"));
    assert!(output.contains("CustomRegex"));
}

#[test]
fn test_output_formatter_json_format() {
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        "Text",
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Json, false, false)
        .expect("Failed to format output");

    assert!(output.contains("\"file_path\""));
    assert!(output.contains("\"pattern_name\""));
    assert!(output.contains("\"pattern_type\""));
    assert!(output.contains("\"category\""));
    assert!(output.contains("\"extracted_text\""));
}

#[test]
fn test_output_formatter_csv_format() {
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
    ];
    let summary = create_test_summary(2, 2);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Csv, false, false)
        .expect("Failed to format output");

    assert!(output.contains("pattern,type,category,file,line,extracted_text"));
    assert!(output.contains("test.rs"));
    assert!(output.contains("test.js"));
    assert!(output.contains("comment"));
    assert!(output.contains("error"));
}

#[test]
fn test_output_formatter_detailed_mode() {
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        "Text",
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let detailed_output =
        OutputFormatter::format(&matches, &summary, OutputFormat::Table, true, false)
            .expect("Failed to format output");
    let simple_output =
        OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
            .expect("Failed to format output");

    assert!(detailed_output.len() > simple_output.len());
}

#[test]
fn test_output_formatter_with_stats() {
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        "Text",
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let output_with_stats =
        OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, true)
            .expect("Failed to format output");
    let output_without_stats =
        OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
            .expect("Failed to format output");

    assert!(output_with_stats.contains("Summary"));
    assert!(!output_without_stats.contains("Statistics"));
}

#[test]
fn test_output_formatter_empty_matches() {
    let matches: Vec<VerifyMatch> = vec![];
    let summary = VerifySummary::new();

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
        .expect("Failed to format output");

    assert!(output.contains("Pattern") || output.contains("Type") || output.contains("Category"));
}

#[test]
fn test_output_formatter_multiple_matches() {
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "log",
            PatternType::StateMachine,
            "output",
            "Log",
            3,
            4,
        ),
    ];
    let summary = VerifySummary {
        total_files: 3,
        total_matches: 3,
        patterns_used: vec![
            ("comment".to_string(), 1),
            ("error".to_string(), 1),
            ("log".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_category: vec![
            ("other".to_string(), 1),
            ("error_handling".to_string(), 1),
            ("output".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_file_type: vec![
            ("rs".to_string(), 1),
            ("js".to_string(), 1),
            ("py".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_pattern_type: vec![
            ("Builtin".to_string(), 1),
            ("CustomRegex".to_string(), 1),
            ("StateMachine".to_string(), 1),
        ]
        .into_iter()
        .collect(),
    };

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
        .expect("Failed to format output");

    assert!(output.contains("test.rs"));
    assert!(output.contains("test.js"));
    assert!(output.contains("test.py"));
}

#[test]
fn test_output_formatter_pattern_types() {
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "log",
            PatternType::StateMachine,
            "output",
            "Log",
            3,
            4,
        ),
    ];
    let summary = VerifySummary {
        total_files: 3,
        total_matches: 3,
        patterns_used: vec![
            ("comment".to_string(), 1),
            ("error".to_string(), 1),
            ("log".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_category: vec![
            ("other".to_string(), 1),
            ("error_handling".to_string(), 1),
            ("output".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_file_type: vec![
            ("rs".to_string(), 1),
            ("js".to_string(), 1),
            ("py".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_pattern_type: vec![
            ("Builtin".to_string(), 1),
            ("CustomRegex".to_string(), 1),
            ("StateMachine".to_string(), 1),
        ]
        .into_iter()
        .collect(),
    };

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, true)
        .expect("Failed to format output");

    assert!(output.contains("Builtin"));
    assert!(output.contains("CustomRegex"));
    assert!(output.contains("StateMachine"));
}

#[test]
fn test_output_formatter_categories() {
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "log",
            PatternType::StateMachine,
            "output",
            "Log",
            3,
            4,
        ),
    ];
    let summary = VerifySummary {
        total_files: 3,
        total_matches: 3,
        patterns_used: vec![
            ("comment".to_string(), 1),
            ("error".to_string(), 1),
            ("log".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_category: vec![
            ("other".to_string(), 1),
            ("error_handling".to_string(), 1),
            ("output".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_file_type: vec![
            ("rs".to_string(), 1),
            ("js".to_string(), 1),
            ("py".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_pattern_type: vec![
            ("Builtin".to_string(), 1),
            ("CustomRegex".to_string(), 1),
            ("StateMachine".to_string(), 1),
        ]
        .into_iter()
        .collect(),
    };

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, true)
        .expect("Failed to format output");

    assert!(output.contains("other"));
    assert!(output.contains("error_handling"));
    assert!(output.contains("output"));
}

#[test]
fn test_output_formatter_file_extensions() {
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "log",
            PatternType::StateMachine,
            "output",
            "Log",
            3,
            4,
        ),
    ];
    let summary = VerifySummary {
        total_files: 3,
        total_matches: 3,
        patterns_used: vec![
            ("comment".to_string(), 1),
            ("error".to_string(), 1),
            ("log".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_category: vec![
            ("other".to_string(), 1),
            ("error_handling".to_string(), 1),
            ("output".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_file_type: vec![
            ("rs".to_string(), 1),
            ("js".to_string(), 1),
            ("py".to_string(), 1),
        ]
        .into_iter()
        .collect(),
        by_pattern_type: vec![
            ("Builtin".to_string(), 1),
            ("CustomRegex".to_string(), 1),
            ("StateMachine".to_string(), 1),
        ]
        .into_iter()
        .collect(),
    };

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, true)
        .expect("Failed to format output");

    assert!(output.contains("rs"));
    assert!(output.contains("js"));
    assert!(output.contains("py"));
}

#[test]
fn test_output_formatter_position_info() {
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        "Text",
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, true, false)
        .expect("Failed to format output");

    assert!(output.contains("1"));
}

#[test]
fn test_output_formatter_long_text() {
    let long_text = "This is a very long text that should be properly formatted in output without breaking layout";
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        long_text,
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
        .expect("Failed to format output");

    assert!(output.contains("This is a very long text"));
}

#[test]
fn test_output_formatter_special_characters() {
    let special_text = "Text with special characters: < > & \" ' \\ /";
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        special_text,
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Json, false, false)
        .expect("Failed to format output");

    assert!(output.contains("Text with special characters"));
}

#[test]
fn test_output_formatter_unicode() {
    let unicode_text = "Unicode text: 你好世界 🌍";
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        unicode_text,
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Json, false, false)
        .expect("Failed to format output");

    assert!(output.contains("你好世界"));
    assert!(output.contains("🌍"));
}

#[test]
fn test_output_formatter_chinese_truncation() {
    // Test that Chinese text (multi-byte UTF-8 characters) is properly truncated
    // without causing "byte index is not a char boundary" panic
    let chinese_text = "这是一个很长的中文文本，用于测试截断功能是否能正确处理多字节字符";
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        chinese_text,
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    // Test table format with truncation (max_len is 60 for non-detailed mode)
    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
        .expect("Failed to format output");

    // Should contain the beginning of the Chinese text
    assert!(output.contains("这是一个"));
    // Should not panic - this is the main test
}

#[test]
fn test_output_formatter_chinese_truncation_detailed() {
    // Test Chinese text truncation in detailed mode (max_len is 40)
    let chinese_text = "TomlParserManager - 管理 TOML 解析器的生命周期，提供预加载模式";
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        chinese_text,
        1,
        4,
    )];
    let summary = create_test_summary(1, 1);

    // Test table format with truncation in detailed mode
    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, true, false)
        .expect("Failed to format output");

    // Should contain the beginning of the text
    assert!(output.contains("TomlParserManager"));
    // Should not panic - this is the main test
}
