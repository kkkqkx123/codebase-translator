//! Collector Integration Tests
//!
//! Tests for the MatchCollector component that extracts verification matches from translation units.

use std::path::PathBuf;

use codebase_translate::commands::verify::{MatchCollector, VerifyMatch};
use codebase_translate::core::models::{
    NodeType, PatternType as CorePatternType, PatternType, Position, TranslationUnit,
};

fn create_test_unit(
    id: &str,
    node_type: NodeType,
    content: &str,
    start_line: usize,
    start_col: usize,
    end_col: usize,
) -> TranslationUnit {
    TranslationUnit {
        id: id.to_string(),
        node_type,
        content: content.to_string(),
        start_pos: Position::new(start_line, start_col, 0),
        end_pos: Position::new(start_line, end_col, 0),
        language: None,
        should_translate: true,
        translated: None,
        format_info: None,
        pattern_type: None,
        pattern_name: None,
    }
}

fn create_test_unit_with_pattern(
    id: &str,
    node_type: NodeType,
    content: &str,
    start_line: usize,
    start_col: usize,
    end_col: usize,
    pattern_type: CorePatternType,
    pattern_name: &str,
) -> TranslationUnit {
    TranslationUnit {
        id: id.to_string(),
        node_type,
        content: content.to_string(),
        start_pos: Position::new(start_line, start_col, 0),
        end_pos: Position::new(start_line, end_col, 0),
        language: None,
        should_translate: true,
        translated: None,
        format_info: None,
        pattern_type: Some(pattern_type),
        pattern_name: Some(pattern_name.to_string()),
    }
}

#[test]
fn test_collector_basic_comment() {
    let file_path = PathBuf::from("test.rs");
    let content = "// This is a comment\nfn main() {}";
    let units = vec![create_test_unit(
        "1",
        NodeType::Comment,
        "This is a comment",
        1,
        4,
        21,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "comment");
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[0].extracted_text, "This is a comment");
    assert_eq!(matches[0].category, "other");
}

#[test]
fn test_collector_docstring() {
    let file_path = PathBuf::from("test.py");
    let content = "\"\"\"This is a docstring\"\"\"\nprint('hello')";
    let units = vec![create_test_unit(
        "1",
        NodeType::DocString,
        "This is a docstring",
        1,
        4,
        21,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "docstring");
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[0].extracted_text, "This is a docstring");
    assert_eq!(matches[0].category, "other");
}

#[test]
fn test_collector_error_message() {
    let file_path = PathBuf::from("test.js");
    let content = "throw new Error('Invalid input');";
    let units = vec![create_test_unit(
        "1",
        NodeType::ErrorMessage,
        "Invalid input",
        1,
        17,
        30,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "error_message");
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[0].extracted_text, "Invalid input");
    assert_eq!(matches[0].category, "error_handling");
}

#[test]
fn test_collector_log_message() {
    let file_path = PathBuf::from("test.ts");
    let content = "console.log('Processing data');";
    let units = vec![create_test_unit(
        "1",
        NodeType::LogMessage,
        "Processing data",
        1,
        13,
        28,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "log_message");
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[0].extracted_text, "Processing data");
    assert_eq!(matches[0].category, "output");
}

#[test]
fn test_collector_format_string() {
    let file_path = PathBuf::from("test.py");
    let content = "f'Hello, {name}!'";
    let units = vec![create_test_unit(
        "1",
        NodeType::FormatString,
        "Hello, {name}!",
        1,
        3,
        18,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "format_string");
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[0].extracted_text, "Hello, {name}!");
    assert_eq!(matches[0].category, "output");
}

#[test]
fn test_collector_string_literal() {
    let file_path = PathBuf::from("test.rs");
    let content = "let x = \"Hello, world!\";";
    let units = vec![create_test_unit(
        "1",
        NodeType::StringLiteral,
        "Hello, world!",
        1,
        9,
        23,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "string_literal");
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[0].extracted_text, "Hello, world!");
    assert_eq!(matches[0].category, "variables");
}

#[test]
fn test_collector_custom_regex_pattern() {
    let file_path = PathBuf::from("test.js");
    let content = "Error: 'File not found'";
    let units = vec![create_test_unit_with_pattern(
        "test.js_cp_error_pattern_0",
        NodeType::StringLiteral,
        "File not found",
        1,
        8,
        22,
        CorePatternType::CustomRegex,
        "error_pattern",
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "error_pattern");
    assert_eq!(matches[0].pattern_type, PatternType::CustomRegex);
    assert_eq!(matches[0].extracted_text, "File not found");
    assert_eq!(matches[0].category, "variables");
}

#[test]
fn test_collector_state_machine_pattern() {
    let file_path = PathBuf::from("test.ts");
    let content = "t('welcome', 'Hello World')";
    let units = vec![create_test_unit_with_pattern(
        "test.ts_sm_i18n_pattern_0",
        NodeType::StringLiteral,
        "Hello World",
        1,
        17,
        28,
        CorePatternType::StateMachine,
        "i18n_pattern",
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "i18n_pattern");
    assert_eq!(matches[0].pattern_type, PatternType::StateMachine);
    assert_eq!(matches[0].extracted_text, "Hello World");
    assert_eq!(matches[0].category, "variables");
}

#[test]
fn test_collector_multiple_units() {
    let file_path = PathBuf::from("test.rs");
    let content = "// Comment\nlet x = \"text\";";
    let units = vec![
        create_test_unit("1", NodeType::Comment, "Comment", 1, 4, 11),
        create_test_unit("2", NodeType::StringLiteral, "text", 2, 9, 13),
    ];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].pattern_name, "comment");
    assert_eq!(matches[1].pattern_name, "string_literal");
}

#[test]
fn test_collector_mixed_pattern_types() {
    let file_path = PathBuf::from("test.js");
    let content = "// Comment\nError: 'error'\nt('key', 'value')";
    let units = vec![
        create_test_unit("1", NodeType::Comment, "Comment", 1, 4, 11),
        create_test_unit_with_pattern(
            "test.js_cp_error_0",
            NodeType::StringLiteral,
            "error",
            2,
            8,
            14,
            CorePatternType::CustomRegex,
            "error_pattern",
        ),
        create_test_unit_with_pattern(
            "test.js_sm_i18n_0",
            NodeType::StringLiteral,
            "value",
            3,
            10,
            15,
            CorePatternType::StateMachine,
            "i18n_pattern",
        ),
    ];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].pattern_type, PatternType::Builtin);
    assert_eq!(matches[1].pattern_type, PatternType::CustomRegex);
    assert_eq!(matches[2].pattern_type, PatternType::StateMachine);
}

#[test]
fn test_collector_raw_match_extraction() {
    let file_path = PathBuf::from("test.rs");
    let content = "    // This is a comment\n";
    let units = vec![create_test_unit(
        "1",
        NodeType::Comment,
        "This is a comment",
        1,
        8,
        26,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(
        matches[0].raw_match,
        Some("    // This is a comment".to_string())
    );
}

#[test]
fn test_collector_position_tracking() {
    let file_path = PathBuf::from("test.rs");
    let content = "// Comment\nfn main() {}";
    let units = vec![create_test_unit(
        "1",
        NodeType::Comment,
        "Comment",
        1,
        4,
        11,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].position.line, 1);
    assert_eq!(matches[0].position.column, 4);
    assert_eq!(matches[0].position.offset, 0);
}

#[test]
fn test_collector_empty_units() {
    let file_path = PathBuf::from("test.rs");
    let content = "// Comment";
    let units: Vec<TranslationUnit> = vec![];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 0);
}

#[test]
fn test_collector_file_path_tracking() {
    let file_path = PathBuf::from("src/main.rs");
    let content = "// Comment";
    let units = vec![create_test_unit(
        "1",
        NodeType::Comment,
        "Comment",
        1,
        4,
        11,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].file_path, PathBuf::from("src/main.rs"));
}

#[test]
fn test_collector_metadata_initialization() {
    let file_path = PathBuf::from("test.rs");
    let content = "// Comment";
    let units = vec![create_test_unit(
        "1",
        NodeType::Comment,
        "Comment",
        1,
        4,
        11,
    )];

    let matches = MatchCollector::collect_from_units(file_path, units, content);

    assert_eq!(matches.len(), 1);
    assert!(matches[0].metadata.is_empty());
}

#[test]
fn test_collector_verify_match_structure() {
    let file_path = PathBuf::from("test.rs");
    let content = "// Comment";
    let units = vec![create_test_unit(
        "1",
        NodeType::Comment,
        "Comment",
        1,
        4,
        11,
    )];

    let matches = MatchCollector::collect_from_units(file_path.clone(), units, content);

    let match_item = &matches[0];
    assert_eq!(match_item.file_path, file_path);
    assert!(!match_item.pattern_name.is_empty());
    assert!(!match_item.extracted_text.is_empty());
    assert_eq!(match_item.position.line, 1);
}
