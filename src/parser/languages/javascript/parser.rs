//! JavaScript language parser implementation
//!
//! This is a refactored version using the core extraction framework.

use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::core::query_executor::QueryExecutor;
use crate::parser::core::string_processor::{CleanedComment, CommentType};
use crate::parser::core::StringProcessor;
use crate::parser::filter::ContentFilter;
use crate::parser::languages::javascript::patterns::JavaScriptPatterns;
use crate::parser::languages::javascript::queries::JavaScriptQueries;
use crate::parser::strategy::{
    ExtractionContext, ExtractionStrategy, ExtractionStrategyImpl, StrategyNodeType,
};
use crate::parser::tree_sitter::ParserConfig;
use crate::parser::Parser as ParserTrait;
use tracing::{debug, error, info, instrument};

/// JavaScript language parser
pub struct JavaScriptParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
    patterns: JavaScriptPatterns,
    string_processor: StringProcessor,
}

impl JavaScriptParser {
    /// Create a new JavaScript parser
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            strategy,
            filter,
            patterns: JavaScriptPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing JavaScript comment markers
    /// Returns CleanedComment with format information for proper reconstruction.
    fn clean_comment_text(&self, text: &str) -> CleanedComment {
        let trimmed = text.trim();

        // Handle JSDoc comments: /**
        if trimmed.starts_with("/**") {
            return self
                .string_processor
                .clean_comment_with_format(trimmed, CommentType::Doc);
        }

        // Handle block comments: /*
        if trimmed.starts_with("/*") {
            return self
                .string_processor
                .clean_comment_with_format(trimmed, CommentType::Block);
        }

        // Handle line comments: //
        if trimmed.starts_with("//") {
            return self
                .string_processor
                .clean_comment_with_format(trimmed, CommentType::Line);
        }

        CleanedComment {
            text: trimmed.to_string(),
            format_info: crate::core::models::FormatInfo {
                style: crate::core::models::CommentStyle::Line,
                base_indent: String::new(),
                line_prefix: None,
                ends_with_newline: false,
                is_multiline: false,
                string_style: None,
                placeholders: None,
                quote_char: None,
            },
        }
    }

    /// Parse file content into a syntax tree
    #[instrument(skip(self, content))]
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set JavaScript language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse JavaScript syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "JavaScript syntax tree parsed successfully"
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
            &tree_sitter_javascript::LANGUAGE.into(),
            JavaScriptQueries::all_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean comment markers (//, /* */)
            let cleaned = self.clean_comment_text(m.text);

            // Apply trim if configured
            let text = if self.config.trim_content {
                cleaned.text.trim().to_string()
            } else {
                cleaned.text
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
            let mut unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            unit.format_info = Some(cleaned.format_info);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract JSDoc comments using the core framework
    fn extract_jsdoc(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_javascript::LANGUAGE.into(),
            JavaScriptQueries::jsdoc_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean JSDoc markers (/** */)
            let cleaned = self.clean_comment_text(m.text);

            // Apply trim if configured
            let text = if self.config.trim_content {
                cleaned.text.trim().to_string()
            } else {
                cleaned.text
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

            let id = format!("{}_jsdoc_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::DocString);
            let mut unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
            unit.format_info = Some(cleaned.format_info);
            units.push(unit);
            match_idx += 1;
        }

        Ok(units)
    }

    /// Extract call expression strings using the core framework
    fn extract_call_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_javascript::LANGUAGE.into(),
            JavaScriptQueries::call_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        // Group matches by call expression
        let mut current_func = String::new();

        for m in matches {
            match m.capture_name.as_str() {
                "func_name" | "method_name" => {
                    current_func = m.text.to_string();
                }
                "call_string" => {
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
                        Some(crate::parser::function_patterns::FunctionCategory::Error) => {
                            StrategyNodeType::ErrorMessage
                        }
                        Some(crate::parser::function_patterns::FunctionCategory::Log) => {
                            StrategyNodeType::LogMessage
                        }
                        _ => continue, // Skip unknown functions
                    };

                    // Apply strategy
                    let ctx = ExtractionContext::new(&text).with_function_name(&current_func);
                    if !self.strategy.should_extract(strategy_node_type, &ctx) {
                        continue;
                    }

                    let id = format!("{}_call_{}", file_path, match_idx);
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

    /// Extract template strings using the core framework
    fn extract_template_strings(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_javascript::LANGUAGE.into(),
            JavaScriptQueries::template_strings(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean the template string (remove backticks)
            let text = m
                .text
                .strip_prefix('`')
                .and_then(|s| s.strip_suffix('`'))
                .map(|s| s.to_string())
                .unwrap_or_else(|| m.text.to_string());

            // Apply filter
            if !self.filter.should_translate(&text) {
                continue;
            }

            // Template strings are treated as FormatString
            let ctx = ExtractionContext::new(&text);
            if !self
                .strategy
                .should_extract(StrategyNodeType::FormatString, &ctx)
            {
                continue;
            }

            let id = format!("{}_template_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::FormatString);
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

        // Extract JSDoc comments
        if self.config.extract_docstrings {
            let jsdoc_units = self.extract_jsdoc(&root_node, content, file_path)?;
            units.extend(jsdoc_units);
        }

        // Extract call expression strings
        if self.config.extract_strings {
            let call_units = self.extract_call_strings(&root_node, content, file_path)?;
            units.extend(call_units);

            // Extract template strings
            let template_units = self.extract_template_strings(&root_node, content, file_path)?;
            units.extend(template_units);
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }
}

impl ParserTrait for JavaScriptParser {
    #[instrument(skip(self, file))]
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing JavaScript file"
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
            "JavaScript file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".js") || lower.ends_with(".mjs") || lower.ends_with(".cjs")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["js", "mjs", "cjs"]
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

    fn create_test_parser() -> JavaScriptParser {
        let config = ParserConfig::default();
        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(ExtractionConfig::default()),
        ));
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        JavaScriptParser::new(config, strategy, filter).unwrap()
    }

    #[tokio::test]
    async fn test_javascript_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
/**
 * This is a JSDoc comment
 */
function main() {
    // This is a regular comment
    console.log("Hello, World!");
}
"#;

        let file = create_test_file(content, "main.js");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("main.js"));
        assert!(parser.supports("main.mjs"));
        assert!(parser.supports("main.cjs"));
        assert!(!parser.supports("main.ts"));
    }

    #[tokio::test]
    async fn test_javascript_jsdoc_comments() {
        let parser = create_test_parser();

        // Note: Avoiding curly braces in JSDoc to pass code pattern filter
        let content = r#"
/**
 * Function documentation
 * @param name - The name parameter
 */
function greet(name) {
    return "Hello, " + name;
}
"#;

        let file = create_test_file(content, "greet.js");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let jsdocs: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::DocString)
            .collect();

        assert!(!jsdocs.is_empty(), "Should extract JSDoc comments");
    }

    #[tokio::test]
    async fn test_javascript_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
function test() {}
"#;

        let file = create_test_file(content, "test.js");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_javascript_complex_structure() {
        let parser = create_test_parser();

        let content = r#"
/**
 * Main application module
 * @module app
 */

// Configuration
const config = {
    name: "My App",
    version: "1.0.0"
};

/**
 * Initialize the application
 */
function init() {
    console.log("Initializing application...");
    
    /* Multi-line comment
     * explaining the logic
     */
    if (config.debug) {
        console.warn("Debug mode is enabled");
    }
}

export { init };
"#;

        let file = create_test_file(content, "app.js");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should extract translation units");
    }

    #[test]
    fn test_supported_extensions() {
        let parser = create_test_parser();
        assert_eq!(parser.supported_extensions(), &["js", "mjs", "cjs"]);
    }
}
