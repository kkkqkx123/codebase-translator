//! C# language parser implementation
//!
//! This parser extracts translatable content from C# source files.

use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::core::traits::{
    ExtractionConfig, Parser as ParserTrait, StrategyNodeType,
};
use crate::parser::filtering::traits::Filter;
use crate::parser::{ContentFilter, FunctionCategory};
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::{CommentType, StringProcessor};
use crate::parser::ParserConfig;
use crate::parser::languages::csharp::patterns::CSharpPatterns;
use crate::parser::languages::csharp::queries::CSharpQueries;
use tracing::{debug, error, info, instrument};

/// C# language parser
pub struct CSharpParser {
    config: ParserConfig,
    extraction_config: ExtractionConfig,
    filter: Arc<ContentFilter>,
    patterns: CSharpPatterns,
    string_processor: StringProcessor,
}

impl CSharpParser {
    /// Create a new C# parser
    pub fn new(
        config: ParserConfig,
        extraction_config: ExtractionConfig,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            extraction_config,
            filter,
            patterns: CSharpPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing C# comment markers
    fn clean_comment_text(&self, text: &str) -> String {
        let trimmed = text.trim();

        // Handle XML doc comments: ///
        if trimmed.starts_with("///") {
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
            .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set C# language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse C# syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "C# syntax tree parsed successfully"
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
            &tree_sitter_c_sharp::LANGUAGE.into(),
            CSharpQueries::all_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean comment markers (//, /* */, ///)
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
            let node_type = self.extraction_config.get_node_type(StrategyNodeType::Comment);
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
            &tree_sitter_c_sharp::LANGUAGE.into(),
            CSharpQueries::all_strings(),
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

            // Apply extraction config
            if !self
                .extraction_config
                .should_extract(StrategyNodeType::FormatString)
            {
                continue;
            }

            let id = format!("{}_string_{}", file_path, match_idx);
            let node_type = self.extraction_config.get_node_type(StrategyNodeType::FormatString);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract method call strings using the core framework
    fn extract_method_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_c_sharp::LANGUAGE.into(),
            CSharpQueries::method_strings(),
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
                        Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                        Some(FunctionCategory::Format) => StrategyNodeType::FormatString,
                        Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
                        Some(FunctionCategory::Debug) => StrategyNodeType::LogMessage,
                        _ => continue, // Skip unknown methods
                    };

                    // Apply extraction config
                    if !self.extraction_config.should_extract(strategy_node_type) {
                        continue;
                    }

                    let id = format!("{}_method_{}", file_path, match_idx);
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

    /// Extract throw statement strings
    fn extract_throw_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_c_sharp::LANGUAGE.into(),
            CSharpQueries::throw_statements(),
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

            // Apply extraction config
            if !self
                .extraction_config
                .should_extract(StrategyNodeType::ErrorMessage)
            {
                continue;
            }

            let id = format!("{}_throw_{}", file_path, match_idx);
            let node_type = self.extraction_config.get_node_type(StrategyNodeType::ErrorMessage);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract attribute strings
    fn extract_attribute_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_c_sharp::LANGUAGE.into(),
            CSharpQueries::doc_attributes(),
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

            // Apply extraction config
            if !self
                .extraction_config
                .should_extract(StrategyNodeType::DocString)
            {
                continue;
            }

            let id = format!("{}_attr_{}", file_path, match_idx);
            let node_type = self.extraction_config.get_node_type(StrategyNodeType::DocString);
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

        // Extract string literals
        if self.config.extract_strings {
            let string_units = self.extract_strings(&root_node, content, file_path)?;
            units.extend(string_units);

            // Extract method call strings
            let method_units = self.extract_method_strings(&root_node, content, file_path)?;
            units.extend(method_units);

            // Extract throw statement strings
            let throw_units = self.extract_throw_strings(&root_node, content, file_path)?;
            units.extend(throw_units);

            // Extract attribute strings
            let attr_units = self.extract_attribute_strings(&root_node, content, file_path)?;
            units.extend(attr_units);
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }
}

impl ParserTrait for CSharpParser {
    #[instrument(skip(self, file))]
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing C# file"
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
            "C# file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".cs")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["cs"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::NodeType;
    use crate::parser::filtering::FilterConfig;
    use crate::parser::core::traits::ExtractionConfig;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    fn create_test_parser() -> CSharpParser {
        let config = ParserConfig::default();
        let extraction_config = ExtractionConfig::default();
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        CSharpParser::new(config, extraction_config, filter).unwrap()
    }

    fn create_test_parser_with_strings() -> CSharpParser {
        let config = ParserConfig {
            extract_strings: true,
            ..Default::default()
        };
        let extraction_config = ExtractionConfig {
            format_strings: true,
            ..Default::default()
        };
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        CSharpParser::new(config, extraction_config, filter).unwrap()
    }

    #[tokio::test]
    async fn test_csharp_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
/* This is a block comment */
class Program {
    // This is a line comment
    static void Main() {
        int x = 5;
    }
}
"#;

        let file = create_test_file(content, "test.cs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("test.cs"));
        assert!(!parser.supports("test.rs"));
        assert!(!parser.supports("test.cpp"));
    }

    #[tokio::test]
    async fn test_csharp_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
class Test {}
"#;

        let file = create_test_file(content, "test.cs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_csharp_strings() {
        let parser = create_test_parser_with_strings();

        let content = r#"
class Test {
    void Method() {
        string msg = "Hello, world!";
        string verbatim = @"This is verbatim";
    }
}
"#;

        let file = create_test_file(content, "test.cs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let strings: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::FormatString)
            .collect();

        assert!(!strings.is_empty(), "Should extract strings");
    }

    #[tokio::test]
    async fn test_csharp_method_calls() {
        let parser = create_test_parser_with_strings();

        let content = r#"
class Test {
    void Method() {
        Console.WriteLine("Hello, world!");
        Debug.Log("Debug message");
    }
}
"#;

        let file = create_test_file(content, "test.cs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let log_msgs: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::LogMessage)
            .collect();

        assert!(!log_msgs.is_empty(), "Should extract log messages");
    }

    #[tokio::test]
    async fn test_csharp_throw() {
        let parser = create_test_parser_with_strings();

        let content = r#"
class Test {
    void Method() {
        throw new Exception("Error occurred");
    }
}
"#;

        let file = create_test_file(content, "test.cs");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let error_msgs: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::ErrorMessage)
            .collect();

        assert!(!error_msgs.is_empty(), "Should extract throw messages");
    }
}

