//! Regex-based parser implementation

use regex::Regex;
use tracing::{debug, info, warn};

use crate::core::error::Result;
use crate::core::models::{File, NodeType, TranslationUnit};
use crate::parser::core::Parser as ParserTrait;
use crate::parser::core::StringProcessor;
use crate::parser::ParserConfig;

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
    string_processor: StringProcessor,
}

impl RegexParser {
    /// Create a new regex parser with default configuration
    pub fn new(config: ParserConfig) -> Self {
        Self::with_config(config, RegexParserConfig::default())
    }

    /// Create a new regex parser with custom configuration
    pub fn with_config(config: ParserConfig, regex_config: RegexParserConfig) -> Self {
        debug!(
            extensions = ?regex_config.extensions,
            has_line_comment = regex_config.line_comment_pattern.is_some(),
            has_block_comment = regex_config.block_comment_pattern.is_some(),
            has_doc_comment = regex_config.doc_comment_pattern.is_some(),
            has_string_pattern = regex_config.string_pattern.is_some(),
            state_machine_patterns_count = regex_config.state_machine_patterns.len(),
            "Creating regex parser with configuration"
        );

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
        let state_machine_matchers: Vec<_> = regex_config
            .state_machine_patterns
            .iter()
            .filter_map(|pattern| {
                StateMachineMatcher::from_config(
                    pattern.name.clone(),
                    pattern.initial_state.clone(),
                    pattern.accepting_states.clone(),
                    &pattern.states,
                    pattern.extraction_rule.clone(),
                )
                .ok()
            })
            .collect();

        debug!(
            line_comment_regex_valid = line_comment_regex.is_some(),
            block_comment_regex_valid = block_comment_regex.is_some(),
            doc_comment_regex_valid = doc_comment_regex.is_some(),
            string_regex_valid = string_regex.is_some(),
            state_machine_matchers_count = state_machine_matchers.len(),
            "Regex parser compilation completed"
        );

        Self {
            config,
            regex_config,
            line_comment_regex,
            block_comment_regex,
            doc_comment_regex,
            string_regex,
            state_machine_matchers,
            string_processor: StringProcessor::new(),
        }
    }

    /// Parse content and extract translation units
    fn parse_content(&self, content: &str, file_path: &str) -> Result<Vec<TranslationUnit>> {
        info!(content_length = content.len(), "Starting regex parsing");
        let mut units = Vec::new();
        let mut id_counter = 0;

        // Extract line comments
        if let Some(ref regex) = self.line_comment_regex {
            let initial_count = units.len();
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let raw_text = group.as_str();
                        let text = self
                            .string_processor
                            .clean_comment(raw_text, crate::parser::core::CommentType::Line);

                        if should_include(
                            &text,
                            self.config.min_content_length,
                            self.config.max_content_length,
                        ) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

                            let id = format!("{}_comment_{}", file_path, id_counter);
                            id_counter += 1;

                            let mut unit = TranslationUnit::new(
                                id,
                                NodeType::Comment,
                                text,
                                start_pos,
                                end_pos,
                            );
                            unit.raw_match = Some(mat.as_str().to_string());
                            units.push(unit);
                        }
                    }
                }
            }
            debug!(
                count = units.len() - initial_count,
                "Line comments extracted"
            );
        }

        // Extract block comments
        if let Some(ref regex) = self.block_comment_regex {
            let initial_count = units.len();
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let raw_text = group.as_str();
                        let text = self
                            .string_processor
                            .clean_comment(raw_text, crate::parser::core::CommentType::Block);

                        if should_include(
                            &text,
                            self.config.min_content_length,
                            self.config.max_content_length,
                        ) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

                            let id = format!("{}_block_{}", file_path, id_counter);
                            id_counter += 1;

                            let mut unit = TranslationUnit::new(
                                id,
                                NodeType::Comment,
                                text,
                                start_pos,
                                end_pos,
                            );
                            unit.raw_match = Some(mat.as_str().to_string());
                            units.push(unit);
                        }
                    }
                }
            }
            debug!(
                count = units.len() - initial_count,
                "Block comments extracted"
            );
        }

        // Extract doc comments
        if let Some(ref regex) = self.doc_comment_regex {
            let initial_count = units.len();
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let raw_text = group.as_str();
                        let text = self
                            .string_processor
                            .clean_comment(raw_text, crate::parser::core::CommentType::Doc);

                        if should_include(
                            &text,
                            self.config.min_content_length,
                            self.config.max_content_length,
                        ) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

                            let id = format!("{}_doc_{}", file_path, id_counter);
                            id_counter += 1;

                            let mut unit = TranslationUnit::new(
                                id,
                                NodeType::DocString,
                                text,
                                start_pos,
                                end_pos,
                            );
                            unit.raw_match = Some(mat.as_str().to_string());
                            units.push(unit);
                        }
                    }
                }
            }
            debug!(
                count = units.len() - initial_count,
                "Doc comments extracted"
            );
        }

        // Extract strings
        if let Some(ref regex) = self.string_regex {
            let initial_count = units.len();
            for mat in regex.find_iter(content) {
                if let Some(captured) = regex.captures(&content[mat.start()..mat.end()]) {
                    if let Some(group) = captured.get(1) {
                        let raw_text = group.as_str();
                        let text = self.string_processor.clean_string_literal(raw_text);

                        if should_include(
                            &text,
                            self.config.min_content_length,
                            self.config.max_content_length,
                        ) {
                            let start_byte = mat.start() + group.start();
                            let end_byte = mat.start() + group.end();
                            let start_pos = byte_to_position(content, start_byte);
                            let end_pos = byte_to_position(content, end_byte);

                            let id = format!("{}_string_{}", file_path, id_counter);
                            id_counter += 1;

                            let mut unit = TranslationUnit::new(
                                id,
                                NodeType::FormatString,
                                text,
                                start_pos,
                                end_pos,
                            );
                            unit.raw_match = Some(mat.as_str().to_string());
                            units.push(unit);
                        }
                    }
                }
            }
            debug!(count = units.len() - initial_count, "Strings extracted");
        }

        // Apply state machine patterns
        for matcher in &self.state_machine_matchers {
            let initial_count = units.len();
            let matches = matcher.find_matches(content)?;
            for m in matches {
                if should_include(
                    &m.extracted_text,
                    self.config.min_content_length,
                    self.config.max_content_length,
                ) {
                    let id = format!("{}_sm_{}_{}", file_path, matcher.name, id_counter);
                    id_counter += 1;

                    let mut unit = TranslationUnit::new(
                        id,
                        NodeType::StringLiteral,
                        m.extracted_text,
                        m.start_pos,
                        m.end_pos,
                    );
                    unit.raw_match = Some(m.raw_content);
                    units.push(unit);
                }
            }
            debug!(
                matcher_name = %matcher.name,
                count = units.len() - initial_count,
                "State machine matches extracted"
            );
        }

        // Sort by position
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        info!(total_units = units.len(), "Regex parsing completed");
        Ok(units)
    }

    /// Get the regex config
    pub fn regex_config(&self) -> &RegexParserConfig {
        &self.regex_config
    }
}

impl ParserTrait for RegexParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file.content_string().map_err(|e| {
            warn!(error = %e, "Failed to get content string from file");
            crate::core::error::TranslateError::Parse(format!("Invalid UTF-8 content: {}", e))
        })?;

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
