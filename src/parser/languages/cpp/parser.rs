//! C++ language parser implementation
//!
//! This parser extracts translatable content from C++ source files.

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
use crate::parser::core::{CommentType, StringProcessor};
use crate::parser::ParserConfig;
use crate::parser::languages::cpp::patterns::CppPatterns;
use crate::parser::languages::cpp::queries::CppQueries;
use tracing::{debug, error, info, instrument};

/// C++ language parser
pub struct CppParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
    patterns: CppPatterns,
    string_processor: StringProcessor,
}

impl CppParser {
    /// Create a new C++ parser
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            strategy,
            filter,
            patterns: CppPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing C++ comment markers
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
    #[instrument(skip(self, content))]
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_cpp::LANGUAGE.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set C++ language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse C++ syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "C++ syntax tree parsed successfully"
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
            &tree_sitter_cpp::LANGUAGE.into(),
            CppQueries::all_comments(),
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

    /// Extract documentation comments using the core framework
    fn extract_docstrings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_cpp::LANGUAGE.into(),
            CppQueries::doc_comments(),
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
                .should_extract(StrategyNodeType::DocString, &ctx)
            {
                continue;
            }

            let id = format!("{}_docstring_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::DocString);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract string literals using the core framework
    fn extract_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_cpp::LANGUAGE.into(),
            CppQueries::all_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean the string literal
            let text = self.string_processor.clean_string_literal(m.text);

            // Apply filter
            if !self.filter.should_translate(&text) {
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(&text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::FormatString, &ctx)
            {
                continue;
            }

            let id = format!("{}_string_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::FormatString);
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
            &tree_sitter_cpp::LANGUAGE.into(),
            CppQueries::function_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        // Group matches by function call
        let mut current_func = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                "func_name" => {
                    current_func = m.text.to_string();
                }
                "func_string" => {
                    if current_func.is_empty() {
                        continue;
                    }

                    // Clean the string literal
                    let text = self.string_processor.clean_string_literal(m.text);

                    // Apply filter
                    if !self.filter.should_translate(&text) {
                        continue;
                    }

                    // Classify function
                    let strategy_node_type = match self.patterns.classify_function(&current_func) {
                        Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                        Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                        Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                        Some(FunctionCategory::Debug) => StrategyNodeType::LogMessage,
                        _ => continue, // Skip unknown functions
                    };

                    // Apply strategy
                    let ctx = ExtractionContext::new(&text).with_function_name(&current_func);
                    if !self.strategy.should_extract(strategy_node_type, &ctx) {
                        continue;
                    }

                    let id = format!("{}_func_{}", file_path, match_idx);
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

    /// Extract throw statement strings
    fn extract_throw_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_cpp::LANGUAGE.into(),
            CppQueries::throw_statements(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean the string literal
            let text = self.string_processor.clean_string_literal(m.text);

            // Apply filter
            if !self.filter.should_translate(&text) {
                continue;
            }

            // Apply strategy
            let ctx = ExtractionContext::new(&text).with_function_name("throw");
            if !self
                .strategy
                .should_extract(StrategyNodeType::ErrorMessage, &ctx)
            {
                continue;
            }

            let id = format!("{}_throw_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::ErrorMessage);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
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

        // Extract doc comments
        if self.config.extract_docstrings {
            let doc_units = self.extract_docstrings(&root_node, content, file_path)?;
            units.extend(doc_units);
        }

        // Extract string literals
        if self.config.extract_strings {
            let string_units = self.extract_strings(&root_node, content, file_path)?;
            units.extend(string_units);

            // Extract function call strings
            let func_units = self.extract_function_strings(&root_node, content, file_path)?;
            units.extend(func_units);

            // Extract throw statement strings
            let throw_units = self.extract_throw_strings(&root_node, content, file_path)?;
            units.extend(throw_units);
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }
}

impl ParserTrait for CppParser {
    #[instrument(skip(self, file))]
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing C++ file"
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
            "C++ file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".cpp")
            || lower.ends_with(".cc")
            || lower.ends_with(".cxx")
            || lower.ends_with(".hpp")
            || lower.ends_with(".hxx")
            || lower.ends_with(".h++")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["cpp", "cc", "cxx", "hpp", "hxx", "h++"]
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

    fn create_test_parser() -> CppParser {
        let config = ParserConfig::default();
        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(ExtractionConfig::default()),
        ));
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        CppParser::new(config, strategy, filter).unwrap()
    }

    fn create_test_parser_with_strings() -> CppParser {
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

        CppParser::new(config, strategy, filter).unwrap()
    }

    #[tokio::test]
    async fn test_cpp_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
/* This is a block comment */
void main() {
    // This is a line comment
    int x = 5;
}
"#;

        let file = create_test_file(content, "test.cpp");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.cpp"));
        assert!(parser.supports("test.cc"));
        assert!(parser.supports("test.hpp"));
        assert!(!parser.supports("test.c"));
    }

    #[tokio::test]
    async fn test_cpp_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
void test() {}
"#;

        let file = create_test_file(content, "test.cpp");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_cpp_strings() {
        let parser = create_test_parser_with_strings();

        let content = r#"
void test() {
    std::cout << "Hello, world!";
    std::string msg = "This is a message";
}
"#;

        let file = create_test_file(content, "test.cpp");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let strings: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::FormatString)
            .collect();

        assert!(!strings.is_empty(), "Should extract strings");
    }

    #[tokio::test]
    async fn test_cpp_throw() {
        let parser = create_test_parser_with_strings();

        let content = r#"
void test() {
    throw "Error occurred";
}
"#;

        let file = create_test_file(content, "test.cpp");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let error_msgs: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::ErrorMessage)
            .collect();

        assert!(!error_msgs.is_empty(), "Should extract throw messages");
    }
}

