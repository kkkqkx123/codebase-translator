//! Regex-based fallback parser
//!
//! This module provides a fallback parser that uses regular expressions
//! to extract comments and strings when tree-sitter parsers are not available
//! or for simple file types.

use regex::Regex;

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, NodeType, Position, TranslationUnit};
use crate::parser::Parser as ParserTrait;

/// Regex-based parser configuration
#[derive(Debug, Clone)]
pub struct RegexParserConfig {
    /// File extensions this parser supports
    pub extensions: Vec<String>,
    /// Regex pattern for line comments
    pub line_comment_pattern: Option<String>,
    /// Regex pattern for block comments
    pub block_comment_pattern: Option<String>,
    /// Regex pattern for doc comments
    pub doc_comment_pattern: Option<String>,
    /// Regex pattern for string literals
    pub string_pattern: Option<String>,
    /// Minimum content length
    pub min_content_length: usize,
    /// Maximum content length
    pub max_content_length: usize,
    /// Whether to trim content
    pub trim_content: bool,
}

impl Default for RegexParserConfig {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            line_comment_pattern: None,
            block_comment_pattern: None,
            doc_comment_pattern: None,
            string_pattern: None,
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
        }
    }
}

/// Regex-based parser for extracting translatable content
pub struct RegexParser {
    config: super::tree_sitter::ParserConfig,
    regex_config: RegexParserConfig,
    line_comment_regex: Option<Regex>,
    block_comment_regex: Option<Regex>,
    doc_comment_regex: Option<Regex>,
    string_regex: Option<Regex>,
}

impl RegexParser {
    /// Create a new regex parser with default configuration
    pub fn new(config: super::tree_sitter::ParserConfig) -> Self {
        Self::with_config(config, RegexParserConfig::default())
    }

    /// Create a new regex parser with custom configuration
    pub fn with_config(
        config: super::tree_sitter::ParserConfig,
        regex_config: RegexParserConfig,
    ) -> Self {
        let line_comment_regex = regex_config
            .line_comment_pattern
            .as_ref()
            .and_then(|p| Regex::new(p).ok());

        let block_comment_regex = regex_config
            .block_comment_pattern
            .as_ref()
            .and_then(|p| Regex::new(p).ok());

        let doc_comment_regex = regex_config
            .doc_comment_pattern
            .as_ref()
            .and_then(|p| Regex::new(p).ok());

        let string_regex = regex_config
            .string_pattern
            .as_ref()
            .and_then(|p| Regex::new(p).ok());

        Self {
            config,
            regex_config,
            line_comment_regex,
            block_comment_regex,
            doc_comment_regex,
            string_regex,
        }
    }

    /// Create a generic fallback parser that handles common comment styles
    pub fn create_fallback_parser(config: super::tree_sitter::ParserConfig) -> Self {
        let regex_config = RegexParserConfig {
            extensions: vec!["txt".to_string(), "md".to_string(), "markdown".to_string()],
            // Match common comment patterns
            line_comment_pattern: Some(r"(?m)^\s*(?://|#|--|;)\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"/\*\s*([\s\S]*?)\s*\*/".to_string()),
            doc_comment_pattern: None,
            string_pattern: Some(r#"["']([^"']{3,})["']"#.to_string()),
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
        };
        Self::with_config(config, regex_config)
    }

    /// Create a shell script parser
    pub fn create_shell_parser(config: super::tree_sitter::ParserConfig) -> Self {
        let regex_config = RegexParserConfig {
            extensions: vec![
                "sh".to_string(),
                "bash".to_string(),
                "zsh".to_string(),
                "fish".to_string(),
            ],
            line_comment_pattern: Some(r"(?m)^\s*#\s*(.+)$".to_string()),
            block_comment_pattern: None,
            doc_comment_pattern: None,
            string_pattern: Some(r#"["']([^"']{3,})["']"#.to_string()),
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
        };
        Self::with_config(config, regex_config)
    }

    /// Create an HTML parser
    pub fn create_html_parser(config: super::tree_sitter::ParserConfig) -> Self {
        let regex_config = RegexParserConfig {
            extensions: vec![
                "html".to_string(),
                "htm".to_string(),
                "xml".to_string(),
                "svg".to_string(),
            ],
            line_comment_pattern: None,
            block_comment_pattern: Some(r"<!--\s*([\s\S]*?)\s*-->".to_string()),
            doc_comment_pattern: None,
            string_pattern: None,
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
        };
        Self::with_config(config, regex_config)
    }

    /// Create a SQL parser
    pub fn create_sql_parser(config: super::tree_sitter::ParserConfig) -> Self {
        let regex_config = RegexParserConfig {
            extensions: vec!["sql".to_string(), "mysql".to_string(), "pgsql".to_string()],
            line_comment_pattern: Some(r"(?m)^\s*--\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"/\*\s*([\s\S]*?)\s*\*/".to_string()),
            doc_comment_pattern: None,
            string_pattern: Some(r#"'([^']{3,})'"#.to_string()),
            min_content_length: config.min_content_length,
            max_content_length: config.max_content_length,
            trim_content: config.trim_content,
        };
        Self::with_config(config, regex_config)
    }

    /// Parse content and extract translation units
    fn parse_content(&self, content: &str, file_path: &str) -> Result<Vec<TranslationUnit>> {
        let mut units = Vec::new();
        let mut id_counter = 0;

        // Extract line comments
        if let Some(ref regex) = self.line_comment_regex {
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let text = if self.config.trim_content {
                            group.as_str().trim()
                        } else {
                            group.as_str()
                        };

                        if self.should_include(text) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = self.byte_to_position(content, start_byte);
                            let end_pos = self.byte_to_position(content, end_byte);

                            let id = format!("{}_comment_{}", file_path, id_counter);
                            id_counter += 1;

                            let unit = TranslationUnit::new(
                                id,
                                NodeType::Comment,
                                text.to_string(),
                                start_pos,
                                end_pos,
                            );
                            units.push(unit);
                        }
                    }
                }
            }
        }

        // Extract block comments
        if let Some(ref regex) = self.block_comment_regex {
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let text = if self.config.trim_content {
                            group.as_str().trim()
                        } else {
                            group.as_str()
                        };

                        if self.should_include(text) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = self.byte_to_position(content, start_byte);
                            let end_pos = self.byte_to_position(content, end_byte);

                            let id = format!("{}_block_{}", file_path, id_counter);
                            id_counter += 1;

                            let unit = TranslationUnit::new(
                                id,
                                NodeType::Comment,
                                text.to_string(),
                                start_pos,
                                end_pos,
                            );
                            units.push(unit);
                        }
                    }
                }
            }
        }

        // Extract doc comments
        if let Some(ref regex) = self.doc_comment_regex {
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let text = if self.config.trim_content {
                            group.as_str().trim()
                        } else {
                            group.as_str()
                        };

                        if self.should_include(text) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = self.byte_to_position(content, start_byte);
                            let end_pos = self.byte_to_position(content, end_byte);

                            let id = format!("{}_doc_{}", file_path, id_counter);
                            id_counter += 1;

                            let unit = TranslationUnit::new(
                                id,
                                NodeType::DocString,
                                text.to_string(),
                                start_pos,
                                end_pos,
                            );
                            units.push(unit);
                        }
                    }
                }
            }
        }

        // Extract strings
        if let Some(ref regex) = self.string_regex {
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let text = if self.config.trim_content {
                            group.as_str().trim()
                        } else {
                            group.as_str()
                        };

                        if self.should_include(text) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = self.byte_to_position(content, start_byte);
                            let end_pos = self.byte_to_position(content, end_byte);

                            let id = format!("{}_string_{}", file_path, id_counter);
                            id_counter += 1;

                            let unit = TranslationUnit::new(
                                id,
                                NodeType::FormatString,
                                text.to_string(),
                                start_pos,
                                end_pos,
                            );
                            units.push(unit);
                        }
                    }
                }
            }
        }

        // Sort by position
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }

    /// Check if content should be included based on filters
    fn should_include(&self, text: &str) -> bool {
        let len = text.len();
        if len < self.config.min_content_length || len > self.config.max_content_length {
            return false;
        }

        // Skip if only symbols/whitespace
        if text.chars().all(|c| c.is_whitespace() || is_punctuation(c)) {
            return false;
        }

        true
    }

    /// Convert byte offset to line/column position
    fn byte_to_position(&self, content: &str, byte_offset: usize) -> Position {
        let content_up_to_offset = &content[..byte_offset.min(content.len())];
        let lines: Vec<&str> = content_up_to_offset.lines().collect();

        let line = lines.len();
        let column = lines.last().map(|l| l.len() + 1).unwrap_or(1);

        Position::new(line, column, byte_offset)
    }
}

impl ParserTrait for RegexParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file
            .content_string()
            .map_err(|e| TranslateError::Parse(format!("Invalid UTF-8 content: {}", e)))?;

        let file_path = file.path.to_string_lossy();
        self.parse_content(&content, &file_path)
    }

    fn supports(&self, filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        self.regex_config
            .extensions
            .iter()
            .any(|ext| filename_lower.ends_with(&format!(".{}", ext.to_lowercase())))
    }

    fn supported_extensions(&self) -> &[&str] {
        // Return empty slice - caller should check supports() method
        &[]
    }
}

/// Check if character is punctuation
fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// Factory for creating regex-based parsers
pub struct RegexParserFactory;

impl RegexParserFactory {
    /// Create all available regex parsers
    pub fn create_all_parsers(config: super::tree_sitter::ParserConfig) -> Vec<RegexParser> {
        vec![
            RegexParser::create_fallback_parser(config.clone()),
            RegexParser::create_shell_parser(config.clone()),
            RegexParser::create_html_parser(config.clone()),
            RegexParser::create_sql_parser(config.clone()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    #[tokio::test]
    async fn test_shell_parser() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RegexParser::create_shell_parser(config);

        let content = r#"#!/bin/bash
# This is a comment
echo "hello world"  # inline comment
"#;

        let file = create_test_file(content, "test.sh");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.sh"));
        assert!(parser.supports("script.bash"));
        assert!(!parser.supports("test.rs"));
    }

    #[tokio::test]
    async fn test_html_parser() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RegexParser::create_html_parser(config);

        let content = r#"<!DOCTYPE html>
<!-- This is an HTML comment -->
<html>
<body>
<!-- Another comment -->
</body>
</html>"#;

        let file = create_test_file(content, "test.html");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.html"));
        assert!(parser.supports("page.htm"));
    }

    #[tokio::test]
    async fn test_sql_parser() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RegexParser::create_sql_parser(config);

        let content = r#"-- This is a SQL comment
SELECT * FROM users;
/* Multi-line
   comment */"#;

        let file = create_test_file(content, "test.sql");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("query.sql"));
    }

    #[test]
    fn test_should_include() {
        let config = super::super::tree_sitter::ParserConfig::default();
        let parser = RegexParser::create_fallback_parser(config);

        assert!(parser.should_include("hello world"));
        assert!(!parser.should_include("x")); // Too short
        assert!(!parser.should_include("   ")); // Only whitespace
        assert!(!parser.should_include("// ")); // Only symbols
    }
}
