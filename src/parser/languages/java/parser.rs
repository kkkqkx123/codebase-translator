//! Java language parser implementation
//!
//! This is a refactored version using the core extraction framework.

use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::{CommentType, StringProcessor};
use crate::parser::filter::ContentFilter;
use crate::parser::languages::java::patterns::JavaPatterns;
use crate::parser::languages::java::queries::JavaQueries;
use crate::parser::strategy::{
    ExtractionContext, ExtractionStrategy, ExtractionStrategyImpl, StrategyNodeType,
};
use crate::parser::tree_sitter::ParserConfig;
use crate::parser::Parser as ParserTrait;
use tracing::{debug, error, info, instrument};

/// Java language parser
pub struct JavaParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
    patterns: JavaPatterns,
    string_processor: StringProcessor,
}

impl JavaParser {
    /// Create a new Java parser
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            strategy,
            filter,
            patterns: JavaPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing Java comment markers
    fn clean_comment_text(&self, text: &str) -> String {
        let trimmed = text.trim();

        // Handle Javadoc comments: /**
        if trimmed.starts_with("/**") {
            return self
                .string_processor
                .clean_comment(trimmed, CommentType::Doc);
        }

        // Handle block comments: /*
        if trimmed.starts_with("/*") {
            return self
                .string_processor
                .clean_comment(trimmed, CommentType::Block);
        }

        // Handle line comments: //
        if trimmed.starts_with("//") {
            return self
                .string_processor
                .clean_comment(trimmed, CommentType::Line);
        }

        trimmed.to_string()
    }

    /// Parse file content into a syntax tree
    #[instrument(skip(self, content))]
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_java::LANGUAGE.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set Java language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse Java syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "Java syntax tree parsed successfully"
        );

        Ok(tree)
    }

    /// Extract comments using the core framework
    fn extract_comments(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_java::LANGUAGE.into(),
            JavaQueries::all_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean comment markers (//, /* */)
            let text = self.clean_comment_text(m.text);

            // Apply trim if configured
            let text = if self.config.trim_content {
                text.trim().to_string()
            } else {
                text
            };

            // Apply length filters
            if text.len() < self.config.min_content_length {
                continue;
            }
            if text.len() > self.config.max_content_length {
                continue;
            }

            // Skip if only symbols
            if self.string_processor.is_only_symbols(&text) {
                continue;
            }

            // Apply content filter
            if !self.filter.should_translate(&text) {
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(&text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::Comment, &ctx)
            {
                continue;
            }

            let id = format!("{}_comment_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::Comment);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract Javadoc comments using the core framework
    fn extract_javadoc(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_java::LANGUAGE.into(),
            JavaQueries::javadoc_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean Javadoc markers (/** */)
            let text = self.clean_comment_text(m.text);

            // Apply trim if configured
            let text = if self.config.trim_content {
                text.trim().to_string()
            } else {
                text
            };

            // Apply length filters
            if text.len() < self.config.min_content_length {
                continue;
            }
            if text.len() > self.config.max_content_length {
                continue;
            }

            // Skip if only symbols
            if self.string_processor.is_only_symbols(&text) {
                continue;
            }

            // Apply content filter
            if !self.filter.should_translate(&text) {
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(&text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::DocString, &ctx)
            {
                continue;
            }

            let id = format!("{}_javadoc_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::DocString);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract method strings using the core framework
    fn extract_method_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_java::LANGUAGE.into(),
            JavaQueries::method_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        // Group matches by method invocation
        let mut current_method = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                "method_name" => {
                    current_method = m.text.to_string();
                }
                "method_string" => {
                    if current_method.is_empty() {
                        continue;
                    }

                    // Clean the string literal
                    let text = self.string_processor.clean_string_literal(m.text);

                    // Apply filter
                    if !self.filter.should_translate(&text) {
                        continue;
                    }

                    // Classify method
                    let strategy_node_type = match self.patterns.classify_method(&current_method) {
                        Some(crate::parser::function_patterns::FunctionCategory::Error) => {
                            StrategyNodeType::ErrorMessage
                        }
                        Some(crate::parser::function_patterns::FunctionCategory::Format) => {
                            StrategyNodeType::FormatString
                        }
                        Some(crate::parser::function_patterns::FunctionCategory::Log) => {
                            StrategyNodeType::LogMessage
                        }
                        Some(crate::parser::function_patterns::FunctionCategory::Debug) => {
                            StrategyNodeType::LogMessage
                        }
                        None => continue, // Skip unknown methods
                    };

                    // Apply strategy
                    let ctx = ExtractionContext::new(&text).with_function_name(&current_method);
                    if !self.strategy.should_extract(strategy_node_type, &ctx) {
                        continue;
                    }

                    let id = format!("{}_method_{}", file_path, match_idx);
                    let node_type = self.strategy.get_node_type(strategy_node_type);
                    let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
                    units.push(unit);
                    match_idx += 1;
                }
                _ => {}
            }
        }

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

        // Extract Javadoc comments
        if self.config.extract_docstrings {
            let javadoc_units = self.extract_javadoc(&root_node, content, file_path)?;
            units.extend(javadoc_units);
        }

        // Extract method strings
        if self.config.extract_strings {
            let method_units = self.extract_method_strings(&root_node, content, file_path)?;
            units.extend(method_units);
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }
}

impl ParserTrait for JavaParser {
    #[instrument(skip(self, file))]
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing Java file"
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
            "Java file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        filename.to_lowercase().ends_with(".java")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["java"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::NodeType;
    use crate::parser::filter::FilterConfig;
    use crate::parser::strategy::{ConfigBasedStrategy, ExtractionConfig};
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    fn create_test_parser() -> JavaParser {
        let config = ParserConfig::default();
        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(ExtractionConfig::default()),
        ));
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        JavaParser::new(config, strategy, filter).unwrap()
    }

    #[tokio::test]
    async fn test_java_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
/**
 * This is a Javadoc comment
 */
public class Main {
    // This is a regular comment
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
"#;

        let file = create_test_file(content, "Main.java");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("Main.java"));
        assert!(!parser.supports("main.rs"));
    }

    #[tokio::test]
    async fn test_java_javadoc_comments() {
        let parser = create_test_parser();

        let content = r#"
/**
 * Class documentation
 */
public class Test {
    /**
     * Method documentation
     */
    public void test() {}
}
"#;

        let file = create_test_file(content, "Test.java");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let javadocs: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::DocString)
            .collect();

        assert!(!javadocs.is_empty(), "Should extract Javadoc comments");
    }

    #[tokio::test]
    async fn test_java_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
public class Test {
    public void test() {}
}
"#;

        let file = create_test_file(content, "Test.java");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_java_complex_structure() {
        let parser = create_test_parser();

        let content = r#"
package com.example;

import java.util.*;

/**
 * Main application class
 * This is a longer description
 */
public class Application {
    // Logger instance
    private static final Logger logger = Logger.getLogger(Application.class.getName());

    /**
     * Main entry point
     * @param args command line arguments
     */
    public static void main(String[] args) {
        // Print welcome message
        System.out.println("Welcome to the application");

        /* Multi-line comment
         * explaining the logic below
         */
        if (args.length > 0) {
            logger.info("Arguments provided");
        }
    }
}
"#;

        let file = create_test_file(content, "Application.java");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should extract translation units");
    }

    #[test]
    fn test_supported_extensions() {
        let parser = create_test_parser();
        assert_eq!(parser.supported_extensions(), &["java"]);
    }
}
