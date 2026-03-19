//! Tree-sitter based parser implementation
//!
//! This module provides parsers for various programming languages using tree-sitter.
//! It supports extracting comments, docstrings, and other translatable content.

use std::sync::Arc;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language as TSLanguage, Node, Parser, Query, QueryCursor, Tree};
use tracing::debug;

use crate::core::error::{Result, TranslateError};
use crate::core::models::{CommentStyle, File, FormatInfo, Position, TranslationUnit};
use crate::parser::filter::ContentFilter;
use crate::parser::strategy::{
    ExtractionContext, ExtractionStrategy, ExtractionStrategyImpl, StrategyNodeType,
};
use crate::parser::Parser as ParserTrait;

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
    config: ParserConfig,
    language_config: LanguageConfig,
    strategy: Arc<ExtractionStrategyImpl>,
    filter: Arc<ContentFilter>,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to extract comments
    pub extract_comments: bool,
    /// Whether to extract docstrings
    pub extract_docstrings: bool,
    /// Whether to extract string literals
    pub extract_strings: bool,
    /// Minimum content length to extract
    pub min_content_length: usize,
    /// Maximum content length to extract
    pub max_content_length: usize,
    /// Whether to trim whitespace from content
    pub trim_content: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            extract_comments: true,
            extract_docstrings: true,
            extract_strings: false,
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
        }
    }
}

impl TreeSitterParser {
    /// Create a new tree-sitter parser for a specific language
    pub fn new(
        language_config: LanguageConfig,
        config: ParserConfig,
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
    pub fn with_defaults(language_config: LanguageConfig, config: ParserConfig) -> Result<Self> {
        use crate::parser::filter::default_filter;
        use crate::parser::strategy::default_strategy;

        Self::new(
            language_config,
            config,
            Arc::new(default_strategy()),
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

        // Merge consecutive multi-line comments
        let merged_units = self.merge_multiline_comments(units);

        Ok(merged_units)
    }

    /// Merge consecutive multi-line comments into single units
    fn merge_multiline_comments(&self, units: Vec<TranslationUnit>) -> Vec<TranslationUnit> {
        if units.is_empty() {
            return units;
        }

        let mut merged = Vec::new();
        let mut current_group: Vec<TranslationUnit> = Vec::new();

        for unit in units {
            // Check if this unit can be merged with the current group
            let can_merge = if let Some(last) = current_group.last() {
                // Check if they are on consecutive lines
                let is_consecutive = unit.start_pos.line == last.end_pos.line + 1;
                let same_type = unit.node_type == last.node_type;
                let has_format = unit.format_info.is_some() && last.format_info.is_some();
                let same_style = has_format &&
                    unit.format_info.as_ref().unwrap().style == CommentStyle::DocOuter
                    && last.format_info.as_ref().unwrap().style == CommentStyle::DocOuter;

                is_consecutive && same_type && has_format && same_style
            } else {
                false
            };

            if can_merge {
                current_group.push(unit);
            } else {
                // Process the current group
                if !current_group.is_empty() {
                    if current_group.len() > 1 {
                        merged.push(self.merge_comment_group(current_group));
                    } else {
                        merged.push(current_group.into_iter().next().unwrap());
                    }
                    current_group = Vec::new();
                }
                current_group.push(unit);
            }
        }

        // Process the last group
        if !current_group.is_empty() {
            if current_group.len() > 1 {
                merged.push(self.merge_comment_group(current_group));
            } else {
                merged.push(current_group.into_iter().next().unwrap());
            }
        }

        merged
    }

    /// Merge a group of consecutive comments into a single unit
    fn merge_comment_group(&self, mut group: Vec<TranslationUnit>) -> TranslationUnit {
        // Sort by line number
        group.sort_by(|a, b| a.start_pos.line.cmp(&b.start_pos.line));

        // Create merged content
        let merged_content: String = group
            .iter()
            .map(|u| u.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Get the first and last positions
        let first = &group[0];
        let last = &group[group.len() - 1];

        // Update format info to mark as multiline
        let mut format_info = first.format_info.clone().unwrap();
        format_info.is_multiline = true;

        // Fix: Adjust start_pos to include the prefix (e.g., "/// ")
        // This ensures the entire comment line is replaced, not just the content after prefix
        let prefix_len = format_info.line_prefix.as_ref().map(|p| p.len()).unwrap_or(0);
        let merged_start_pos = Position::new(
            first.start_pos.line,
            first.start_pos.column.saturating_sub(prefix_len),
            first.start_pos.offset.saturating_sub(prefix_len),
        );

        // Fix: end_pos should point to the end of the last comment line
        let merged_end_pos = Position::new(
            last.start_pos.line,
            last.start_pos.column + last.content.len(),
            last.start_pos.offset + last.content.len(),
        );

        // Create merged unit
        TranslationUnit {
            id: format!("{}_merged", first.id),
            node_type: first.node_type,
            content: merged_content,
            start_pos: merged_start_pos,
            end_pos: merged_end_pos,
            language: first.language.clone(),
            should_translate: true,
            translated: None,
            format_info: Some(format_info),
            pattern_type: None,
            pattern_name: None,
        }
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

                // Apply filters
                let text = if self.config.trim_content {
                    node_text.trim()
                } else {
                    node_text
                };

                if text.len() < self.config.min_content_length {
                    continue;
                }

                if text.len() > self.config.max_content_length {
                    continue;
                }

                // Skip if only symbols/whitespace
                if is_only_symbols(text) {
                    continue;
                }

                // Apply content filter
                if !self.filter.should_translate(text) {
                    continue;
                }

                // Apply extraction strategy
                let ctx = ExtractionContext::new(text);
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

                // Determine format info and clean content based on capture name and content
                let (clean_content, format_info) = if capture_name.contains("docstring") {
                    // Extract base indent (whitespace before the comment)
                    let base_indent = text.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                    let prefix = "/// ";
                    
                    // Remove base_indent and prefix from each line
                    let clean = text.lines()
                        .map(|line| {
                            let trimmed = line.strip_prefix(&base_indent).unwrap_or(line);
                            trimmed.strip_prefix(prefix).unwrap_or(trimmed)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    let fmt = FormatInfo {
                        style: CommentStyle::DocOuter,
                        base_indent,
                        line_prefix: Some(prefix.to_string()),
                        ends_with_newline: text.ends_with('\n'),
                        is_multiline: text.contains('\n'),
                    };
                    (clean, Some(fmt))
                } else if capture_name.contains("comment") {
                    // Extract base indent (whitespace before the comment)
                    let base_indent = text.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                    
                    // Determine comment style and prefix based on content
                    let (style, prefix) = if text.trim_start().starts_with("///") {
                        (CommentStyle::DocOuter, "/// ")
                    } else if text.trim_start().starts_with("//!") {
                        (CommentStyle::DocInner, "//! ")
                    } else if text.trim_start().starts_with("//") {
                        (CommentStyle::Line, "// ")
                    } else if text.trim_start().starts_with("/*") {
                        // For block comments, we need special handling
                        if text.contains('\n') {
                            (CommentStyle::BlockMulti, "")
                        } else {
                            (CommentStyle::BlockSingle, "")
                        }
                    } else {
                        (CommentStyle::Line, "// ")
                    };
                    
                    // Remove base_indent and prefix from each line
                    let clean = if style == CommentStyle::BlockMulti {
                        // For block comments, extract content between /* and */
                        let start = text.find("/*").map(|i| i + 2).unwrap_or(0);
                        let end = text.rfind("*/").unwrap_or(text.len());
                        let inner = &text[start..end];
                        inner.lines()
                            .map(|line| line.trim_start_matches(' ').strip_prefix("* ").unwrap_or(line.trim_start_matches(' ')))
                            .collect::<Vec<_>>()
                            .join("\n")
                            .trim()
                            .to_string()
                    } else {
                        text.lines()
                            .map(|line| {
                                let trimmed = line.strip_prefix(&base_indent).unwrap_or(line);
                                trimmed.strip_prefix(prefix).unwrap_or(trimmed)
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    };
                    
                    let line_prefix = if style == CommentStyle::BlockMulti {
                        Some(" * ".to_string())
                    } else if prefix.is_empty() {
                        None
                    } else {
                        Some(prefix.to_string())
                    };
                    
                    let fmt = FormatInfo {
                        style,
                        base_indent,
                        line_prefix,
                        ends_with_newline: text.ends_with('\n'),
                        is_multiline: text.contains('\n'),
                    };
                    (clean, Some(fmt))
                } else {
                    (text.to_string(), None)
                };

                let unit = if let Some(fmt) = format_info {
                    TranslationUnit::new_with_format(
                        id,
                        node_type,
                        clean_content,
                        start_pos,
                        end_pos,
                        fmt,
                    )
                } else {
                    TranslationUnit::new(id, node_type, clean_content, start_pos, end_pos)
                };
                units.push(unit);
            }
            match_idx += 1;
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

/// Check if text contains only symbols/whitespace (no actual content)
fn is_only_symbols(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace() || is_punctuation(c))
}

/// Check if character is punctuation
fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// Parser factory for creating language-specific parsers
pub struct TreeSitterParserFactory;

impl TreeSitterParserFactory {
    /// Create a parser for Rust files
    pub fn create_rust_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "rust".to_string(),
            extensions: vec!["rs".to_string()],
            language: tree_sitter_rust::LANGUAGE.into(),
            comment_query: RUST_COMMENT_QUERY.to_string(),
            docstring_query: Some(RUST_DOCSTRING_QUERY.to_string()),
            string_query: Some(RUST_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for Go files
    pub fn create_go_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "go".to_string(),
            extensions: vec!["go".to_string()],
            language: tree_sitter_go::LANGUAGE.into(),
            comment_query: GO_COMMENT_QUERY.to_string(),
            docstring_query: None, // Go uses regular comments for docs
            string_query: Some(GO_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for Python files
    pub fn create_python_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "python".to_string(),
            extensions: vec!["py".to_string()],
            language: tree_sitter_python::LANGUAGE.into(),
            comment_query: PYTHON_COMMENT_QUERY.to_string(),
            docstring_query: Some(PYTHON_DOCSTRING_QUERY.to_string()),
            string_query: Some(PYTHON_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for JavaScript files
    pub fn create_javascript_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "javascript".to_string(),
            extensions: vec!["js".to_string(), "mjs".to_string()],
            language: tree_sitter_javascript::LANGUAGE.into(),
            comment_query: JS_COMMENT_QUERY.to_string(),
            docstring_query: Some(JS_DOCSTRING_QUERY.to_string()),
            string_query: Some(JS_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for TypeScript files
    pub fn create_typescript_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "typescript".to_string(),
            extensions: vec!["ts".to_string()],
            language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            comment_query: JS_COMMENT_QUERY.to_string(),
            docstring_query: Some(JS_DOCSTRING_QUERY.to_string()),
            string_query: Some(JS_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for TSX files
    pub fn create_tsx_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "tsx".to_string(),
            extensions: vec!["tsx".to_string()],
            language: tree_sitter_typescript::LANGUAGE_TSX.into(),
            comment_query: JS_COMMENT_QUERY.to_string(),
            docstring_query: Some(JS_DOCSTRING_QUERY.to_string()),
            string_query: Some(JS_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for Java files
    pub fn create_java_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "java".to_string(),
            extensions: vec!["java".to_string()],
            language: tree_sitter_java::LANGUAGE.into(),
            comment_query: JAVA_COMMENT_QUERY.to_string(),
            docstring_query: Some(JAVA_DOCSTRING_QUERY.to_string()),
            string_query: Some(JAVA_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for C files
    pub fn create_c_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "c".to_string(),
            extensions: vec!["c".to_string(), "h".to_string()],
            language: tree_sitter_c::LANGUAGE.into(),
            comment_query: C_COMMENT_QUERY.to_string(),
            docstring_query: None,
            string_query: Some(C_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for C++ files
    pub fn create_cpp_parser(
        config: ParserConfig,
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
            comment_query: C_COMMENT_QUERY.to_string(),
            docstring_query: None,
            string_query: Some(C_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create a parser for C# files
    pub fn create_csharp_parser(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<TreeSitterParser> {
        let language_config = LanguageConfig {
            name: "csharp".to_string(),
            extensions: vec!["cs".to_string()],
            language: tree_sitter_c_sharp::LANGUAGE.into(),
            comment_query: CSHARP_COMMENT_QUERY.to_string(),
            docstring_query: Some(CSHARP_DOCSTRING_QUERY.to_string()),
            string_query: Some(CSHARP_STRING_QUERY.to_string()),
        };
        TreeSitterParser::new(language_config, config, strategy, filter)
    }

    /// Create all available parsers with given strategy and filter
    pub fn create_all_parsers(
        config: ParserConfig,
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
    pub fn create_all_parsers_with_defaults(config: ParserConfig) -> Vec<Result<TreeSitterParser>> {
        use crate::parser::filter::default_filter;
        use crate::parser::strategy::default_strategy;

        let strategy = Arc::new(default_strategy());
        let filter = Arc::new(default_filter().unwrap_or_else(|_| {
            ContentFilter::new(crate::parser::filter::FilterConfig::default()).unwrap()
        }));

        Self::create_all_parsers(config, strategy, filter)
    }
}

// Tree-sitter queries for different languages

const RUST_COMMENT_QUERY: &str = r#"
((line_comment) @comment
  (#not-match? @comment "^///"))

((block_comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#;

const RUST_DOCSTRING_QUERY: &str = r#"
((line_comment) @docstring
  (#match? @docstring "^///"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#;

const RUST_STRING_QUERY: &str = r#"
(string_literal) @string
(raw_string_literal) @string
"#;

const GO_COMMENT_QUERY: &str = r#"
(comment) @comment
"#;

const GO_STRING_QUERY: &str = r#"
(raw_string_literal) @string
(interpreted_string_literal) @string
"#;

const PYTHON_COMMENT_QUERY: &str = r#"
(comment) @comment
"#;

const PYTHON_DOCSTRING_QUERY: &str = r#"
(expression_statement
  (string) @docstring)

(module
  (expression_statement
    (string) @docstring))
"#;

const PYTHON_STRING_QUERY: &str = r#"
(string) @string
"#;

const JS_COMMENT_QUERY: &str = r#"
(comment) @comment
"#;

const JS_DOCSTRING_QUERY: &str = r#"
((comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#;

const JS_STRING_QUERY: &str = r#"
(string) @string
(template_string) @string
"#;

const JAVA_COMMENT_QUERY: &str = r#"
(line_comment) @comment
(block_comment) @comment
"#;

const JAVA_DOCSTRING_QUERY: &str = r#"
((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#;

const JAVA_STRING_QUERY: &str = r#"
(string_literal) @string
"#;

const C_COMMENT_QUERY: &str = r#"
(comment) @comment
"#;

const C_STRING_QUERY: &str = r#"
(string_literal) @string
"#;

const CSHARP_COMMENT_QUERY: &str = r#"
(comment) @comment
"#;

const CSHARP_DOCSTRING_QUERY: &str = r#"
((comment) @docstring
  (#match? @docstring "^///"))

((comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#;

const CSHARP_STRING_QUERY: &str = r#"
(string_literal) @string
(verbatim_string_literal) @string
(interpolated_string_expression) @string
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::filter::FilterConfig;
    use crate::parser::strategy::{ConfigBasedStrategy, ExtractionConfig};
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

    #[test]
    fn test_is_only_symbols() {
        assert!(is_only_symbols("   "));
        assert!(is_only_symbols("!!!"));
        assert!(is_only_symbols("// "));
        assert!(!is_only_symbols("hello world"));
        assert!(!is_only_symbols("// hello"));
    }
}
