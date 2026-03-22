//! Shell script parser

use tracing::debug;

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};
use crate::parser::abstraction::parser::Parser as ParserTrait;
use crate::parser::ParserConfig;
use crate::parser::regex::{RegexParser, RegexParserConfig};

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
                "ps1".to_string(),
                "psm1".to_string(),
                "psd1".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
            ],
            line_comment_pattern: Some(r"(?m)^\s*(?:#|REM|::)\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"<#\s*([\s\S]*?)\s*#>".to_string()),
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
        debug!(
            format = "shell",
            file = %file.path.display(),
            "Parsing special format"
        );
        self.inner.parse(file)
    }

    fn supports(&self, filename: &str) -> bool {
        self.inner.supports(filename)
    }

    fn supported_extensions(&self) -> &[&str] {
        &[
            "sh", "bash", "zsh", "fish", "ps1", "psm1", "psd1", "bat", "cmd",
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

    #[test]
    fn test_shell_parser_supports() {
        let parser = ShellParser::new(ParserConfig::default());

        assert!(parser.supports("script.sh"));
        assert!(parser.supports("script.bash"));
        assert!(parser.supports("config.zsh"));
        assert!(parser.supports("script.fish"));
        assert!(parser.supports("script.ps1"));
        assert!(parser.supports("module.psm1"));
        assert!(parser.supports("manifest.psd1"));
        assert!(parser.supports("script.bat"));
        assert!(parser.supports("script.cmd"));
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

    #[test]
    fn test_powershell_single_line_comments() {
        let parser = ShellParser::new(ParserConfig::default());

        let content = r#"# PowerShell single line comment
Write-Host "Hello"  # inline comment
# Another comment
"#;

        let file = create_test_file(content, "test.ps1");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }

    #[test]
    fn test_powershell_block_comments() {
        let parser = ShellParser::new(ParserConfig::default());

        let content = r#"<#
This is a multi-line
PowerShell comment
#>
Write-Host "Hello"
"#;

        let file = create_test_file(content, "test.ps1");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }

    #[test]
    fn test_cmd_rem_comments() {
        let parser = ShellParser::new(ParserConfig::default());

        let content = r#"@echo off
REM This is a REM comment
echo Hello World
REM Another comment
"#;

        let file = create_test_file(content, "test.bat");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }

    #[test]
    fn test_cmd_colon_comments() {
        let parser = ShellParser::new(ParserConfig::default());

        let content = r#"@echo off
:: This is a colon comment
echo Hello World
:: Another comment
"#;

        let file = create_test_file(content, "test.cmd");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
    }
}
