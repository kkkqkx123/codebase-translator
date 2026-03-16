//! Shell script parser

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};
use crate::parser::regex::{RegexParser, RegexParserConfig};
use crate::parser::tree_sitter::ParserConfig;
use crate::parser::Parser as ParserTrait;

/// Shell script parser
pub struct ShellParser {
    inner: RegexParser,
}

impl ShellParser {
    /// Create a new shell parser
    pub fn new(config: ParserConfig) -> Self {
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
            state_machine_patterns: Vec::new(),
        };
        Self {
            inner: RegexParser::with_config(config, regex_config),
        }
    }
}

impl ParserTrait for ShellParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["sh", "bash", "zsh", "fish"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    #[test]
    fn test_shell_parser_supports() {
        let parser = ShellParser::new(ParserConfig::default());

        assert!(parser.supports("script.sh"));
        assert!(parser.supports("script.bash"));
        assert!(parser.supports("config.zsh"));
        assert!(parser.supports("script.fish"));
        assert!(!parser.supports("test.rs"));
        assert!(!parser.supports("test.txt"));
    }

    #[test]
    fn test_shell_parser_extracts_comments() {
        let parser = ShellParser::new(ParserConfig::default());

        let content = r#"#!/bin/bash
# This is a comment
echo "hello world"  # inline comment
"#;

        let file = create_test_file(content, "test.sh");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }
}
