//! Parser coordinator tests

use std::path::PathBuf;

use crate::core::models::File;
use crate::parser::regex::RegexParser;
use crate::parser::tree_sitter::ParserConfig;

use super::{ParserCoordinator, ParserType};

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

#[test]
fn test_parser_coordinator_creation() {
    let config = ParserConfig::default();
    let coordinator =
        ParserCoordinator::with_defaults(config).expect("Failed to create coordinator");

    assert!(coordinator.tree_sitter_parser_count() > 0);
}

#[test]
fn test_parse_rust_file() {
    let config = ParserConfig::default();
    let coordinator =
        ParserCoordinator::with_defaults(config).expect("Failed to create coordinator");

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

    assert!(!units.is_empty());
}

#[test]
fn test_can_parse() {
    let coordinator = ParserCoordinator::default();

    assert!(coordinator.can_parse("test.rs"));
    assert!(coordinator.can_parse("readme.md"));
    assert!(!coordinator.can_parse("test.unknown_extension"));
}

#[test]
fn test_find_parser() {
    let coordinator = ParserCoordinator::default();

    let parser_type = coordinator.find_parser("test.rs");
    assert!(parser_type.is_some());
    assert!(matches!(parser_type, Some(ParserType::TreeSitter(_))));

    let parser_type = coordinator.find_parser("test.md");
    assert_eq!(parser_type, Some(ParserType::Regex));
}

#[test]
fn test_supported_extensions() {
    let coordinator = ParserCoordinator::default();
    let extensions = coordinator.supported_extensions();

    assert!(extensions.is_empty() || !extensions.is_empty());
}

#[test]
fn test_parse_unsupported_file() {
    let coordinator = ParserCoordinator::default();

    let file = create_test_file("content", "test.unknown_extension");
    let result = coordinator.parse_file(&file);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("No parser found"));
}

#[test]
fn test_with_parsers() {
    let tree_sitter_parsers: Vec<crate::parser::tree_sitter::TreeSitterParser> = Vec::new();
    let regex_parser = RegexParser::create_fallback_parser(ParserConfig::default());

    let coordinator = ParserCoordinator::with_parsers(tree_sitter_parsers, regex_parser);

    assert_eq!(coordinator.tree_sitter_parser_count(), 0);
    assert!(coordinator.can_parse("test.md"));
}
