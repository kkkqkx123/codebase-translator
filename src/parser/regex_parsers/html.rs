//! HTML/XML parser

use tracing::debug;

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};
use crate::parser::abstraction::parser::Parser as ParserTrait;
use crate::parser::engine::ParserConfig;
use crate::parser::regex::{RegexParser, RegexParserConfig};

/// HTML/XML parser
pub struct HtmlParser {
    inner: RegexParser,
}

impl HtmlParser {
    /// Create a new HTML parser
    pub fn new(config: ParserConfig) -> Self {
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
            state_machine_patterns: Vec::new(),
        };
        Self {
            inner: RegexParser::with_config(config, regex_config),
        }
    }
}

impl ParserTrait for HtmlParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        debug!(
            format = "html/xml",
            file = %file.path.display(),
            "Parsing special format"
        );
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["html", "htm", "xml", "svg"]
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
    fn test_html_parser_supports() {
        let parser = HtmlParser::new(ParserConfig::default());

        assert!(parser.supports("page.html"));
        assert!(parser.supports("page.htm"));
        assert!(parser.supports("config.xml"));
        assert!(parser.supports("image.svg"));
        assert!(!parser.supports("test.rs"));
        assert!(!parser.supports("script.sh"));
    }

    #[test]
    fn test_html_parser_extracts_comments() {
        let parser = HtmlParser::new(ParserConfig::default());

        let content = r#"<!-- This is a comment -->
<div>Hello</div>
"#;

        let file = create_test_file(content, "test.html");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }
}
