//! SQL parser

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};
use crate::parser::regex::{RegexParser, RegexParserConfig};
use crate::parser::tree_sitter::ParserConfig;
use crate::parser::Parser as ParserTrait;

/// SQL parser
pub struct SqlParser {
    inner: RegexParser,
}

impl SqlParser {
    /// Create a new SQL parser
    pub fn new(config: ParserConfig) -> Self {
        let regex_config = RegexParserConfig {
            extensions: vec!["sql".to_string(), "mysql".to_string(), "pgsql".to_string()],
            line_comment_pattern: Some(r"(?m)^\s*--\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"/\*\s*([\s\S]*?)\s*\*/".to_string()),
            doc_comment_pattern: None,
            string_pattern: Some(r#"'([^']{3,})'"#.to_string()),
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

impl ParserTrait for SqlParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["sql", "mysql", "pgsql"]
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
    fn test_sql_parser_supports() {
        let parser = SqlParser::new(ParserConfig::default());

        assert!(parser.supports("query.sql"));
        assert!(parser.supports("schema.mysql"));
        assert!(parser.supports("data.pgsql"));
        assert!(!parser.supports("test.rs"));
        assert!(!parser.supports("script.sh"));
    }

    #[test]
    fn test_sql_parser_extracts_comments() {
        let parser = SqlParser::new(ParserConfig::default());

        let content = r#"-- This is a comment
SELECT * FROM users;  -- inline comment
"#;

        let file = create_test_file(content, "test.sql");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }
}
