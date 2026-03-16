//! Rust language parser implementation
//!
//! This is a refactored version using the core extraction framework.

use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::StringProcessor;
use crate::parser::filter::ContentFilter;
use crate::parser::languages::rust::patterns::RustPatterns;
use crate::parser::languages::rust::queries::RustQueries;
use crate::parser::strategy::{
    ExtractionContext, ExtractionStrategy, ExtractionStrategyImpl, StrategyNodeType,
};
use crate::parser::tree_sitter::ParserConfig;
use crate::parser::Parser as ParserTrait;

/// Rust language parser
pub struct RustParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
    patterns: RustPatterns,
    string_processor: StringProcessor,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            strategy,
            filter,
            patterns: RustPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Parse file content into a syntax tree
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| TranslateError::Parse(format!("Failed to set language: {}", e)))?;
        parser
            .parse(content, None)
            .ok_or_else(|| TranslateError::Parse("Failed to parse file".to_string()))
    }

    /// Extract comments using the core framework
    fn extract_comments(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_rust::LANGUAGE.into(),
            RustQueries::all_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            let text = if self.config.trim_content {
                m.text.trim()
            } else {
                m.text
            };

            // Apply length filters
            if text.len() < self.config.min_content_length {
                continue;
            }
            if text.len() > self.config.max_content_length {
                continue;
            }

            // Skip if only symbols
            if self.string_processor.is_only_symbols(text) {
                continue;
            }

            // Apply content filter
            if !self.filter.should_translate(text) {
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::Comment, &ctx)
            {
                continue;
            }

            let id = format!("{}_comment_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::Comment);
            let unit =
                TranslationUnit::new(id, node_type, text.to_string(), m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract docstrings using the core framework
    fn extract_docstrings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_rust::LANGUAGE.into(),
            RustQueries::doc_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            let text = if self.config.trim_content {
                m.text.trim()
            } else {
                m.text
            };

            // Apply length filters
            if text.len() < self.config.min_content_length {
                continue;
            }
            if text.len() > self.config.max_content_length {
                continue;
            }

            // Skip if only symbols
            if self.string_processor.is_only_symbols(text) {
                continue;
            }

            // Apply content filter
            if !self.filter.should_translate(text) {
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::DocString, &ctx)
            {
                continue;
            }

            let id = format!("{}_docstring_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::DocString);
            let unit =
                TranslationUnit::new(id, node_type, text.to_string(), m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract macro strings using the core framework
    fn extract_macro_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_rust::LANGUAGE.into(),
            RustQueries::macro_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        // Group matches by macro invocation
        let mut current_macro = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                "macro_name" => {
                    current_macro = m.text.to_string();
                }
                "macro_string" => {
                    if current_macro.is_empty() {
                        continue;
                    }

                    // Clean the string literal
                    let text = self.string_processor.clean_string_literal(m.text);

                    // Apply filter
                    if !self.filter.should_translate(&text) {
                        continue;
                    }

                    // Classify macro
                    let strategy_node_type = match self.patterns.classify_macro(&current_macro) {
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
                        None => continue, // Skip unknown macros
                    };

                    // Apply strategy
                    let ctx = ExtractionContext::new(&text).with_function_name(&current_macro);
                    if !self.strategy.should_extract(strategy_node_type, &ctx) {
                        continue;
                    }

                    let id = format!("{}_macro_{}", file_path, match_idx);
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

        // Extract docstrings
        if self.config.extract_docstrings {
            let doc_units = self.extract_docstrings(&root_node, content, file_path)?;
            units.extend(doc_units);
        }

        // Extract macro strings
        if self.config.extract_strings {
            let macro_units = self.extract_macro_strings(&root_node, content, file_path)?;
            units.extend(macro_units);
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }
}

impl ParserTrait for RustParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file
            .content_string()
            .map_err(|e| TranslateError::Parse(format!("Invalid UTF-8 content: {}", e)))?;

        let tree = self.parse_tree(&content)?;
        let file_path = file.path.to_string_lossy().to_string();

        self.extract_units(&tree, &content, &file_path)
    }

    fn supports(&self, filename: &str) -> bool {
        filename.to_lowercase().ends_with(".rs")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs"]
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

    fn create_test_parser() -> RustParser {
        let config = ParserConfig::default();
        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(ExtractionConfig::default()),
        ));
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        RustParser::new(config, strategy, filter).unwrap()
    }

    #[tokio::test]
    async fn test_rust_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
/// This is a doc comment
fn main() {
    // This is a regular comment
    let x = 5;
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.rs"));
        assert!(!parser.supports("test.go"));
    }

    #[tokio::test]
    async fn test_rust_doc_comments() {
        let parser = create_test_parser();

        let content = r#"
/// Outer doc comment
//! Inner doc comment
fn test() {}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let docstrings: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::DocString)
            .collect();

        assert!(!docstrings.is_empty(), "Should extract docstrings");
    }

    #[tokio::test]
    async fn test_rust_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
fn test() {}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_rust_complex_structure() {
        let parser = create_test_parser();

        let content = r#"
//! Crate documentation

/// Module documentation
pub mod module_a {
    //! Inner documentation
    
    /// Function documentation
    pub fn test_func() {
        // Implementation comment
    }
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(
            !units.is_empty(),
            "Should extract units from complex structure"
        );

        // Verify position tracking
        for i in 1..units.len() {
            assert!(
                units[i].start_pos.offset >= units[i - 1].start_pos.offset,
                "Units should be sorted by offset"
            );
        }
    }

    #[test]
    fn test_string_processor_integration() {
        let parser = create_test_parser();

        // Test string cleaning through the parser's processor
        let raw = r##"r#"hello "world""#"##;
        let cleaned = parser.string_processor.clean_string_literal(raw);
        assert_eq!(cleaned, r#"hello "world""#);
    }
}
