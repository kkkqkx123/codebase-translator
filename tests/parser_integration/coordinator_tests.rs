//! Parser Coordinator Integration Tests
//!
//! Tests for the parser coordinator that manages multiple parsers
//! and routes files to appropriate parsers.

use std::path::PathBuf;
use std::sync::Arc;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::{ParserCoordinator, ParserType};
use codebase_translate::parser::abstraction::filter::{ContentFilter, FilterConfig};
use codebase_translate::parser::regex::RegexParser;
use codebase_translate::parser::abstraction::strategy::{
    default_strategy, ConfigBasedStrategy, ExtractionConfig, ExtractionStrategyImpl,
};
use codebase_translate::parser::engine::ParserConfig;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

fn create_test_coordinator() -> ParserCoordinator {
    ParserCoordinator::with_defaults(ParserConfig::default())
        .expect("Failed to create coordinator")
}

#[test]
fn test_coordinator_creation_with_defaults() {
    let coordinator = create_test_coordinator();

    assert!(
        coordinator.tree_sitter_parser_count() > 0,
        "Should have at least one tree-sitter parser"
    );
}

#[test]
fn test_coordinator_creation_with_custom_config() {
    let parser_config = ParserConfig {
        extract_comments: true,
        extract_docstrings: false,
        extract_strings: true,
        trim_content: true,
        ..Default::default()
    };

    let filter_config = FilterConfig {
        min_length: 5,
        max_length: 500,
        ..Default::default()
    };

    let filter = Arc::new(
        ContentFilter::new(filter_config).expect("Failed to create filter")
    );
    let strategy = Arc::new(default_strategy());

    let coordinator =
        ParserCoordinator::new(parser_config, strategy, filter).expect("Failed to create coordinator");

    assert!(coordinator.tree_sitter_parser_count() > 0);
}

#[test]
fn test_coordinator_routing_tree_sitter_priority() {
    let coordinator = create_test_coordinator();

    let rust_file = "test.rs";
    let parser_type = coordinator.find_parser(rust_file);

    assert!(
        parser_type.is_some(),
        "Should find parser for .rs file"
    );
    assert!(
        matches!(parser_type, Some(ParserType::TreeSitter(_))),
        "Rust files should use TreeSitter parser"
    );
}

#[test]
fn test_coordinator_routing_regex_fallback() {
    let coordinator = create_test_coordinator();

    let md_file = "readme.md";
    let parser_type = coordinator.find_parser(md_file);

    assert!(
        parser_type.is_some(),
        "Should find parser for .md file"
    );
    assert_eq!(
        parser_type,
        Some(ParserType::Regex),
        "Markdown files should use Regex parser"
    );
}

#[test]
fn test_coordinator_routing_unknown_extension() {
    let coordinator = create_test_coordinator();

    let unknown_file = "test.unknown_extension";
    let parser_type = coordinator.find_parser(unknown_file);

    assert!(
        parser_type.is_none(),
        "Should not find parser for unknown extension"
    );
}

#[test]
fn test_can_parse_supported_extensions() {
    let coordinator = create_test_coordinator();

    assert!(coordinator.can_parse("test.rs"), "Should support .rs files");
    assert!(coordinator.can_parse("test.py"), "Should support .py files");
    assert!(coordinator.can_parse("test.go"), "Should support .go files");
    assert!(coordinator.can_parse("test.java"), "Should support .java files");
    assert!(coordinator.can_parse("test.js"), "Should support .js files");
    assert!(coordinator.can_parse("test.ts"), "Should support .ts files");
    assert!(coordinator.can_parse("test.c"), "Should support .c files");
    assert!(coordinator.can_parse("test.cpp"), "Should support .cpp files");
    assert!(coordinator.can_parse("test.cs"), "Should support .cs files");
    assert!(coordinator.can_parse("test.md"), "Should support .md files");
    assert!(coordinator.can_parse("test.txt"), "Should support .txt files");
}

#[test]
fn test_can_parse_unsupported_extensions() {
    let coordinator = create_test_coordinator();

    assert!(
        !coordinator.can_parse("test.unknown"),
        "Should not support unknown extensions"
    );
    assert!(
        !coordinator.can_parse("test.bin"),
        "Should not support .bin files"
    );
    assert!(
        !coordinator.can_parse("test.exe"),
        "Should not support .exe files"
    );
}

#[test]
fn test_parse_rust_file_with_comments() {
    let coordinator = create_test_coordinator();

    let content = r#"
/// This is a doc comment
fn main() {
    // This is a regular comment
    let x = 5;
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    assert!(!units.is_empty(), "Should extract translation units");
}

#[test]
fn test_parse_python_file_with_docstrings() {
    let coordinator = create_test_coordinator();

    let content = r#"
def hello():
    """This is a docstring."""
    # This is a comment
    pass
"#;

    let file = create_test_file(content, "test.py");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    assert!(!units.is_empty(), "Should extract translation units");
}

#[test]
fn test_parse_go_file_with_comments() {
    let coordinator = create_test_coordinator();

    let content = r#"
package main

// This is a comment
func main() {
    /* This is a block comment */
}
"#;

    let file = create_test_file(content, "test.go");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    assert!(!units.is_empty(), "Should extract translation units");
}

#[test]
fn test_parse_markdown_file() {
    let coordinator = create_test_coordinator();

    let content = r#"# Header

This is a paragraph.

// This looks like a comment
"#;

    let file = create_test_file(content, "test.md");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    assert!(!units.is_empty(), "Should extract translation units from markdown");
}

#[test]
fn test_parse_unsupported_file_returns_error() {
    let coordinator = create_test_coordinator();

    let file = create_test_file("content", "test.unknown_extension");
    let result = coordinator.parse_file(&file);

    assert!(result.is_err(), "Should return error for unsupported file");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("No parser found"),
        "Error should indicate no parser found"
    );
}

#[test]
fn test_supported_extensions_list() {
    let coordinator = create_test_coordinator();
    let extensions = coordinator.supported_extensions();

    assert!(!extensions.is_empty(), "Should have supported extensions");

    let expected_extensions = vec![
        "rs", "py", "go", "java", "js", "ts", "c", "cpp", "cs", "md", "txt",
    ];

    for ext in expected_extensions {
        assert!(
            extensions.iter().any(|e| e == ext),
            "Should support .{} files",
            ext
        );
    }
}

#[test]
fn test_coordinator_with_custom_strategy() {
    let config = ParserConfig::default();
    let strategy_config = ExtractionConfig {
        comments: true,
        docstrings: false,
        error_messages: true,
        format_strings: false,
        log_messages: true,
        custom_patterns: vec![],
    };
    let strategy = Arc::new(
        ConfigBasedStrategy::new(strategy_config).expect("Failed to create strategy"),
    );
    let filter = Arc::new(
        ContentFilter::new(FilterConfig::default()).expect("Failed to create filter"),
    );

    let coordinator =
        ParserCoordinator::new(config, strategy, filter).expect("Failed to create coordinator");

    assert!(coordinator.tree_sitter_parser_count() > 0);
}

#[test]
fn test_coordinator_with_parsers_constructor() {
    let tree_sitter_parsers: Vec<codebase_translate::parser::engine::TreeSitterParser> =
        Vec::new();
    let regex_parser = RegexParser::create_fallback_parser(ParserConfig::default());

    let coordinator = ParserCoordinator::with_parsers(tree_sitter_parsers, regex_parser);

    assert_eq!(coordinator.tree_sitter_parser_count(), 0);
    assert!(coordinator.can_parse("test.md"), "Should still support markdown via regex");
}

#[test]
fn test_multi_language_project_parsing() {
    let coordinator = create_test_coordinator();

    let files = vec![
        ("fn main() {}", "main.rs"),
        ("def main(): pass", "main.py"),
        ("package main\nfunc main() {}", "main.go"),
        ("public class Main {}", "Main.java"),
    ];

    for (content, path) in files {
        let file = create_test_file(content, path);
        let result = coordinator.parse_file(&file);
        assert!(
            result.is_ok(),
            "Should parse {} successfully",
            path
        );
    }
}

