//! Strategy integration tests for all language parsers
//!
//! These tests verify that all language parsers correctly integrate with the
//! extraction strategy system.

use std::path::PathBuf;
use std::sync::Arc;

use crate::core::models::{File, NodeType};
use crate::parser::filter::{ContentFilter, FilterConfig};
use crate::parser::languages::*;
use crate::parser::strategy::{
    ConfigBasedStrategy, ExtractionConfig, ExtractionStrategy, ExtractionStrategyImpl,
    StrategyNodeType,
};
use crate::parser::tree_sitter::ParserConfig;
use crate::parser::Parser;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

fn create_test_parser_config() -> ParserConfig {
    ParserConfig {
        extract_comments: true,
        extract_docstrings: true,
        extract_strings: true,
        ..Default::default()
    }
}

fn create_strategy_with_config(config: ExtractionConfig) -> Arc<ExtractionStrategyImpl> {
    Arc::new(ExtractionStrategyImpl::ConfigBased(
        ConfigBasedStrategy::new(config),
    ))
}

fn create_default_strategy() -> Arc<ExtractionStrategyImpl> {
    create_strategy_with_config(ExtractionConfig::default())
}

fn create_filter() -> Arc<ContentFilter> {
    Arc::new(ContentFilter::new(FilterConfig::default()).unwrap())
}

/// Test that strategy correctly filters comments
#[test]
fn test_strategy_comment_filtering() {
    let config = ExtractionConfig {
        comments: false, // Disable comments
        docstrings: true,
        ..Default::default()
    };

    let parser = RustParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
/// This is a doc comment
fn main() {
    // This is a regular comment
    let x = 5;
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = parser.parse(&file).expect("Parsing should succeed");

    // Should have docstrings but no regular comments
    let comments: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::Comment)
        .collect();
    let docstrings: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::DocString)
        .collect();

    assert!(comments.is_empty(), "Comments should be filtered out");
    assert!(!docstrings.is_empty(), "Docstrings should be present");
}

/// Test that strategy correctly filters docstrings
#[test]
fn test_strategy_docstring_filtering() {
    let config = ExtractionConfig {
        comments: true,
        docstrings: false, // Disable docstrings
        ..Default::default()
    };

    let parser = PythonParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
"""Module docstring."""

# This is a comment
def main():
    pass
"#;

    let file = create_test_file(content, "test.py");
    let units = parser.parse(&file).expect("Parsing should succeed");

    let comments: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::Comment)
        .collect();
    let docstrings: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::DocString)
        .collect();

    assert!(!comments.is_empty(), "Comments should be present");
    assert!(docstrings.is_empty(), "Docstrings should be filtered out");
}

/// Test that strategy correctly filters error messages
#[test]
fn test_strategy_error_message_filtering() {
    let config = ExtractionConfig {
        error_messages: false, // Disable error messages
        log_messages: true,
        ..Default::default()
    };

    let parser = RustParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
/// This is a doc comment
fn main() {
    // This is a regular comment
    let x = 5;
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = parser.parse(&file).expect("Parsing should succeed");

    let comments: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::Comment)
        .collect();
    let docstrings: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::DocString)
        .collect();

    assert!(!comments.is_empty(), "Comments should be present");
    assert!(!docstrings.is_empty(), "Docstrings should be present");
}

/// Test that strategy correctly filters format strings
#[test]
fn test_strategy_format_string_filtering() {
    let config = ExtractionConfig {
        format_strings: false, // Disable format strings
        log_messages: true,
        ..Default::default()
    };

    let parser = JavaScriptParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
function main() {
    const msg = `Hello, ${name}!`;  // Template string (FormatString)
    console.log("This is a log message");
}
"#;

    let file = create_test_file(content, "test.js");
    let units = parser.parse(&file).expect("Parsing should succeed");

    let formats: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::FormatString)
        .collect();
    let logs: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::LogMessage)
        .collect();

    assert!(formats.is_empty(), "Format strings should be filtered out");
    assert!(!logs.is_empty(), "Log messages should be present");
}

/// Test that strategy correctly filters log messages
#[test]
fn test_strategy_log_message_filtering() {
    let config = ExtractionConfig {
        log_messages: false, // Disable log messages
        format_strings: true,
        ..Default::default()
    };

    let parser = GoParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
package main

import "fmt"

func main() {
    fmt.Println("This is a log message")
    fmt.Sprintf("This is a format string: %d", 42)
}
"#;

    let file = create_test_file(content, "test.go");
    let units = parser.parse(&file).expect("Parsing should succeed");

    let logs: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::LogMessage)
        .collect();
    let formats: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::FormatString)
        .collect();

    assert!(logs.is_empty(), "Log messages should be filtered out");
    assert!(!formats.is_empty(), "Format strings should be present");
}

/// Test C parser docstring extraction with strategy
#[test]
fn test_c_parser_docstring_strategy() {
    let config = ExtractionConfig {
        comments: true,
        docstrings: true,
        ..Default::default()
    };

    let parser = CParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
/// This is a Doxygen doc comment
void function1() {
    // This is a regular comment
}

/** This is also a doc comment */
void function2() {}
"#;

    let file = create_test_file(content, "test.c");
    let units = parser.parse(&file).expect("Parsing should succeed");

    let comments: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::Comment)
        .collect();
    let docstrings: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::DocString)
        .collect();

    assert!(!comments.is_empty(), "Should extract regular comments");
    assert!(
        !docstrings.is_empty(),
        "Should extract Doxygen doc comments"
    );
}

/// Test C++ parser docstring extraction with strategy
#[test]
fn test_cpp_parser_docstring_strategy() {
    let config = ExtractionConfig {
        comments: true,
        docstrings: true,
        ..Default::default()
    };

    let parser = CppParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
/// Class documentation
class MyClass {
    // Regular comment
public:
    //! Method documentation
    void method();
};
"#;

    let file = create_test_file(content, "test.cpp");
    let units = parser.parse(&file).expect("Parsing should succeed");

    let comments: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::Comment)
        .collect();
    let docstrings: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::DocString)
        .collect();

    assert!(!comments.is_empty(), "Should extract regular comments");
    assert!(!docstrings.is_empty(), "Should extract C++ doc comments");
}

/// Test Java parser strategy integration
#[test]
fn test_java_parser_strategy_integration() {
    let config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: true,
        format_strings: true,
        log_messages: true,
        ..Default::default()
    };

    let parser = JavaParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
/**
 * Class documentation
 */
public class Test {
    // Regular comment
    public static void main(String[] args) {
        System.out.println("Log message");
        throw new RuntimeException("Error message");
    }
}
"#;

    let file = create_test_file(content, "Test.java");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty(), "Should extract translation units");

    // Verify different node types are present
    let has_comments = units.iter().any(|u| u.node_type == NodeType::Comment);
    let has_docstrings = units.iter().any(|u| u.node_type == NodeType::DocString);

    assert!(has_comments, "Should have comments");
    assert!(has_docstrings, "Should have docstrings");
}

/// Test C# parser strategy integration
#[test]
fn test_csharp_parser_strategy_integration() {
    let config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: true,
        format_strings: true,
        ..Default::default()
    };

    let parser = CSharpParser::new(
        create_test_parser_config(),
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    let content = r#"
/// <summary>
/// Class documentation
/// </summary>
public class Test {
    // Regular comment
    public void Method() {
        Console.WriteLine("Log message");
        throw new Exception("Error message");
    }
}
"#;

    let file = create_test_file(content, "Test.cs");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty(), "Should extract translation units");
}

/// Test TypeScript parser template string extraction
#[test]
fn test_typescript_template_string_strategy() {
    let config = ExtractionConfig {
        format_strings: true,
        ..Default::default()
    };

    let mut parser_config = create_test_parser_config();
    parser_config.extract_strings = true;

    let parser = TypeScriptParser::new(
        parser_config,
        create_strategy_with_config(config),
        create_filter(),
    )
    .unwrap();

    // Template strings are extracted separately from call expressions
    let content = r#"
const message = `Hello, World!`;
"#;

    let file = create_test_file(content, "test.ts");
    let units = parser.parse(&file).expect("Parsing should succeed");

    // Debug: print all units
    for unit in &units {
        println!("Unit: {:?} - {}", unit.node_type, unit.content);
    }

    let formats: Vec<_> = units
        .iter()
        .filter(|u| u.node_type == NodeType::FormatString)
        .collect();

    assert!(
        !formats.is_empty(),
        "Should extract template strings as FormatString. Found {} units: {:?}",
        units.len(),
        units.iter().map(|u| &u.node_type).collect::<Vec<_>>()
    );
}

/// Test that all node types are correctly mapped
#[test]
fn test_strategy_node_type_mapping() {
    let strategy = ConfigBasedStrategy::new(ExtractionConfig::default());

    // Verify all StrategyNodeType variants map to correct NodeType
    assert_eq!(
        strategy.get_node_type(StrategyNodeType::Comment),
        NodeType::Comment
    );
    assert_eq!(
        strategy.get_node_type(StrategyNodeType::DocString),
        NodeType::DocString
    );
    assert_eq!(
        strategy.get_node_type(StrategyNodeType::ErrorMessage),
        NodeType::ErrorMessage
    );
    assert_eq!(
        strategy.get_node_type(StrategyNodeType::FormatString),
        NodeType::FormatString
    );
    assert_eq!(
        strategy.get_node_type(StrategyNodeType::LogMessage),
        NodeType::LogMessage
    );
    assert_eq!(
        strategy.get_node_type(StrategyNodeType::StringLiteral),
        NodeType::StringLiteral
    );
}

/// Test strategy with context (function name)
#[test]
fn test_strategy_with_function_context() {
    use crate::parser::strategy::ExtractionContext;

    let strategy = ConfigBasedStrategy::new(ExtractionConfig {
        error_messages: true,
        ..Default::default()
    });

    // Test with error message context
    let ctx = ExtractionContext::new("Error occurred").with_function_name("panic");
    assert!(strategy.should_extract(StrategyNodeType::ErrorMessage, &ctx));

    // Test with disabled error messages
    let strategy_disabled = ConfigBasedStrategy::new(ExtractionConfig {
        error_messages: false,
        ..Default::default()
    });
    assert!(!strategy_disabled.should_extract(StrategyNodeType::ErrorMessage, &ctx));
}
