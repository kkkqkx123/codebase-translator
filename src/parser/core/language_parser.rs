//! Generic language parser trait
//!
//! This module provides a common trait for language parsers to reduce code duplication
//! and ensure consistent extraction configuration across all language implementations.

use std::sync::Arc;
use tree_sitter::{Language, Node, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::TranslationUnit;
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::traits::{ExtractionConfig, StrategyNodeType};
use crate::parser::core::StringProcessor;
use crate::parser::filtering::traits::Filter;
use crate::parser::ParserConfig;
use crate::parser::{ContentFilter, FunctionCategory};
use tracing::{debug, error};

/// Generic language parser trait
///
/// This trait provides default implementations for common extraction operations
/// that are shared across different language parsers.
pub trait LanguageParser: Send + Sync {
    /// Get the parser configuration
    fn config(&self) -> &ParserConfig;

    /// Get the extraction configuration
    fn extraction_config(&self) -> &ExtractionConfig;

    /// Get the content filter
    fn filter(&self) -> &Arc<ContentFilter>;

    /// Get the string processor
    fn string_processor(&self) -> &StringProcessor;

    /// Get the tree-sitter language
    fn tree_sitter_language(&self) -> Language;

    /// Parse content into a syntax tree
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&self.tree_sitter_language())
            .map_err(|e| {
                error!(error = %e, "Failed to set language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "Syntax tree parsed successfully"
        );

        Ok(tree)
    }

    /// Extract comments using a query
    fn extract_with_strategy(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
        query: &str,
        strategy_node_type: StrategyNodeType,
        id_prefix: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(&self.tree_sitter_language(), query)?;
        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();

        for (idx, m) in matches.iter().enumerate() {
            let text = if self.config().trim_content {
                m.text.trim()
            } else {
                m.text
            };

            // Apply length filters
            if text.len() < self.config().min_content_length {
                continue;
            }
            if text.len() > self.config().max_content_length {
                continue;
            }

            // Skip if only symbols
            if self.string_processor().is_only_symbols(text) {
                continue;
            }

            // Apply content filter
            if !self.filter().should_translate(text) {
                continue;
            }

            // Apply extraction config
            if !self.extraction_config().should_extract(strategy_node_type) {
                continue;
            }

            let id = format!("{}_{}_{}", file_path, id_prefix, idx);
            let node_type = self.extraction_config().get_node_type(strategy_node_type);
            let unit =
                TranslationUnit::new(id, node_type, text.to_string(), m.start_pos, m.end_pos);
            units.push(unit);
        }

        Ok(units)
    }

    /// Extract strings from function/method calls
    #[allow(clippy::too_many_arguments)]
    fn extract_function_strings<F>(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
        query: &str,
        func_name_capture: &str,
        string_capture: &str,
        classify_function: F,
    ) -> Result<Vec<TranslationUnit>>
    where
        F: Fn(&str) -> Option<FunctionCategory>,
    {
        let executor = QueryExecutor::from_string(&self.tree_sitter_language(), query)?;
        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        // Group matches by function call
        let mut current_func = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                name if name == func_name_capture => {
                    current_func = m.text.to_string();
                }
                name if name == string_capture => {
                    if current_func.is_empty() {
                        continue;
                    }

                    // Clean the string literal
                    let text = self.string_processor().clean_string_literal(m.text);

                    // Apply filter
                    if !self.filter().should_translate(&text) {
                        continue;
                    }

                    // Classify function
                    let strategy_node_type = match classify_function(&current_func) {
                        Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                        Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                        Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                        Some(FunctionCategory::Debug) => StrategyNodeType::LogMessage,
                        None => continue, // Skip unknown functions
                    };

                    // Apply extraction config
                    if !self.extraction_config().should_extract(strategy_node_type) {
                        continue;
                    }

                    let id = format!("{}_func_{}", file_path, match_idx);
                    let node_type = self.extraction_config().get_node_type(strategy_node_type);
                    let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
                    units.push(unit);
                    match_idx += 1;
                }
                _ => {}
            }
        }

        Ok(units)
    }

    /// Extract strings with operand (e.g., obj.method calls)
    #[allow(clippy::too_many_arguments)]
    fn extract_function_strings_with_operand<F>(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
        query: &str,
        operand_capture: &str,
        func_name_capture: &str,
        string_capture: &str,
        classify_function: F,
    ) -> Result<Vec<TranslationUnit>>
    where
        F: Fn(&str) -> Option<FunctionCategory>,
    {
        let executor = QueryExecutor::from_string(&self.tree_sitter_language(), query)?;
        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        let mut current_func = String::new();
        let mut current_operand = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                name if name == func_name_capture => {
                    current_func = m.text.to_string();
                }
                name if name == operand_capture => {
                    current_operand = m.text.to_string();
                }
                name if name == string_capture => {
                    // Build full function name
                    let full_func_name = if !current_operand.is_empty() {
                        format!("{}.{}", current_operand, current_func)
                    } else {
                        current_func.clone()
                    };

                    if full_func_name.is_empty() {
                        continue;
                    }

                    let text = self.string_processor().clean_string_literal(m.text);

                    if !self.filter().should_translate(&text) {
                        continue;
                    }

                    let strategy_node_type = match classify_function(&full_func_name) {
                        Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                        Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                        Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                        Some(FunctionCategory::Debug) => StrategyNodeType::LogMessage,
                        None => continue,
                    };

                    if !self.extraction_config().should_extract(strategy_node_type) {
                        continue;
                    }

                    let id = format!("{}_func_{}", file_path, match_idx);
                    let node_type = self.extraction_config().get_node_type(strategy_node_type);
                    let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
                    units.push(unit);
                    match_idx += 1;
                }
                _ => {}
            }
        }

        Ok(units)
    }

    /// Extract raw strings with a specific strategy type
    fn extract_raw_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
        query: &str,
        strategy_node_type: StrategyNodeType,
        id_prefix: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(&self.tree_sitter_language(), query)?;
        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();

        for (idx, m) in matches.iter().enumerate() {
            let text = self.string_processor().clean_string_literal(m.text);

            if !self.filter().should_translate(&text) {
                continue;
            }

            if !self.extraction_config().should_extract(strategy_node_type) {
                continue;
            }

            let id = format!("{}_{}_{}", file_path, id_prefix, idx);
            let node_type = self.extraction_config().get_node_type(strategy_node_type);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
        }

        Ok(units)
    }
}
