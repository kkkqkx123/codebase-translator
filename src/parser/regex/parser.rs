//! Regex-based parser implementation

use regex::Regex;

use crate::core::error::Result;
use crate::core::models::{File, NodeType, TranslationUnit};
use crate::parser::r#trait::Parser as ParserTrait;
use crate::parser::tree_sitter::ParserConfig;

use super::config::RegexParserConfig;
use super::state_machine::StateMachineMatcher;
use super::utils::{byte_to_position, should_include};

/// Regex-based parser for extracting translatable content
pub struct RegexParser {
    config: ParserConfig,
    regex_config: RegexParserConfig,
    line_comment_regex: Option<Regex>,
    block_comment_regex: Option<Regex>,
    doc_comment_regex: Option<Regex>,
    string_regex: Option<Regex>,
    state_machine_matchers: Vec<StateMachineMatcher>,
}

impl RegexParser {
    /// Create a new regex parser with default configuration
    pub fn new(config: ParserConfig) -> Self {
        Self::with_config(config, RegexParserConfig::default())
    }

    /// Create a new regex parser with custom configuration
    pub fn with_config(config: ParserConfig, regex_config: RegexParserConfig) -> Self {
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

        // Build state machine matchers from config
        let state_machine_matchers = regex_config
            .state_machine_patterns
            .iter()
            .filter_map(|pattern| {
                StateMachineMatcher::from_config(
                    pattern.name.clone(),
                    pattern.initial_state.clone(),
                    pattern.accepting_states.clone(),
                    &pattern.states,
                )
                .ok()
            })
            .collect();

        Self {
            config,
            regex_config,
            line_comment_regex,
            block_comment_regex,
            doc_comment_regex,
            string_regex,
            state_machine_matchers,
        }
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

                        if should_include(text, self.config.min_content_length, self.config.max_content_length) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

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

                        if should_include(text, self.config.min_content_length, self.config.max_content_length) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

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

                        if should_include(text, self.config.min_content_length, self.config.max_content_length) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

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

                        if should_include(text, self.config.min_content_length, self.config.max_content_length) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

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

        // Apply state machine patterns
        for matcher in &self.state_machine_matchers {
            let matches = matcher.find_matches(content)?;
            for m in matches {
                if should_include(&m.content, self.config.min_content_length, self.config.max_content_length) {
                    let id = format!("{}_sm_{}_{}", file_path, matcher.name, id_counter);
                    id_counter += 1;

                    let unit = TranslationUnit::new(
                        id,
                        NodeType::StringLiteral,
                        m.content,
                        m.start_pos,
                        m.end_pos,
                    );
                    units.push(unit);
                }
            }
        }

        // Sort by position
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }

    /// Get the regex config
    pub fn regex_config(&self) -> &RegexParserConfig {
        &self.regex_config
    }
}

impl ParserTrait for RegexParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file
            .content_string()
            .map_err(|e| crate::core::error::TranslateError::Parse(format!("Invalid UTF-8 content: {}", e)))?;

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
