//! Generic fallback parser for simple file types

use tracing::warn;

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};
use crate::parser::core::traits::Parser as ParserTrait;
use crate::parser::ParserConfig;
use crate::parser::regex::{RegexParser, RegexParserConfig};

/// Generic fallback parser for simple file types
pub struct FallbackParser {
    inner: RegexParser,
}

impl FallbackParser {
    /// Create a new fallback parser
    pub fn new(config: ParserConfig) -> Self {
        let regex_config = RegexParserConfig {
            extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "markdown".to_string(),
                "yml".to_string(),
                "yaml".to_string(),
                "toml".to_string(),
            ],
            line_comment_pattern: Some(r"(?m)^\s*(?://|#|--|;)\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"/\*\s*([\s\S]*?)\s*\*/".to_string()),
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

    /// Get a reference to the inner regex parser
    pub fn inner_parser(&self) -> &RegexParser {
        &self.inner
    }
}

impl ParserTrait for FallbackParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        warn!(
            file = %file.path.display(),
            reason = "no_tree_sitter_parser",
            "Using fallback parser"
        );
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["txt", "md", "markdown", "yml", "yaml", "toml"]
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
    fn test_fallback_parser_supports() {
        let parser = FallbackParser::new(ParserConfig::default());

        assert!(parser.supports("test.txt"));
        assert!(parser.supports("readme.md"));
        assert!(parser.supports("config.yaml"));
        assert!(parser.supports("config.yml"));
        assert!(parser.supports("Cargo.toml"));
        assert!(!parser.supports("test.rs"));
        assert!(!parser.supports("script.sh"));
    }

    #[test]
    fn test_fallback_parser_extracts_comments() {
        let parser = FallbackParser::new(ParserConfig::default());

        let content = r#"# This is a comment
key = value  # inline comment
"#;

        let file = create_test_file(content, "test.txt");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }
}
