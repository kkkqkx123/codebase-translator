//! Extraction strategy trait
//!
//! This module provides the core trait for extraction strategies.
//! Concrete implementations are located in `parser::core::strategies`.

use crate::core::models::NodeType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy node type for extraction decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyNodeType {
    /// Regular comment
    Comment,
    /// Documentation string
    DocString,
    /// Error message
    ErrorMessage,
    /// Format string
    FormatString,
    /// Log message
    LogMessage,
    /// String literal
    StringLiteral,
    /// Markdown paragraph
    MarkdownParagraph,
    /// Markdown heading
    MarkdownHeading,
    /// Markdown list item
    MarkdownListItem,
    /// Markdown table cell
    MarkdownTableCell,
}

impl StrategyNodeType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::DocString => "docstring",
            Self::ErrorMessage => "error_message",
            Self::FormatString => "format_string",
            Self::LogMessage => "log_message",
            Self::StringLiteral => "string_literal",
            Self::MarkdownParagraph => "markdown_paragraph",
            Self::MarkdownHeading => "markdown_heading",
            Self::MarkdownListItem => "markdown_list_item",
            Self::MarkdownTableCell => "markdown_table_cell",
        }
    }
}

impl std::fmt::Display for StrategyNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Context for extraction decisions
#[derive(Debug, Clone, Default)]
pub struct ExtractionContext {
    /// Content to extract
    pub content: String,
    /// Function name (if applicable)
    pub function_name: Option<String>,
    /// Whether the item is exported/public
    pub is_exported: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ExtractionContext {
    /// Create a new extraction context
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            function_name: None,
            is_exported: false,
            metadata: HashMap::new(),
        }
    }

    /// Set function name
    pub fn with_function(mut self, name: impl Into<String>) -> Self {
        self.function_name = Some(name.into());
        self
    }

    /// Set function name (alias for with_function)
    pub fn with_function_name(mut self, name: impl Into<String>) -> Self {
        self.function_name = Some(name.into());
        self
    }

    /// Set exported flag
    pub fn with_exported(mut self, exported: bool) -> Self {
        self.is_exported = exported;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Configuration for extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Extract comments
    pub comments: bool,
    /// Extract docstrings
    pub docstrings: bool,
    /// Extract string literals
    pub string_literals: bool,
    /// Extract error messages
    pub error_messages: bool,
    /// Extract format strings
    pub format_strings: bool,
    /// Extract log messages
    pub log_messages: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            comments: true,
            docstrings: true,
            string_literals: false,
            error_messages: true,
            format_strings: true,
            log_messages: true,
        }
    }
}

/// Extraction strategy trait
pub trait ExtractionStrategy: Send + Sync {
    /// Determine if a node should be extracted
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool;

    /// Get the node type for a strategy node type
    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType;

    /// Get strategy name
    fn name(&self) -> &str;
}
