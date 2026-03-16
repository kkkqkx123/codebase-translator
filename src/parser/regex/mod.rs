//! Regex-based fallback parser
//!
//! This module provides a fallback parser that uses regular expressions
//! to extract comments and strings when tree-sitter parsers are not available
//! or for simple file types.
//!
//! # Architecture
//!
//! The regex parser module is organized into several submodules:
//!
//! - `config`: Parser configuration structures
//! - `parser`: Main regex parser implementation
//! - `state_machine`: State machine pattern matcher for complex extraction
//! - `presets`: Pre-configured parsers for common file types
//! - `factory`: Factory for creating parsers
//! - `utils`: Utility functions

// Configuration
pub mod config;

// Main parser
pub mod parser;

// State machine matcher
pub mod state_machine;

// Preset parsers
pub mod presets;

// Factory
pub mod factory;

// Utilities
pub mod utils;

// Re-exports
pub use config::RegexParserConfig;
pub use factory::RegexParserFactory;
pub use parser::RegexParser;
pub use state_machine::{StateMachineBuilder, StateMachineMatch, StateMachineMatcher};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::File;
    use crate::parser::Parser;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    #[tokio::test]
    async fn test_shell_parser() {
        let config = crate::parser::tree_sitter::ParserConfig::default();
        let parser = presets::create_shell_parser(config);

        let content = r#"#!/bin/bash
# This is a comment
echo "hello world"  # inline comment
"#;

        let file = create_test_file(content, "test.sh");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.sh"));
        assert!(parser.supports("script.bash"));
    }

    #[test]
    fn test_html_parser() {
        let config = crate::parser::tree_sitter::ParserConfig::default();
        let parser = presets::create_html_parser(config);

        let content = r#"<!-- This is a comment -->
<div>Hello</div>
"#;

        let file = create_test_file(content, "test.html");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.html"));
        assert!(parser.supports("page.htm"));
    }

    #[test]
    fn test_factory_creates_all_parsers() {
        let config = crate::parser::tree_sitter::ParserConfig::default();
        let parsers = RegexParserFactory::create_all_parsers(config);

        assert!(!parsers.is_empty());
    }
}
