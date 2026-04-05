//! Extraction strategy types
//!
//! This module provides types for extraction configuration and decisions.

use crate::core::models::NodeType;
use serde::{Deserialize, Serialize};

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
    /// Variable assignment string
    VariableString,
    /// Object property string
    PropertyString,
    /// Test description (it, describe, test, etc.)
    TestDescription,
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
            Self::VariableString => "variable_string",
            Self::PropertyString => "property_string",
            Self::TestDescription => "test_description",
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
    /// Extract variable assignment strings (e.g., const x = "message")
    pub variable_strings: bool,
    /// Extract object property strings (e.g., { description: "message" })
    pub property_strings: bool,
    /// Extract test descriptions (e.g., it("should work", fn))
    pub test_descriptions: bool,
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
            variable_strings: false,
            property_strings: false,
            test_descriptions: true,
        }
    }
}

impl ExtractionConfig {
    /// Determine if a node should be extracted based on configuration
    pub fn should_extract(&self, node_type: StrategyNodeType) -> bool {
        match node_type {
            StrategyNodeType::Comment => self.comments,
            StrategyNodeType::DocString => self.docstrings,
            StrategyNodeType::ErrorMessage => self.error_messages,
            StrategyNodeType::FormatString => self.format_strings,
            StrategyNodeType::LogMessage => self.log_messages,
            StrategyNodeType::StringLiteral => self.string_literals,
            StrategyNodeType::VariableString => self.variable_strings,
            StrategyNodeType::PropertyString => self.property_strings,
            StrategyNodeType::TestDescription => self.test_descriptions,
            StrategyNodeType::MarkdownParagraph
            | StrategyNodeType::MarkdownHeading
            | StrategyNodeType::MarkdownListItem
            | StrategyNodeType::MarkdownTableCell => true,
        }
    }

    /// Get the node type for a strategy node type
    pub fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        match node_type {
            StrategyNodeType::Comment => NodeType::Comment,
            StrategyNodeType::DocString => NodeType::DocString,
            StrategyNodeType::ErrorMessage => NodeType::ErrorMessage,
            StrategyNodeType::FormatString => NodeType::FormatString,
            StrategyNodeType::LogMessage => NodeType::LogMessage,
            StrategyNodeType::StringLiteral => NodeType::StringLiteral,
            StrategyNodeType::VariableString => NodeType::StringLiteral,
            StrategyNodeType::PropertyString => NodeType::StringLiteral,
            StrategyNodeType::TestDescription => NodeType::TestDescription,
            StrategyNodeType::MarkdownParagraph => NodeType::Comment,
            StrategyNodeType::MarkdownHeading => NodeType::Comment,
            StrategyNodeType::MarkdownListItem => NodeType::Comment,
            StrategyNodeType::MarkdownTableCell => NodeType::Comment,
        }
    }

    /// Load from a TOML file
    pub fn from_file(path: &std::path::Path) -> crate::core::error::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::core::error::TranslateError::Config(format!("Failed to read config file: {}", e))
        })?;
        let config: Self = toml::from_str(&content).map_err(|e| {
            crate::core::error::TranslateError::Config(format!("Failed to parse config: {}", e))
        })?;
        Ok(config)
    }
}
