//! Go language parser implementation
//!
//! This is a refactored version using the core extraction framework.

use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

use crate::config::project::ExtractionConfig;
use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::core::StrategyNodeType;
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::Parser as ParserTrait;
use crate::parser::core::{CommentType, StringProcessor};
use crate::parser::filtering::traits::Filter;
use crate::parser::languages::go::patterns::GoPatterns;
use crate::parser::languages::go::queries::GoQueries;
use crate::parser::ParserConfig;
use crate::parser::{ContentFilter, FunctionCategory};
use tracing::{debug, error, info};

/// Go language parser
pub struct GoParser {
    config: ParserConfig,
    extraction_config: ExtractionConfig,
    filter: Arc<ContentFilter>,
    patterns: GoPatterns,
    string_processor: StringProcessor,
}

impl GoParser {
    /// Create a new Go parser
    pub fn new(
        config: ParserConfig,
        extraction_config: ExtractionConfig,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            extraction_config,
            filter,
            patterns: GoPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing Go comment markers
    fn clean_comment_text(&self, text: &str) -> String {
        let trimmed = text.trim();

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
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set Go language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse Go syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "Go syntax tree parsed successfully"
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
            &tree_sitter_go::LANGUAGE.into(),
            GoQueries::all_comments(),
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

            // Apply extraction config
            if !self
                .extraction_config
                .should_extract(StrategyNodeType::Comment)
            {
                continue;
            }

            let id = format!("{}_comment_{}", file_path, match_idx);
            let node_type = self
                .extraction_config
                .get_node_type(StrategyNodeType::Comment);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract docstrings using the core framework
    /// In Go, doc comments are regular comments that appear before declarations
    fn extract_docstrings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_go::LANGUAGE.into(),
            GoQueries::doc_comments(),
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

            // Apply extraction config
            if !self
                .extraction_config
                .should_extract(StrategyNodeType::DocString)
            {
                continue;
            }

            let id = format!("{}_docstring_{}", file_path, match_idx);
            let node_type = self
                .extraction_config
                .get_node_type(StrategyNodeType::DocString);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract function call strings using the core framework
    fn extract_function_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_go::LANGUAGE.into(),
            GoQueries::function_call_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
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
                    // Build full function name for selector expressions
                    let full_func_name = if !current_operand.is_empty() {
                        format!("{}.{}", current_operand, current_func)
                    } else {
                        current_func.clone()
                    };

                    if full_func_name.is_empty() {
                        continue;
                    }

                    // Clean the string literal
                    let text = self.string_processor.clean_string_literal(m.text);

                    // Apply filter
                    if !self.filter.should_translate(&text) {
                        continue;
                    }

                    // Classify function
                    let strategy_node_type = match self.patterns.classify_function(&full_func_name)
                    {
                        Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                        Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                        Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                        Some(FunctionCategory::Debug) => StrategyNodeType::LogMessage,
                        Some(FunctionCategory::Test) => StrategyNodeType::TestDescription,
                        None => continue, // Skip unknown functions
                    };

                    // Apply extraction config
                    if !self.extraction_config.should_extract(strategy_node_type) {
                        continue;
                    }

                    let id = format!("{}_func_{}", file_path, match_idx);
                    let node_type = self.extraction_config.get_node_type(strategy_node_type);
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

impl ParserTrait for GoParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing Go file"
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
            "Go file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        filename.to_lowercase().ends_with(".go")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["go"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::NodeType;

    use crate::parser::core::ExtractionConfig;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    fn create_test_parser() -> GoParser {
        let config = ParserConfig {
            extract_strings: true,
            ..Default::default()
        };
        let extraction_config = ExtractionConfig {
            format_strings: true,
            ..Default::default()
        };
        let filter = Arc::new(crate::parser::filtering::test_filter().unwrap());

        GoParser::new(config, extraction_config, filter).unwrap()
    }

    #[tokio::test]
    async fn test_go_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
// This is a package comment
package main

// main function documentation
func main() {
    // This is a regular comment
    fmt.Println("Hello, World!")
}
"#;

        let file = create_test_file(content, "test.go");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.go"));
        assert!(!parser.supports("test.rs"));
    }

    #[tokio::test]
    async fn test_go_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
func test() {}
"#;

        let file = create_test_file(content, "test.go");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_go_function_calls() {
        let parser = create_test_parser();

        let content = r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
    fmt.Printf("Value: %d", 42)
    panic("Something went wrong")
}
"#;

        let file = create_test_file(content, "test.go");
        let units = parser.parse(&file).expect("Parsing should succeed");

        // Should extract strings from function calls
        assert!(!units.is_empty());
    }

    #[tokio::test]
    async fn test_go_doc_comments() {
        let parser = create_test_parser();

        let content = r#"
// Package main provides the main entry point
package main

// Person represents a person in the system
type Person struct {
    Name string
}

// Greet returns a greeting message
func (p *Person) Greet() string {
    return "Hello"
}
"#;

        let file = create_test_file(content, "test.go");
        let units = parser.parse(&file).expect("Parsing should succeed");

        // In Go, doc comments are syntactically identical to regular comments.
        // They are extracted as Comment type, not DocString.
        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(
            !comments.is_empty(),
            "Should extract comments including doc comments"
        );

        // Verify that doc comment content is present
        let comment_texts: Vec<_> = comments.iter().map(|u| u.content.as_str()).collect();
        assert!(comment_texts
            .iter()
            .any(|t| t.contains("Package main provides")));
        assert!(comment_texts
            .iter()
            .any(|t| t.contains("Person represents")));
        assert!(comment_texts.iter().any(|t| t.contains("Greet returns")));
    }

    #[tokio::test]
    async fn test_go_raw_strings() {
        let parser = create_test_parser();

        // Go raw strings use backticks
        let content = "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(`Hello, World!`)\n    fmt.Printf(`Value: %d`, 42)\n}";

        let file = create_test_file(content, "test.go");
        let units = parser.parse(&file).expect("Parsing should succeed");

        // Should handle raw string literals in function calls
        assert!(!units.is_empty());
    }

    #[tokio::test]
    async fn test_go_complex_structure() {
        let parser = create_test_parser();

        let content = r#"
// Package main is the main package
package main

import (
    "fmt"
    "log"
)

// Config holds application configuration
type Config struct {
    Debug bool
}

// NewConfig creates a new Config
func NewConfig() *Config {
    return &Config{}
}

func main() {
    // Initialize configuration
    cfg := NewConfig()
    
    if cfg.Debug {
        log.Println("Debug mode enabled")
    }
    
    fmt.Println("Application started")
}
"#;

        let file = create_test_file(content, "test.go");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should extract translation units");
    }
}
