//! TypeScript language parser implementation
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
use crate::parser::core::{CommentType, StringProcessor};
use crate::parser::engine::ParserConfig;
use crate::parser::languages::typescript::patterns::TypeScriptPatterns;
use crate::parser::languages::typescript::queries::TypeScriptQueries;
use tracing::{debug, error, info, instrument};

/// TypeScript language parser
pub struct TypeScriptParser {
    config: ParserConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
    patterns: TypeScriptPatterns,
    string_processor: StringProcessor,
}

impl TypeScriptParser {
    /// Create a new TypeScript parser
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            strategy,
            filter,
            patterns: TypeScriptPatterns::new(),
            string_processor: StringProcessor::new(),
        })
    }

    /// Clean comment text by removing TypeScript comment markers
    fn clean_comment_text(&self, text: &str) -> String {
        let trimmed = text.trim();

        // Handle JSDoc comments: /**
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
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| {
                error!(error = %e, "Failed to set TypeScript language");
                TranslateError::Parse(format!("Failed to set language: {}", e))
            })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            error!("Failed to parse TypeScript syntax tree");
            TranslateError::Parse("Failed to parse file".to_string())
        })?;

        debug!(
            root_node = tree.root_node().kind(),
            "TypeScript syntax tree parsed successfully"
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
            &tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            TypeScriptQueries::all_comments(),
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

    /// Extract JSDoc comments using the core framework
    fn extract_jsdoc(
        &self,
        root_node: &Node,
        content: &str,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let executor = QueryExecutor::from_string(
            &tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            TypeScriptQueries::jsdoc_comments(),
        )?;

        let matches = executor.execute(root_node, content)?;
        let mut units = Vec::new();
        let mut match_idx = 0usize;

        for m in matches {
            // Clean JSDoc markers (/** */)
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

            let id = format!("{}_jsdoc_{}", file_path, match_idx);
            let node_type = self.strategy.get_node_type(StrategyNodeType::DocString);
            let unit = TranslationUnit::new(id, node_type, text, m.start_pos, m.end_pos);
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
            &tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            TypeScriptQueries::call_strings(),
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
                        Some(FunctionCategory::Error) => StrategyNodeType::ErrorMessage,
                        Some(FunctionCategory::Log) => StrategyNodeType::LogMessage,
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
            &tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            TypeScriptQueries::template_strings(),
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

impl ParserTrait for TypeScriptParser {
    #[instrument(skip(self, file))]
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let start = std::time::Instant::now();
        let file_path = file.path.to_string_lossy().to_string();

        info!(
            file = %file_path,
            size = file.content.len(),
            "Parsing TypeScript file"
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
            "TypeScript file parsed successfully"
        );

        Ok(units)
    }

    fn supports(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".ts")
            || lower.ends_with(".tsx")
            || lower.ends_with(".mts")
            || lower.ends_with(".cts")
    }

    fn supported_extensions(&self) -> &[&str] {
        &["ts", "tsx", "mts", "cts"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::NodeType;
    use crate::parser::filtering::FilterConfig;
    use crate::parser::abstraction::strategy::{ConfigBasedStrategy, ExtractionConfig};
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    fn create_test_parser() -> TypeScriptParser {
        let config = ParserConfig::default();
        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(ExtractionConfig::default()),
        ));
        let filter = Arc::new(ContentFilter::new(FilterConfig::default()).unwrap());

        TypeScriptParser::new(config, strategy, filter).unwrap()
    }

    #[tokio::test]
    async fn test_typescript_parser_basic() {
        let parser = create_test_parser();

        let content = r#"
/**
 * This is a JSDoc comment
 */
function main(): void {
    // This is a regular comment
    console.log("Hello, World!");
}
"#;

        let file = create_test_file(content, "main.ts");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty());
        assert!(parser.supports("main.ts"));
        assert!(parser.supports("main.tsx"));
        assert!(parser.supports("main.mts"));
        assert!(parser.supports("main.cts"));
        assert!(!parser.supports("main.js"));
    }

    #[tokio::test]
    async fn test_typescript_jsdoc_comments() {
        let parser = create_test_parser();

        let content = r#"
/**
 * Function documentation
 * @param name - The name parameter
 * @returns The greeting message
 */
function greet(name: string): string {
    return `Hello, ${name}`;
}
"#;

        let file = create_test_file(content, "greet.ts");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let jsdocs: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::DocString)
            .collect();

        assert!(!jsdocs.is_empty(), "Should extract JSDoc comments");
    }

    #[tokio::test]
    async fn test_typescript_comments() {
        let parser = create_test_parser();

        let content = r#"
// Line comment
/* Block comment */
function test(): void {}
"#;

        let file = create_test_file(content, "test.ts");
        let units = parser.parse(&file).expect("Parsing should succeed");

        let comments: Vec<_> = units
            .iter()
            .filter(|u| u.node_type == NodeType::Comment)
            .collect();

        assert!(!comments.is_empty(), "Should extract comments");
    }

    #[tokio::test]
    async fn test_typescript_complex_structure() {
        let parser = create_test_parser();

        let content = r#"
/**
 * Main application interface
 */
interface AppConfig {
    name: string;
    version: string;
    debug?: boolean;
}

// Configuration
const config: AppConfig = {
    name: "My App",
    version: "1.0.0"
};

/**
 * Initialize the application
 */
function init(): void {
    console.log("Initializing application...");
    
    /* Multi-line comment
     * explaining the logic
     */
    if (config.debug) {
        console.warn("Debug mode is enabled");
    }
}

export { init, config };
"#;

        let file = create_test_file(content, "app.ts");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(!units.is_empty(), "Should extract translation units");
    }

    #[tokio::test]
    async fn test_typescript_tsx() {
        let parser = create_test_parser();

        let content = r#"
import React from 'react';

/**
 * Props for the Button component
 */
interface ButtonProps {
    label: string;
    onClick: () => void;
}

// Button component
export const Button: React.FC<ButtonProps> = ({ label, onClick }) => {
    return (
        <button onClick={onClick}>
            {label}
        </button>
    );
};
"#;

        let file = create_test_file(content, "Button.tsx");
        let units = parser.parse(&file).expect("Parsing should succeed");

        assert!(
            !units.is_empty(),
            "Should extract translation units from TSX"
        );
    }

    #[test]
    fn test_supported_extensions() {
        let parser = create_test_parser();
        assert_eq!(parser.supported_extensions(), &["ts", "tsx", "mts", "cts"]);
    }
}

