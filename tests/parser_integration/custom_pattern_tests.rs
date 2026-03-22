//! Integration tests for custom patterns and state machines
//!
//! Tests that custom regex patterns and state machine patterns
//! are correctly applied to both Tree-sitter and Regex parsers.

use std::path::PathBuf;

use codebase_translate::config::project::{
    CustomRegexPattern, ExtractionConfig as ProjectExtractionConfig, ExtractionRule, PatternState,
    StateMachinePattern, StateTransition, StringLiteralCategory,
};
use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::core::traits::ExtractionConfig;
use codebase_translate::parser::filtering::{ContentFilter, FilterConfig};
use codebase_translate::parser::ParserConfig;
use std::sync::Arc;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

/// Create a filter that allows English content to be extracted
/// This is needed for tests that expect to extract English text
fn create_test_filter() -> Arc<ContentFilter> {
    let config = FilterConfig {
        source_langs: vec!["EN".to_string()],
        ..FilterConfig::default()
    };
    Arc::new(ContentFilter::new(config).expect("Failed to create filter"))
}

#[test]
fn test_custom_pattern_applied_to_tree_sitter_files() {
    let custom_patterns = vec![CustomRegexPattern {
        name: "todo_pattern".to_string(),
        file_extensions: vec!["py".to_string(), "js".to_string()],
        category: StringLiteralCategory::Other,
        regex: r#"TODO:\s*(.+)"#.to_string(),
        group: 1,
    }];

    let project_extraction_config = ProjectExtractionConfig {
        custom_patterns,
        ..Default::default()
    };

    let coordinator = ParserCoordinator::with_extraction_config(
        ParserConfig::default(),
        ExtractionConfig::default(),
        create_test_filter(),
        Some(project_extraction_config),
    )
    .unwrap();

    let py_content = r#"
def main():
    # TODO: Implement this function
    print("Hello")
    # TODO: Add error handling
"#;

    let file = create_test_file(py_content, "test.py");
    let units = coordinator.parse_file(&file).unwrap();

    let todo_units: Vec<_> = units
        .iter()
        .filter(|u| u.id.contains("cp_todo_pattern"))
        .collect();

    assert_eq!(todo_units.len(), 2, "Should extract 2 TODO comments");
    assert!(
        todo_units[0].content.contains("Implement this function"),
        "First TODO should contain 'Implement this function'"
    );
    assert!(
        todo_units[1].content.contains("Add error handling"),
        "Second TODO should contain 'Add error handling'"
    );
}

#[test]
fn test_custom_pattern_applied_to_regex_parser_files() {
    let custom_patterns = vec![CustomRegexPattern {
        name: "note_pattern".to_string(),
        file_extensions: vec!["md".to_string(), "txt".to_string()],
        category: StringLiteralCategory::Other,
        regex: r#"NOTE:\s*(.+)"#.to_string(),
        group: 1,
    }];

    let project_extraction_config = ProjectExtractionConfig {
        custom_patterns,
        ..Default::default()
    };

    let coordinator = ParserCoordinator::with_extraction_config(
        ParserConfig::default(),
        ExtractionConfig::default(),
        create_test_filter(),
        Some(project_extraction_config),
    )
    .unwrap();

    let md_content = r#"
# Documentation

NOTE: This is important information
NOTE: Another note here
"#;

    let file = create_test_file(md_content, "test.md");
    let units = coordinator.parse_file(&file).unwrap();

    let note_units: Vec<_> = units
        .iter()
        .filter(|u| u.id.contains("cp_note_pattern"))
        .collect();

    assert_eq!(note_units.len(), 2, "Should extract 2 NOTE comments");
}

#[test]
fn test_state_machine_applied_to_tree_sitter_files() {
    let state_machine_patterns = vec![StateMachinePattern {
        name: "i18n_pattern".to_string(),
        file_extensions: vec!["js".to_string(), "ts".to_string()],
        category: StringLiteralCategory::Other,
        extraction_rule: ExtractionRule::RemoveQuotes,
        states: vec![
            PatternState {
                name: "start".to_string(),
                regex: r#"t\s*\(\s*['"][^'"]+['"]\s*,\s*['"]"#.to_string(),
                capture_group: None,
                transitions: vec![StateTransition {
                    target: "extract".to_string(),
                    condition: None,
                }],
                is_final: false,
            },
            PatternState {
                name: "extract".to_string(),
                regex: r#"([^'"]+)"#.to_string(),
                capture_group: Some(1),
                transitions: vec![],
                is_final: true,
            },
        ],
        initial_state: "start".to_string(),
        accepting_states: vec!["extract".to_string()],
    }];

    let project_extraction_config = ProjectExtractionConfig {
        state_machine_patterns,
        ..Default::default()
    };

    let coordinator = ParserCoordinator::with_extraction_config(
        ParserConfig::default(),
        ExtractionConfig::default(),
        create_test_filter(),
        Some(project_extraction_config),
    )
    .unwrap();

    let js_content = r#"
function main() {
    t("hello", "Hello, World!")
    t("goodbye", "Goodbye!")
}
"#;

    let file = create_test_file(js_content, "test.js");
    let units = coordinator.parse_file(&file).unwrap();

    let i18n_units: Vec<_> = units
        .iter()
        .filter(|u| u.id.contains("sm_i18n_pattern"))
        .collect();

    assert_eq!(i18n_units.len(), 2, "Should extract 2 i18n strings");
    assert_eq!(i18n_units[0].content, "Hello, World!");
    assert_eq!(i18n_units[1].content, "Goodbye!");
}

#[test]
fn test_both_patterns_applied_to_same_file() {
    let custom_patterns = vec![CustomRegexPattern {
        name: "todo_pattern".to_string(),
        file_extensions: vec!["js".to_string()],
        category: StringLiteralCategory::Other,
        regex: r#"TODO:\s*(.+)"#.to_string(),
        group: 1,
    }];

    let state_machine_patterns = vec![StateMachinePattern {
        name: "error_pattern".to_string(),
        file_extensions: vec!["js".to_string()],
        category: StringLiteralCategory::ErrorHandling,
        extraction_rule: ExtractionRule::RemoveQuotes,
        states: vec![
            PatternState {
                name: "start".to_string(),
                regex: r#"throw new Error\s*\(\s*['"]"#.to_string(),
                capture_group: None,
                transitions: vec![StateTransition {
                    target: "extract".to_string(),
                    condition: None,
                }],
                is_final: false,
            },
            PatternState {
                name: "extract".to_string(),
                regex: r#"([^'"]+)"#.to_string(),
                capture_group: Some(1),
                transitions: vec![],
                is_final: true,
            },
        ],
        initial_state: "start".to_string(),
        accepting_states: vec!["extract".to_string()],
    }];

    let project_extraction_config = ProjectExtractionConfig {
        custom_patterns,
        state_machine_patterns,
        ..Default::default()
    };

    let coordinator = ParserCoordinator::with_extraction_config(
        ParserConfig::default(),
        ExtractionConfig::default(),
        create_test_filter(),
        Some(project_extraction_config),
    )
    .unwrap();

    let js_content = r#"
function main() {
    // TODO: Implement error handling
    if (error) {
        throw new Error("Invalid input")
    }
    // TODO: Add logging
}
"#;

    let file = create_test_file(js_content, "test.js");
    let units = coordinator.parse_file(&file).unwrap();

    let todo_units: Vec<_> = units
        .iter()
        .filter(|u| u.id.contains("cp_todo_pattern"))
        .collect();

    let error_units: Vec<_> = units
        .iter()
        .filter(|u| u.id.contains("sm_error_pattern"))
        .collect();

    assert_eq!(todo_units.len(), 2, "Should extract 2 TODO comments");
    assert_eq!(error_units.len(), 1, "Should extract 1 error message");
    assert_eq!(error_units[0].content, "Invalid input");
}

#[test]
fn test_pattern_with_wildcard_extension() {
    let custom_patterns = vec![CustomRegexPattern {
        name: "note_pattern".to_string(),
        file_extensions: vec![], // Empty means applies to all files
        category: StringLiteralCategory::Other,
        regex: r#"NOTE:\s*(.+)"#.to_string(),
        group: 1,
    }];

    let project_extraction_config = ProjectExtractionConfig {
        custom_patterns,
        ..Default::default()
    };

    let coordinator = ParserCoordinator::with_extraction_config(
        ParserConfig::default(),
        ExtractionConfig::default(),
        create_test_filter(),
        Some(project_extraction_config),
    )
    .unwrap();

    let py_content = "# NOTE: This is Python";
    let js_content = "// NOTE: This is JavaScript";
    let md_content = "NOTE: This is Markdown";

    let py_file = create_test_file(py_content, "test.py");
    let js_file = create_test_file(js_content, "test.js");
    let md_file = create_test_file(md_content, "test.md");

    let py_units = coordinator.parse_file(&py_file).unwrap();
    let js_units = coordinator.parse_file(&js_file).unwrap();
    let md_units = coordinator.parse_file(&md_file).unwrap();

    assert!(
        py_units.iter().any(|u| u.id.contains("cp_note_pattern")),
        "Should apply to Python file"
    );
    assert!(
        js_units.iter().any(|u| u.id.contains("cp_note_pattern")),
        "Should apply to JavaScript file"
    );
    assert!(
        md_units.iter().any(|u| u.id.contains("cp_note_pattern")),
        "Should apply to Markdown file"
    );
}
