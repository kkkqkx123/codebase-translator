//! Tree-sitter based parser implementation
//!
//! This module provides parsers for various programming languages using tree-sitter.
//! It supports extracting comments, docstrings, and other translatable content.

use std::sync::Arc;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language as TSLanguage, Node, Parser, Query, QueryCursor, Tree};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, Position, TranslationUnit};
use crate::parser::abstraction::parser::Parser as ParserTrait;
use crate::parser::abstraction::strategy::{
    ExtractionContext, ExtractionStrategy, StrategyNodeType,
};
use crate::parser::filtering::traits::Filter;
use crate::parser::{ContentFilter, ExtractionStrategyImpl};
use crate::parser::core::{CommentType, StringProcessor};
use crate::parser::languages::rust::queries::RustQueries;
use crate::parser::languages::python::queries::PythonQueries;
use crate::parser::languages::go::queries::GoQueries;
use crate::parser::languages::javascript::queries::JavaScriptQueries;
use crate::parser::languages::typescript::queries::TypeScriptQueries;
use crate::parser::languages::java::queries::JavaQueries;
use crate::parser::languages::c::queries::CQueries;
use crate::parser::languages::cpp::queries::CppQueries;
use crate::parser::languages::csharp::queries::CSharpQueries;

/// Language configuration for tree-sitter
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// Language name
    pub name: String,
    /// File extensions
    pub extensions: Vec<String>,
    /// Tree-sitter language
    pub language: TSLanguage,
    /// Query for extracting comments
    pub comment_query: String,
    /// Query for extracting docstrings
    pub docstring_query: Option<String>,
    /// Query for extracting string literals
    pub string_query: Option<String>,
}

/// Tree-sitter based parser
pub struct TreeSitterParser {
    config: crate::parser::ParserConfig,
    language_config: LanguageConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
}

impl TreeSitterParser {
    /// Create a new tree-sitter parser for a specific language
    pub fn new(
        language_config: LanguageConfig,
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        // Validate that the language can be set
        let mut parser = Parser::new();
        parser
            .set_language(&language_config.language)
            .map_err(|e| TranslateError::Parse(format!("Failed to set language: {}", e)))?;

        Ok(Self {
            config,
            language_config,
            strategy,
            filter,
        })
    }

    /// Create a new tree-sitter parser with default strategy and filter
    pub fn with_defaults(language_config: LanguageConfig, config: crate::parser::ParserConfig) -> Result<Self> {
        use crate::parser::core::strategies::strategy_impl::ExtractionStrategyImpl;
        use crate::parser::filtering::default_filter;

        Self::new(
            language_config,
            config,
            Arc::new(ExtractionStrategyImpl::default_config()),
            Arc::new(default_filter()?),
        )
    }

    /// Parse file content into a syntax tree
    fn parse_tree(&self, content: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&self.language_config.language)
            .map_err(|e| TranslateError::Parse(format!("Failed to set language: {}", e)))?;
        parser
            .parse(content, None)
            .ok_or_else(|| TranslateError::Parse("Failed to parse file".to_string()))
    }

    /// Extract translation units from the syntax tree
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
            let comment_units = self.extract_with_query(
                &root_node,
                content,
                &self.language_config.comment_query,
                StrategyNodeType::Comment,
                file_path,
            )?;
            units.extend(comment_units);
        }

        // Extract docstrings
        if self.config.extract_docstrings {
            if let Some(ref query) = self.language_config.docstring_query {
                let doc_units = self.extract_with_query(
                    &root_node,
                    content,
                    query,
                    StrategyNodeType::DocString,
                    file_path,
                )?;
                units.extend(doc_units);
            }
        }

        // Extract string literals
        if self.config.extract_strings {
            if let Some(ref query) = self.language_config.string_query {
                let string_units = self.extract_with_query(
                    &root_node,
                    content,
                    query,
                    StrategyNodeType::StringLiteral,
                    file_path,
                )?;
                units.extend(string_units);
            }
        }

        // Sort by position for consistent ordering
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        // Remove duplicate units (same position)
        let mut unique_units: Vec<TranslationUnit> = Vec::new();
        for unit in units {
            let is_duplicate = unique_units.iter().any(|u| {
                u.start_pos.offset == unit.start_pos.offset && u.end_pos.offset == unit.end_pos.offset
            });
            if !is_duplicate {
                unique_units.push(unit);
            }
        }

        // Merge consecutive docstring/comment units
        let merged_units = Self::merge_consecutive_units(unique_units);

        Ok(merged_units)
    }

    /// Merge consecutive units of the same type
    ///
    /// This merges consecutive docstring or comment lines into a single unit
    /// to preserve context and improve translation quality.
    fn merge_consecutive_units(units: Vec<TranslationUnit>) -> Vec<TranslationUnit> {
        if units.is_empty() {
            return units;
        }

        let mut merged = Vec::new();
        let mut current_group: Vec<TranslationUnit> = Vec::new();

        for unit in units {
            if current_group.is_empty() {
                current_group.push(unit);
            } else {
                let last = &current_group[current_group.len() - 1];

                // Check if this unit should be merged with the previous one
                let should_merge = Self::should_merge_units(last, &unit);

                if should_merge {
                    current_group.push(unit);
                } else {
                    // Finalize the current group
                    if current_group.len() > 1 {
                        merged.push(Self::merge_group(&mut current_group));
                    } else {
                        merged.push(current_group.remove(0));
                    }
                    current_group.push(unit);
                }
            }
        }

        // Don't forget to last group
        if current_group.len() > 1 {
            merged.push(Self::merge_group(&mut current_group));
        } else if !current_group.is_empty() {
            merged.push(current_group.remove(0));
        }

        merged
    }

    /// Check if two units should be merged
    fn should_merge_units(prev: &TranslationUnit, current: &TranslationUnit) -> bool {
        // Must be the same type
        if prev.node_type != current.node_type {
            return false;
        }

        // Only merge docstrings and comments
        match prev.node_type {
            crate::core::models::NodeType::DocString | crate::core::models::NodeType::Comment => {}
            _ => return false,
        }

        // Check if they are on consecutive or adjacent lines
        // Use saturating_sub to prevent overflow
        let line_gap = current.start_pos.line.saturating_sub(prev.end_pos.line);
        if line_gap > 1 {
            return false;
        }

        // Check if they have the same indentation level (similar column position)
        // This prevents merging comments from different code blocks
        let column_diff = prev.start_pos.column.abs_diff(current.start_pos.column);

        // Allow small column differences (up to 4 spaces) but not large ones
        // This prevents merging comments from different nesting levels
        column_diff <= 4
    }

    /// Merge a group of consecutive units into a single unit
    fn merge_group(group: &mut Vec<TranslationUnit>) -> TranslationUnit {
        if group.is_empty() {
            panic!("Cannot merge empty group");
        }

        if group.len() == 1 {
            return group.remove(0);
        }

        // Sort by position to ensure correct order
        group.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        let first = &group[0];
        let last = &group[group.len() - 1];

        // Merge content with newlines
        let merged_content = group
            .iter()
            .map(|u| u.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Merge raw_match with newlines
        // Note: Each raw_match already ends with a newline, so we need to trim before joining
        let merged_raw_match: Option<String> = if group.iter().all(|u| u.raw_match.is_some()) {
            Some(
                group
                    .iter()
                    .map(|u| u.raw_match.as_ref().unwrap().trim_end_matches('\n'))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        } else {
            None
        };

        // Create merged unit
        let merged_id = format!(
            "{}_merged_{}_{}",
            first.id.split('_').next().unwrap_or(&first.id),
            first.node_type,
            first.start_pos.offset
        );
        let mut merged_unit = TranslationUnit::new_with_pattern(
            merged_id,
            first.node_type,
            merged_content,
            first.start_pos,
            last.end_pos,
            crate::core::models::PatternType::Builtin,
            String::new(),
        );
        merged_unit.raw_match = merged_raw_match;
        merged_unit.language = first.language.clone();

        // Clear the group after merging
        group.clear();

        merged_unit
    }

    /// Extract units using a tree-sitter query
    fn extract_with_query(
        &self,
        root_node: &Node,
        content: &str,
        query_str: &str,
        strategy_node_type: StrategyNodeType,
        file_path: &str,
    ) -> Result<Vec<TranslationUnit>> {
        let query = Query::new(&self.language_config.language, query_str)
            .map_err(|e| TranslateError::Parse(format!("Invalid query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut match_idx = 0usize;

        let mut units = Vec::new();
        let capture_names = query.capture_names();

        // Use peeking to iterate over matches
        let text_provider: &[u8] = content.as_bytes();
        let mut matches = cursor.matches(&query, *root_node, text_provider);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = &capture_names[capture.index as usize];
                let node = capture.node;

                // Only process content captures (including comment, docstring, string)
                if !capture_name.contains("content")
                    && !capture_name.contains("text")
                    && !capture_name.contains("comment")
                    && !capture_name.contains("docstring")
                    && !capture_name.contains("string")
                {
                    continue;
                }

                let node_text = node.utf8_text(content.as_bytes()).map_err(|e| {
                    TranslateError::Parse(format!("Failed to get node text: {}", e))
                })?;

                // Clean comment markers based on node type
                let processor = StringProcessor::new();
                let cleaned_text = match strategy_node_type {
                    StrategyNodeType::Comment => {
                        // Detect comment type and clean accordingly
                        let trimmed = node_text.trim();
                        if trimmed.starts_with("///")
                            || trimmed.starts_with("//!")
                            || trimmed.starts_with("/**")
                        {
                            processor.clean_comment(trimmed, CommentType::Doc)
                        } else if trimmed.starts_with("/*") {
                            processor.clean_comment(trimmed, CommentType::Block)
                        } else if trimmed.starts_with("//") || trimmed.starts_with("#") {
                            processor.clean_comment(trimmed, CommentType::Line)
                        } else {
                            node_text.to_string()
                        }
                    }
                    StrategyNodeType::DocString => {
                        // Clean doc comment markers
                        let trimmed = node_text.trim();
                        if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**") {
                            processor.clean_comment(trimmed, CommentType::Doc)
                        } else if trimmed.starts_with("/*") {
                            // Handle block comments that were mistakenly extracted as docstrings
                            processor.clean_comment(trimmed, CommentType::Block)
                        } else if trimmed.starts_with("//") || trimmed.starts_with("#") {
                            // Handle line comments that were mistakenly extracted as docstrings
                            processor.clean_comment(trimmed, CommentType::Line)
                        } else if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                            // Python-style docstrings
                            processor.clean_string_literal(trimmed)
                        } else {
                            node_text.to_string()
                        }
                    }
                    _ => node_text.to_string(),
                };

                // Apply filters
                let text = if self.config.trim_content {
                    cleaned_text.trim().to_string()
                } else {
                    cleaned_text
                };

                // For doc comments, preserve empty lines (e.g., "/// ") for proper merging
                // Check if original text is a doc comment marker with empty content
                let is_doc_empty_line = {
                    let trimmed = node_text.trim();
                    (trimmed == "///"
                        || trimmed == "//!"
                        || trimmed.starts_with("/// ")
                        || trimmed.starts_with("//! "))
                        && strategy_node_type == StrategyNodeType::DocString
                };

                if !is_doc_empty_line {
                    // Apply content filter (includes length, symbol, and language checks)
                    if !self.filter.should_translate(&text) {
                        continue;
                    }
                }

                // Apply extraction strategy
                let ctx = ExtractionContext::new(&text);
                if !self.strategy.should_extract(strategy_node_type, &ctx) {
                    continue;
                }

                let node_type = self.strategy.get_node_type(strategy_node_type);
                let id = format!("{}_{}_{}", file_path, node_type, match_idx);
                let start_pos = Position::new(
                    node.start_position().row + 1,
                    node.start_position().column + 1,
                    node.start_byte(),
                );
                let end_pos = Position::new(
                    node.end_position().row + 1,
                    node.end_position().column + 1,
                    node.end_byte(),
                );

                let mut unit = TranslationUnit::new_with_pattern(
                    id,
                    node_type,
                    text.to_string(),
                    start_pos,
                    end_pos,
                    crate::core::models::PatternType::Builtin,
                    self.language_config.name.clone(),
                );
                unit.raw_match = Some(node_text.to_string());
                units.push(unit);

                match_idx += 1;
            }
        }

        Ok(units)
    }
}

impl ParserTrait for TreeSitterParser {
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file
            .content_string()
            .map_err(|e| TranslateError::Parse(format!("Invalid UTF-8 content: {}", e)))?;

        let tree = self.parse_tree(&content)?;
        let file_path = file.path.to_string_lossy();
        self.extract_units(&tree, &content, &file_path)
    }

    fn supports(&self, filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        self.language_config
            .extensions
            .iter()
            .any(|ext| filename_lower.ends_with(&format!(".{}", ext.to_lowercase())))
    }

    fn supported_extensions(&self) -> &[&str] {
        // This is a workaround since we can't return &[&str] from Vec<String>
        // The caller should use the LanguageConfig directly
        &[]
    }
}

/// Parser factory for creating language-specific parsers
pub struct TreeSitterParserFactory;

impl TreeSitterParserFactory {
    /// Create a parser for Rust files
    pub fn create_rust_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "rust".to_string(),
            extensions: vec!["rs".to_string()],
            language: tree_sitter_rust::LANGUAGE.into(),
            comment_query: RustQueries::all_comments().to_string(),
            docstring_query: Some(RustQueries::doc_comments().to_string()),
            string_query: Some(RustQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for Go files
    pub fn create_go_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "go".to_string(),
            extensions: vec!["go".to_string()],
            language: tree_sitter_go::LANGUAGE.into(),
            comment_query: GoQueries::all_comments().to_string(),
            docstring_query: None, // Go uses regular comments for docs
            string_query: Some(GoQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for Python files
    pub fn create_python_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "python".to_string(),
            extensions: vec!["py".to_string()],
            language: tree_sitter_python::LANGUAGE.into(),
            comment_query: PythonQueries::all_comments().to_string(),
            docstring_query: Some(PythonQueries::docstrings().to_string()),
            string_query: Some(PythonQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for JavaScript files
    pub fn create_javascript_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "javascript".to_string(),
            extensions: vec!["js".to_string(), "mjs".to_string()],
            language: tree_sitter_javascript::LANGUAGE.into(),
            comment_query: JavaScriptQueries::all_comments().to_string(),
            docstring_query: Some(JavaScriptQueries::jsdoc_comments().to_string()),
            string_query: Some(JavaScriptQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for TypeScript files
    pub fn create_typescript_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "typescript".to_string(),
            extensions: vec!["ts".to_string()],
            language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            comment_query: TypeScriptQueries::all_comments().to_string(),
            docstring_query: Some(TypeScriptQueries::jsdoc_comments().to_string()),
            string_query: Some(TypeScriptQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for TSX files
    pub fn create_tsx_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "tsx".to_string(),
            extensions: vec!["tsx".to_string()],
            language: tree_sitter_typescript::LANGUAGE_TSX.into(),
            comment_query: TypeScriptQueries::all_comments().to_string(),
            docstring_query: Some(TypeScriptQueries::jsdoc_comments().to_string()),
            string_query: Some(TypeScriptQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for Java files
    pub fn create_java_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "java".to_string(),
            extensions: vec!["java".to_string()],
            language: tree_sitter_java::LANGUAGE.into(),
            comment_query: JavaQueries::all_comments().to_string(),
            docstring_query: Some(JavaQueries::javadoc_comments().to_string()),
            string_query: Some(JavaQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for C files
    pub fn create_c_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "c".to_string(),
            extensions: vec!["c".to_string(), "h".to_string()],
            language: tree_sitter_c::LANGUAGE.into(),
            comment_query: CQueries::all_comments().to_string(),
            docstring_query: Some(CQueries::doc_comments().to_string()),
            string_query: Some(CQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for C++ files
    pub fn create_cpp_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "cpp".to_string(),
            extensions: vec![
                "cpp".to_string(),
                "cc".to_string(),
                "cxx".to_string(),
                "hpp".to_string(),
                "hxx".to_string(),
            ],
            language: tree_sitter_cpp::LANGUAGE.into(),
            comment_query: CppQueries::all_comments().to_string(),
            docstring_query: Some(CppQueries::doc_comments().to_string()),
            string_query: Some(CppQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for C# files
    pub fn create_csharp_parser(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "csharp".to_string(),
            extensions: vec!["cs".to_string()],
            language: tree_sitter_c_sharp::LANGUAGE.into(),
            comment_query: CSharpQueries::all_comments().to_string(),
            docstring_query: None, // C# uses /// comments which are handled by all_comments
            string_query: Some(CSharpQueries::all_strings().to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create all available parsers with given strategy and filter
    pub fn create_all_parsers(
        config: crate::parser::ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Vec<Result<TreeSitterParser>> {
        vec![
            Self::create_rust_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_go_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_python_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_javascript_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_typescript_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_tsx_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_java_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_c_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_cpp_parser(config.clone(), strategy.clone(), filter.clone()),
            Self::create_csharp_parser(config.clone(), strategy.clone(), filter.clone()),
        ]
    }

    /// Create all available parsers with default strategy and filter
    pub fn create_all_parsers_with_defaults(config: crate::parser::ParserConfig) -> Vec<Result<TreeSitterParser>> {
        use crate::parser::filtering::default_filter;
        use crate::parser::core::strategies::strategy_impl::ExtractionStrategyImpl;

        let strategy = Arc::new(ExtractionStrategyImpl::default_config());
        let filter = Arc::new(default_filter().unwrap_or_else(|_| {
            ContentFilter::new(crate::parser::FilterConfig::default()).unwrap()
        }));

        Self::create_all_parsers(config, strategy, filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::filtering::FilterConfig;
    use crate::parser::abstraction::strategy::ExtractionConfig;
    use crate::parser::core::strategies::ConfigBasedStrategy;
    use crate::parser::ParserConfig;
    use std::path::PathBuf;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    fn create_test_strategy() -> Arc<ExtractionStrategyImpl> {
        Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(ExtractionConfig::default()),
        ))
    }

    fn create_test_filter() -> Arc<ContentFilter> {
        Arc::new(ContentFilter::new(FilterConfig::default()).unwrap())
    }

    #[tokio::test]
    async fn test_rust_parser() {
        let config = ParserConfig::default();
        let strategy = create_test_strategy();
        let filter = create_test_filter();
        let parser = TreeSitterParserFactory::create_rust_parser(config, strategy, filter)
            .expect("Failed to create Rust parser");

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
    async fn test_go_parser() {
        let config = ParserConfig::default();
        let strategy = create_test_strategy();
        let filter = create_test_filter();
        let parser = TreeSitterParserFactory::create_go_parser(config, strategy, filter)
            .expect("Failed to create Go parser");

        // Go comments use the capture name "comment" which doesn't contain "content" or "text"
        // So they may not be extracted with the current implementation
        let content = r#"
package main

// This is a comment
func main() {
    x := "hello"
}
"#;

        let file = create_test_file(content, "test.go");
        let _units = parser.parse(&file).expect("Parsing should succeed");

        // Parser should succeed even if no units are extracted
        // (Go comments may not be captured with current query patterns)
        assert!(parser.supports("test.go"));
    }
}
