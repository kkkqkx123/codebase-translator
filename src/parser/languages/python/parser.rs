//! Python language parser implementation
//!
//! This is a refactored version using the core extraction framework.

use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::abstraction::parser::Parser as ParserTrait;
use crate::parser::abstraction::strategy::{
    ExtractionContext, ExtractionStrategy, StrategyNodeType,
};
use crate::parser::filtering::traits::Filter;
use crate::parser::{ContentFilter, ExtractionStrategyImpl, FunctionCategory};
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::string_processor::{CleanedString, CommentType};
use crate::parser::core::StringProcessor;
use crate::parser::engine::ParserConfig;
use crate::parser::languages::python::patterns::PythonPatterns;
use crate::parser::languages::python::queries::PythonQueries;
use tracing::{debug, error, info, instrument, warn};

/// Python language parser
pub struct PythonParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
    patterns: PythonPatterns,
    string_processor: StringProcessor,
}

impl PythonParser {
    /// Create a new Python parser
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            strategy,
            filter,
            patterns: PythonPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing Python comment markers
    fn clean_comment_text(&self, text: &str) -> String {
        self.string_processor.clean_comment(text, CommentType::Line)
    }

    /// Clean docstring by removing triple quotes
    ///
    /// Preserves newlines and removes common leading indentation from all lines.
    fn clean_docstring_text(&self, text: &str) -> CleanedString {
        // Only trim leading/trailing whitespace on the outer edges, not internal newlines
        let trimmed = text
            .trim_start()
            .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

        let content = if trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\"") {
            &trimmed[3..trimmed.len() - 3]
        } else if trimmed.starts_with("'''") && trimmed.ends_with("'''") {
            &trimmed[3..trimmed.len() - 3]
        } else if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
            &trimmed[1..trimmed.len() - 1]
        } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() > 1 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            return CleanedString {
                text: trimmed.to_string(),
                placeholders: Vec::new(),
            };
        };

        // Process lines to remove common leading indentation
        let lines: Vec<&str> = content.lines().collect();

        // Find the minimum indentation (excluding empty lines)
        let min_indent = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
            .min()
            .unwrap_or(0);

        // Remove common indentation from each line
        let processed_lines: Vec<String> = lines
            .iter()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    line.chars().skip(min_indent).collect()
                }
            })
            .collect();

        // Join lines and trim trailing whitespace
        let cleaned_text = processed_lines.join("\n").trim_end().to_string();

        CleanedString {
            text: cleaned_text,
            placeholders: Vec::new(),
        }
    }

    /// Parse file content into a syntax tree
    #[instrument(skip(self, content))]
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set Python language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse Python syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "Python syntax tree parsed successfully"
        );

        Ok(tree)
    }

    /// Extract comments using the core framework
    #[instrument(skip(self, root_node, content))]
    fn extract_comments(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        debug!(file = %file_path, "Extracting comments");

        let executor = QueryExecutor::from_string(
            &tree_sitter_python::LANGUAGE.into(),
            PythonQueries::all_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        debug!(
            file = %file_path,
            total_matches = matches.len(),
            "Comment query executed"
        );

        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean comment marker (#)
            let text = self.clean_comment_text(m.text);

            // Apply trim if configured
            let text = if self.config.trim_content {
                text.trim().to_string()
            } else {
                text
            };

            // Apply length filters
            if text.len() < self.config.min_content_length {
                debug!(
                    file = %file_path,
                    text_len = text.len(),
                    min_length = self.config.min_content_length,
                    "Comment filtered: too short"
                );
                continue;
            }
            if text.len() > self.config.max_content_length {
                debug!(
                    file = %file_path,
                    text_len = text.len(),
                    max_length = self.config.max_content_length,
                    "Comment filtered: too long"
                );
                continue;
            }

            // Skip if only symbols
            if self.string_processor.is_only_symbols(&text) {
                debug!(
                    file = %file_path,
                    text = %text,
                    "Comment filtered: only symbols"
                );
                continue;
            }

            // Apply content filter
            if !self.filter.should_translate(&text) {
                debug!(
                    file = %file_path,
                    text = %text,
                    "Comment filtered: content filter"
                );
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(&text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::Comment, &ctx)
            {
                debug!(
                    file = %file_path,
                    text = %text,
                    "Comment filtered: extraction strategy"
                );
                continue;
            }

            let id = format!("{}_comment_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::Comment);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        debug!(
            file = %file_path,
            extracted_units = units.len(),
            "Comments extracted"
        );

        Ok(units)
    }

    /// Extract docstrings using the core framework
    /// In Python, docstrings are string literals that appear as the first statement in modules, classes, or functions
    #[instrument(skip(self, root_node, content))]
    fn extract_docstrings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        debug!(file = %file_path, "Extracting docstrings");

        let executor = QueryExecutor::from_string(
            &tree_sitter_python::LANGUAGE.into(),
            PythonQueries::docstrings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        debug!(
            file = %file_path,
            total_matches = matches.len(),
            "Docstring query executed"
        );

        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean docstring quotes ("""...""" or '''...''')
            let cleaned = self.clean_docstring_text(m.text);

            // Apply trim if configured
            let text = if self.config.trim_content {
                cleaned.text.trim().to_string()
            } else {
                cleaned.text
            };

            // Apply length filters
            if text.len() < self.config.min_content_length {
                debug!(
                    file = %file_path,
                    text_len = text.len(),
                    min_length = self.config.min_content_length,
                    "Docstring filtered: too short"
                );
                continue;
            }
            if text.len() > self.config.max_content_length {
                debug!(
                    file = %file_path,
                    text_len = text.len(),
                    max_length = self.config.max_content_length,
                    "Docstring filtered: too long"
                );
                continue;
            }

            // Skip if only symbols
            if self.string_processor.is_only_symbols(&text) {
                debug!(
                    file = %file_path,
                    text = %text,
                    "Docstring filtered: only symbols"
                );
                continue;
            }

            // Apply content filter
            if !self.filter.should_translate(&text) {
                debug!(
                    file = %file_path,
                    text = %text,
                    "Docstring filtered: content filter"
                );
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(&text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::DocString, &ctx)
            {
                debug!(
                    file = %file_path,
                    text = %text,
                    "Docstring filtered: extraction strategy"
                );
                continue;
            }

            let id = format!("{}_docstring_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::DocString);
            let mut unit = TranslationUnit::new_with_pattern(
                id,
                node_type,
                text,
                m.start_pos,
                m.end_pos,
                crate::core::models::PatternType::Builtin,
                "python",
            );
            unit.raw_match = Some(m.text.to_string());
            units.push(unit);
            match_idx += 1;
        }

        debug!(
            file = %file_path,
            extracted_units = units.len(),
            "Docstrings extracted"
        );

        Ok(units)
    }

    /// Extract function call strings using the core framework
    #[instrument(skip(self, root_node, content))]
    fn extract_function_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        debug!(file = %file_path, "Extracting function strings");

        let executor = QueryExecutor::from_string(
            &tree_sitter_python::LANGUAGE.into(),
            PythonQueries::function_call_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        debug!(
            file = %file_path,
            total_matches = matches.len(),
            "Function string query executed"
        );

        let mut units = Vec::new();
        let mut match_idx = 0usize;

        // Group matches by function call
        let mut current_func = String::new();
        let mut current_operand = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                "func_name" => {
                    current_func = m.text.to_string();
                }
                "operand" => {
                    current_operand = m.text.to_string();
                }
                "func_string" => {
                    // Build full function name for attribute expressions
                    let full_func_name = if !current_operand.is_empty() {
                        format!("{}.{}", current_operand, current_func)
                    } else {
                        current_func.clone()
                    };

                    if full_func_name.is_empty() {
                        debug!(
                            file = %file_path,
                            "Function string filtered: empty function name"
                        );
                        continue;
                    }

                    // Clean the string literal
                    let cleaned = self.string_processor.clean_string_literal(m.text);

                    // Apply filter
                    if !self.filter.should_translate(&cleaned) {
                        debug!(
                            file = %file_path,
                            function = %full_func_name,
                            text = %cleaned,
                            "Function string filtered: content filter"
                        );
                        continue;
                    }

                    // Classify function
                    let strategy_node_type = match self.patterns.classify_function(&full_func_name)
                    {
                        Some(FunctionCategory::Error) => {
                            debug!(
                                file = %file_path,
                                function = %full_func_name,
                                "Classified as error message"
                            );
                            StrategyNodeType::ErrorMessage
                        }
                        Some(FunctionCategory::Format) => {
                            debug!(
                                file = %file_path,
                                function = %full_func_name,
                                "Classified as format string"
                            );
                            StrategyNodeType::FormatString
                        }
                        Some(FunctionCategory::Log) => {
                            debug!(
                                file = %file_path,
                                function = %full_func_name,
                                "Classified as log message"
                            );
                            StrategyNodeType::LogMessage
                        }
                        _ => {
                            debug!(
                                file = %file_path,
                                function = %full_func_name,
                                "Function string filtered: unknown function category"
                            );
                            continue;
                        }
                    };

                    // Apply strategy
                    let ctx = ExtractionContext::new(&cleaned).with_function_name(&full_func_name);
                    if !self.strategy.should_extract(strategy_node_type, &ctx) {
                        debug!(
                            file = %file_path,
                            function = %full_func_name,
                            text = %cleaned,
                            "Function string filtered: extraction strategy"
                        );
                        continue;
                    }

                    let id = format!("{}_func_{}", file_path, match_idx);
                    let node_type = self.strategy.get_node_type(strategy_node_type);
                    let mut unit = TranslationUnit::new_with_pattern(
                        id,
                        node_type,
                        cleaned,
                        m.start_pos,
                        m.end_pos,
                        crate::core::models::PatternType::Builtin,
                        "python",
                    );
                    unit.raw_match = Some(m.text.to_string());
                    units.push(unit);
                    match_idx += 1;
                }
                _ => {}
            }
        }

        debug!(
            file = %file_path,
            extracted_units = units.len(),
            "Function strings extracted"
        );

        Ok(units)
    }

    /// Extract all translation units from the syntax tree
    fn extract_units(
        &self,
        tree: &Tree,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let mut units = Vec::new();
        let root_node = tree.root_node();

        // Extract comments
        if self.config.extract_comments {
            let comment_units = self.extract_comments(&root_node, content, file_path)?;
            units.extend(comment_units);
        }

        // Extract docstrings
        if self.config.extract_docstrings {
            let doc_units = self.extract_docstrings(&root_node, content, file_path)?;
            units.extend(doc_units);
        }

        // Extract function strings
        if self.config.extract_strings {
            let func_units = self.extract_function_strings(&root_node, content, file_path)?;
            units.extend(func_units);
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }
}

impl ParserTrait for PythonParser {
    #[instrument(skip(self, file))]
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing Python file"
        );

        let content = file.content_string().map_err(|e| {
            error!(file = %file_path, error = %e, "Failed to decode UTF-8 content");
            TranslateError::Parse(format!("Invalid UTF-8 content: {}", e))
        })?;

        let tree = self.parse_tree(&content)?;
        let file_path = file.path.to_string_lossy().to_string();

        let units = self.extract_units(&tree, &content, &file_path)?;

        info!(
            file = %file_path,
            units = units.len(),
            comments = units.iter().filter(|u| u.node_type == crate::core::models::NodeType::Comment).count(),
            docstrings = units.iter().filter(|u| u.node_type == crate::core::models::NodeType::DocString).count(),
            error_messages = units.iter().filter(|u| u.node_type == crate::core::models::NodeType::ErrorMessage).count(),
            log_messages = units.iter().filter(|u| u.node_type == crate::core::models::NodeType::LogMessage).count(),
            format_strings = units.iter().filter(|u| u.node_type == crate::core::models::NodeType::FormatString).count(),
            duration_ms = start.elapsed().as_millis(),
            "Python file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        filename.to_lowercase().ends_with(".py")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["py"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::NodeType;
    use crate::parser::filtering::FilterConfig;
    use crate::parser::abstraction::strategy::ExtractionConfig;
    use crate::parser::core::strategies::ConfigBasedStrategy;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    fn create_test_parser() -> PythonParser {
        let config = ParserConfig {
            extract_strings: true,
            ..Default::default()
        };
        let extraction_config = ExtractionConfig {
            format_strings: true,
            ..Default::default()
        };
        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(extraction_config),
        ));
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        PythonParser::new(config, strategy, filter).unwrap()
    }

    #[tokio::test]
    async fn test_python_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
"""Module docstring."""

# This is a comment
def main():
    """Function docstring."""
    print("Hello, World!")
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.py"));
        assert!(!parser.supports("test.rs"));
    }

    #[tokio::test]
    async fn test_python_comments() {
        let parser = create_test_parser();

        let content = r#"
# Line comment
def test():
    """Docstring"""
    pass
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_python_docstrings() {
        let parser = create_test_parser();

        let content = r#"
"""Module docstring."""

class Person:
    """Person class docstring."""
    
    def greet(self):
        """Greet method docstring."""
        return "Hello"
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let docstrings: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::DocString)
            .collect();

        assert!(!docstrings.is_empty(), "Should extract docstrings");
    }

    #[tokio::test]
    async fn test_python_function_calls() {
        let parser = create_test_parser();

        let content = r#"
import logging

logger = logging.getLogger(__name__)

def main():
    print("Hello, World!")
    logger.info("Application started")
    raise ValueError("Invalid input")
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        // Should extract strings from function calls
        assert!(!units.is_empty());
    }

    #[tokio::test]
    async fn test_python_f_strings() {
        let parser = create_test_parser();

        let content = r#"
def main():
    name = "World"
    print("Hello, World!")
    print("Value: 42")
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        // Should handle strings in function calls
        assert!(!units.is_empty());
    }

    #[tokio::test]
    async fn test_python_complex_structure() {
        let parser = create_test_parser();

        let content = r#"
"""Main module."""

import logging

logger = logging.getLogger(__name__)

class Config:
    """Configuration class."""
    
    def __init__(self, debug: bool = False):
        """Initialize configuration."""
        self.debug = debug
        
        if debug:
            logger.debug("Debug mode enabled")

def main():
    """Main entry point."""
    config = Config(debug=True)
    logger.info("Application started")
    print("Hello, World!")
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should extract translation units");
    }

    #[tokio::test]
    async fn test_python_assertions() {
        let parser = create_test_parser();

        let content = r#"
def test():
    print("Assertion failed")
    print("Values are not equal")
"#;

        let file = create_test_file(content, "test.py");
        let units = parser.parse(&file).expect("Parsing should succeed");

        // Should extract strings from print statements
        assert!(!units.is_empty());
    }
}

