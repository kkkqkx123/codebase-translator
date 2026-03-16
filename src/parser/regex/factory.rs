//! Factory for creating regex-based parsers

use crate::parser::tree_sitter::ParserConfig;

use super::parser::RegexParser;
use super::presets;

/// Factory for creating regex-based parsers
pub struct RegexParserFactory;

impl RegexParserFactory {
    /// Create all available regex parsers
    pub fn create_all_parsers(config: ParserConfig) -> Vec<RegexParser> {
        presets::create_all_parsers(config)
    }

    /// Create a parser by file extension
    pub fn create_for_extension(config: ParserConfig, ext: &str) -> Option<RegexParser> {
        match ext.to_lowercase().as_str() {
            "txt" | "md" | "markdown" => Some(presets::create_fallback_parser(config)),
            "sh" | "bash" | "zsh" | "fish" => Some(presets::create_shell_parser(config)),
            "html" | "htm" | "xml" | "svg" => Some(presets::create_html_parser(config)),
            "sql" | "mysql" | "pgsql" => Some(presets::create_sql_parser(config)),
            "yml" | "yaml" => Some(presets::create_yaml_parser(config)),
            "toml" => Some(presets::create_toml_parser(config)),
            _ => None,
        }
    }

    /// Get all supported extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        presets::get_all_supported_extensions()
    }
}
