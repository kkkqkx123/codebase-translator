//! Statistics Integration Tests
//!
//! Tests for StatisticsGenerator component that generates verification summaries.

use std::path::PathBuf;

use codebase_translate::commands::verify::{StatisticsGenerator, VerifyMatch};
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

#[test]
fn test_stats_generator_empty_matches() {
    let matches: Vec<VerifyMatch> = vec![];
    let total_files = 0;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.total_matches, 0);
}

#[test]
fn test_stats_generator_single_match() {
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        "Text",
        1,
        4,
    )];
    let total_files = 1;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.total_matches, 1);
}

#[test]
fn test_stats_generator_multiple_matches() {
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
    let total_files = 3;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_files, 3);
    assert_eq!(summary.total_matches, 3);
}

#[test]
fn test_stats_generator_pattern_type_counts() {
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
        create_test_match(
            "test.rs",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            4,
            4,
        ),
        create_test_match(
            "test.js",
            "custom",
            PatternType::CustomRegex,
            "other",
            "Custom",
            5,
            4,
        ),
    ];
    let total_files = 3;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_matches, 5);

    let builtin_count = summary.by_pattern_type.get("Builtin").copied().unwrap_or(0);
    let custom_count = summary
        .by_pattern_type
        .get("CustomRegex")
        .copied()
        .unwrap_or(0);
    let sm_count = summary
        .by_pattern_type
        .get("StateMachine")
        .copied()
        .unwrap_or(0);

    assert_eq!(builtin_count, 2);
    assert_eq!(custom_count, 2);
    assert_eq!(sm_count, 1);
}

#[test]
fn test_stats_generator_category_counts() {
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
        create_test_match(
            "test.rs",
            "format",
            PatternType::Builtin,
            "output",
            "Format",
            4,
            4,
        ),
        create_test_match(
            "test.js",
            "string",
            PatternType::Builtin,
            "variables",
            "String",
            5,
            4,
        ),
    ];
    let total_files = 3;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_matches, 5);

    let other_count = summary.by_category.get("other").copied().unwrap_or(0);
    let error_count = summary
        .by_category
        .get("error_handling")
        .copied()
        .unwrap_or(0);
    let output_count = summary.by_category.get("output").copied().unwrap_or(0);
    let variables_count = summary.by_category.get("variables").copied().unwrap_or(0);

    assert_eq!(other_count, 1);
    assert_eq!(error_count, 1);
    assert_eq!(output_count, 2);
    assert_eq!(variables_count, 1);
}

#[test]
fn test_stats_generator_extension_counts() {
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
        create_test_match(
            "main.rs",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            4,
            4,
        ),
        create_test_match(
            "lib.js",
            "custom",
            PatternType::CustomRegex,
            "other",
            "Custom",
            5,
            4,
        ),
    ];
    let total_files = 3;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_matches, 5);

    let rs_count = summary.by_file_type.get("rs").copied().unwrap_or(0);
    let js_count = summary.by_file_type.get("js").copied().unwrap_or(0);
    let py_count = summary.by_file_type.get("py").copied().unwrap_or(0);

    assert_eq!(rs_count, 2);
    assert_eq!(js_count, 2);
    assert_eq!(py_count, 1);
}

#[test]
fn test_stats_generator_files_with_matches() {
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
            "test.rs",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.js",
            "log",
            PatternType::StateMachine,
            "output",
            "Log",
            3,
            4,
        ),
    ];
    let total_files = 2;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_files, 2);
}

#[test]
fn test_stats_generator_all_pattern_types() {
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
    let total_files = 3;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.by_pattern_type.len(), 3);
    assert!(summary.by_pattern_type.contains_key("Builtin"));
    assert!(summary.by_pattern_type.contains_key("CustomRegex"));
    assert!(summary.by_pattern_type.contains_key("StateMachine"));
}

#[test]
fn test_stats_generator_all_categories() {
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
        create_test_match(
            "test.rs",
            "string",
            PatternType::Builtin,
            "variables",
            "String",
            4,
            4,
        ),
    ];
    let total_files = 3;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert!(summary.by_category.contains_key("other"));
    assert!(summary.by_category.contains_key("error_handling"));
    assert!(summary.by_category.contains_key("output"));
    assert!(summary.by_category.contains_key("variables"));
}

#[test]
fn test_stats_generator_large_dataset() {
    let matches: Vec<VerifyMatch> = (0..100)
        .map(|i| {
            let pattern_type = match i % 3 {
                0 => PatternType::Builtin,
                1 => PatternType::CustomRegex,
                _ => PatternType::StateMachine,
            };
            create_test_match(
                &format!("test{}.rs", i),
                "pattern",
                pattern_type,
                "other",
                &format!("Text {}", i),
                1,
                4,
            )
        })
        .collect();
    let total_files = 50;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_files, 50);
    assert_eq!(summary.total_matches, 100);
}

#[test]
fn test_stats_generator_duplicate_files() {
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
            "test.rs",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.js",
            "log",
            PatternType::StateMachine,
            "output",
            "Log",
            3,
            4,
        ),
    ];
    let total_files = 2;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.total_files, 2);
}

#[test]
fn test_stats_generator_summary_structure() {
    let matches = vec![create_test_match(
        "test.rs",
        "comment",
        PatternType::Builtin,
        "other",
        "Text",
        1,
        4,
    )];
    let total_files = 1;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert!(!summary.by_pattern_type.is_empty());
    assert!(!summary.by_category.is_empty());
    assert!(!summary.by_file_type.is_empty());
}

#[test]
fn test_stats_generator_mixed_extensions() {
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
        create_test_match(
            "test.ts",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            4,
            4,
        ),
        create_test_match(
            "test.go",
            "string",
            PatternType::Builtin,
            "variables",
            "String",
            5,
            4,
        ),
    ];
    let total_files = 5;

    let summary = StatisticsGenerator::generate(&matches, total_files);

    assert_eq!(summary.by_file_type.len(), 5);
    assert_eq!(summary.total_files, 5);
}
