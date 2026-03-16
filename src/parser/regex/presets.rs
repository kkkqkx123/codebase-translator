//! Preset regex parsers for common file types

use super::config::RegexParserConfig;
use super::parser::RegexParser;
use crate::parser::tree_sitter::ParserConfig;

/// Create a generic fallback parser that handles common comment styles
pub fn create_fallback_parser(config: ParserConfig) -> RegexParser {
    RegexParser::with_config(config, RegexParserConfig::fallback())
}

/// Create a shell script parser
pub fn create_shell_parser(config: ParserConfig) -> RegexParser {
    RegexParser::with_config(config, RegexParserConfig::shell())
}

/// Create an HTML parser
pub fn create_html_parser(config: ParserConfig) -> RegexParser {
    RegexParser::with_config(config, RegexParserConfig::html())
}

/// Create a SQL parser
pub fn create_sql_parser(config: ParserConfig) -> RegexParser {
    RegexParser::with_config(config, RegexParserConfig::sql())
}

/// Create a Markdown parser
pub fn create_markdown_parser(config: ParserConfig) -> RegexParser {
    let regex_config = RegexParserConfig {
        extensions: vec!["md".to_string(), "markdown".to_string()],
        line_comment_pattern: None,
        block_comment_pattern: Some(r"<!--\s*([\s\S]*?)\s*-->".to_string()),
        doc_comment_pattern: None,
        string_pattern: None,
        min_content_length: config.min_content_length,
        max_content_length: config.max_content_length,
        trim_content: config.trim_content,
        state_machine_patterns: Vec::new(),
    };
    RegexParser::with_config(config, regex_config)
}

/// Create a YAML parser
pub fn create_yaml_parser(config: ParserConfig) -> RegexParser {
    let regex_config = RegexParserConfig {
        extensions: vec!["yml".to_string(), "yaml".to_string()],
        line_comment_pattern: Some(r"(?m)^\s*#\s*(.+)$".to_string()),
        block_comment_pattern: None,
        doc_comment_pattern: None,
        string_pattern: Some(r#"["']([^"']{3,})["']"#.to_string()),
        min_content_length: config.min_content_length,
        max_content_length: config.max_content_length,
        trim_content: config.trim_content,
        state_machine_patterns: Vec::new(),
    };
    RegexParser::with_config(config, regex_config)
}

/// Create a TOML parser
pub fn create_toml_parser(config: ParserConfig) -> RegexParser {
    let regex_config = RegexParserConfig {
        extensions: vec!["toml".to_string()],
        line_comment_pattern: Some(r"(?m)^\s*#\s*(.+)$".to_string()),
        block_comment_pattern: None,
        doc_comment_pattern: None,
        string_pattern: Some(r#""([^"]{3,})""#.to_string()),
        min_content_length: config.min_content_length,
        max_content_length: config.max_content_length,
        trim_content: config.trim_content,
        state_machine_patterns: Vec::new(),
    };
    RegexParser::with_config(config, regex_config)
}

/// Create all available preset parsers
pub fn create_all_parsers(config: ParserConfig) -> Vec<RegexParser> {
    vec![
        create_fallback_parser(config.clone()),
        create_shell_parser(config.clone()),
        create_html_parser(config.clone()),
        create_sql_parser(config.clone()),
        create_markdown_parser(config.clone()),
        create_yaml_parser(config.clone()),
        create_toml_parser(config.clone()),
    ]
}

/// Get all supported extensions from presets
pub fn get_all_supported_extensions() -> Vec<&'static str> {
    vec![
        "txt", "md", "markdown", "sh", "bash", "zsh", "fish", "html", "htm", "xml", "svg", "sql",
        "mysql", "pgsql", "yml", "yaml", "toml",
    ]
}
