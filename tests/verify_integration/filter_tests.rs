//! Filter Integration Tests
//!
//! Tests for MatchFilter component that filters verification matches.

use std::path::PathBuf;

use codebase_translate::commands::verify::{FilterOptions, MatchFilter, VerifyMatch};
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
fn test_filter_empty_options() {
    let options = FilterOptions::new();
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

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_filter_by_pattern_name() {
    let options = FilterOptions::new().with_pattern_name("comment".to_string());
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
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].pattern_name, "comment");
}

#[test]
fn test_filter_by_pattern_name_partial() {
    let options = FilterOptions::new().with_pattern_name("error".to_string());
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
            "error_pattern",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].pattern_name, "error_pattern");
}

#[test]
fn test_filter_by_extension() {
    let options = FilterOptions::new().with_extension("rs".to_string());
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
            "main.rs",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 2);
    assert!(filtered
        .iter()
        .all(|m| m.file_path.extension().unwrap() == "rs"));
}

#[test]
fn test_filter_by_extension_case_insensitive() {
    let options = FilterOptions::new().with_extension("JS".to_string());
    let matches = vec![
        create_test_match(
            "test.js",
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
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].file_path.extension().unwrap(), "js");
}

#[test]
fn test_filter_by_category() {
    let options = FilterOptions::new().with_category("error_handling".to_string());
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
            PatternType::Builtin,
            "output",
            "Log",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].category, "error_handling");
}

#[test]
fn test_filter_by_search_text() {
    let options = FilterOptions::new().with_search_text("Error".to_string());
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "This is text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error message",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "log",
            PatternType::Builtin,
            "output",
            "Log message",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].extracted_text.contains("Error"));
}

#[test]
fn test_filter_by_search_text_partial() {
    let options = FilterOptions::new().with_search_text("text".to_string());
    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "This is text",
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
            PatternType::Builtin,
            "output",
            "Another text",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|m| m.extracted_text.contains("text")));
}

#[test]
fn test_filter_multiple_criteria() {
    let options = FilterOptions::new()
        .with_extension("rs".to_string())
        .with_category("other".to_string());

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
            "main.rs",
            "log",
            PatternType::Builtin,
            "output",
            "Log",
            3,
            4,
        ),
        create_test_match(
            "lib.rs",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            4,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 2);
    assert!(filtered
        .iter()
        .all(|m| { m.file_path.extension().unwrap() == "rs" && m.category == "other" }));
}

#[test]
fn test_filter_all_criteria() {
    let options = FilterOptions::new()
        .with_pattern_name("comment".to_string())
        .with_extension("rs".to_string())
        .with_category("other".to_string())
        .with_search_text("text".to_string());

    let matches = vec![
        create_test_match(
            "test.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "This is text",
            1,
            4,
        ),
        create_test_match(
            "test.js",
            "comment",
            PatternType::Builtin,
            "other",
            "Text",
            2,
            8,
        ),
        create_test_match(
            "main.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Not match",
            3,
            4,
        ),
        create_test_match(
            "lib.rs",
            "docstring",
            PatternType::Builtin,
            "other",
            "text",
            4,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].pattern_name, "comment");
    assert_eq!(filtered[0].file_path.extension().unwrap(), "rs");
    assert_eq!(filtered[0].category, "other");
    assert!(filtered[0].extracted_text.contains("text"));
}

#[test]
fn test_filter_no_matches() {
    let options = FilterOptions::new()
        .with_pattern_name("nonexistent".to_string())
        .with_extension("xyz".to_string());

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

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_filter_empty_matches() {
    let options = FilterOptions::new().with_pattern_name("comment".to_string());
    let matches: Vec<VerifyMatch> = vec![];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_filter_preserves_order() {
    let options = FilterOptions::new().with_extension("rs".to_string());
    let matches = vec![
        create_test_match(
            "a.rs",
            "comment",
            PatternType::Builtin,
            "other",
            "Text A",
            1,
            4,
        ),
        create_test_match(
            "b.js",
            "error",
            PatternType::CustomRegex,
            "error_handling",
            "Error",
            2,
            8,
        ),
        create_test_match("c.rs", "log", PatternType::Builtin, "output", "Log", 3, 4),
        create_test_match(
            "d.rs",
            "docstring",
            PatternType::Builtin,
            "other",
            "Doc",
            4,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 3);
    assert_eq!(filtered[0].file_path, PathBuf::from("a.rs"));
    assert_eq!(filtered[1].file_path, PathBuf::from("c.rs"));
    assert_eq!(filtered[2].file_path, PathBuf::from("d.rs"));
}

#[test]
fn test_filter_by_pattern_type() {
    let options = FilterOptions::new().with_pattern_name("custom".to_string());
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
            "custom_pattern",
            PatternType::CustomRegex,
            "other",
            "Custom",
            2,
            8,
        ),
        create_test_match(
            "test.py",
            "sm_pattern",
            PatternType::StateMachine,
            "other",
            "SM",
            3,
            4,
        ),
    ];

    let filtered = MatchFilter::filter(matches, &options);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].pattern_type, PatternType::CustomRegex);
}
