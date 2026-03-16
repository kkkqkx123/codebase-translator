//! Extraction strategy module
//!
//! This module provides strategies for determining which content should be extracted
//! and translated. It supports configuration-based strategies and allows for custom
//! strategy implementations.

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

/// Extraction strategy trait
pub trait ExtractionStrategy: Send + Sync {
    /// Determine if a node should be extracted
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool;

    /// Get the corresponding NodeType for a StrategyNodeType
    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType;

    /// Get strategy name
    fn name(&self) -> &str;
}

/// Extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Extract comments
    #[serde(default = "default_true")]
    pub comments: bool,

    /// Extract docstrings
    #[serde(default = "default_true")]
    pub docstrings: bool,

    /// Extract error messages
    #[serde(default = "default_true")]
    pub error_messages: bool,

    /// Extract format strings
    #[serde(default = "default_false")]
    pub format_strings: bool,

    /// Extract log messages
    #[serde(default = "default_true")]
    pub log_messages: bool,

    /// Extract string literals
    #[serde(default = "default_false")]
    pub string_literals: bool,

    /// Custom extraction patterns
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            comments: true,
            docstrings: true,
            error_messages: true,
            format_strings: false,
            log_messages: true,
            string_literals: false,
            custom_patterns: Vec::new(),
        }
    }
}

/// Config-based extraction strategy
#[derive(Clone)]
pub struct ConfigBasedStrategy {
    config: ExtractionConfig,
}

impl ConfigBasedStrategy {
    /// Create a new config-based strategy
    pub fn new(config: ExtractionConfig) -> Self {
        Self { config }
    }

    /// Create from a TOML file
    pub fn from_file(path: &std::path::Path) -> crate::core::error::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::core::error::TranslateError::Config(format!("Failed to read config file: {}", e))
        })?;
        let config: ExtractionConfig = toml::from_str(&content).map_err(|e| {
            crate::core::error::TranslateError::Config(format!("Failed to parse config: {}", e))
        })?;
        Ok(Self::new(config))
    }

    /// Get configuration
    pub fn config(&self) -> &ExtractionConfig {
        &self.config
    }
}

impl ExtractionStrategy for ConfigBasedStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, _ctx: &ExtractionContext) -> bool {
        match node_type {
            StrategyNodeType::Comment => self.config.comments,
            StrategyNodeType::DocString => self.config.docstrings,
            StrategyNodeType::ErrorMessage => self.config.error_messages,
            StrategyNodeType::FormatString => self.config.format_strings,
            StrategyNodeType::LogMessage => self.config.log_messages,
            StrategyNodeType::StringLiteral => self.config.string_literals,
            StrategyNodeType::MarkdownParagraph
            | StrategyNodeType::MarkdownHeading
            | StrategyNodeType::MarkdownListItem
            | StrategyNodeType::MarkdownTableCell => true,
        }
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        match node_type {
            StrategyNodeType::Comment => NodeType::Comment,
            StrategyNodeType::DocString => NodeType::DocString,
            StrategyNodeType::ErrorMessage => NodeType::ErrorMessage,
            StrategyNodeType::FormatString => NodeType::FormatString,
            StrategyNodeType::LogMessage => NodeType::LogMessage,
            StrategyNodeType::StringLiteral => NodeType::StringLiteral,
            StrategyNodeType::MarkdownParagraph => NodeType::Comment,
            StrategyNodeType::MarkdownHeading => NodeType::Comment,
            StrategyNodeType::MarkdownListItem => NodeType::Comment,
            StrategyNodeType::MarkdownTableCell => NodeType::Comment,
        }
    }

    fn name(&self) -> &str {
        "config_based"
    }
}

/// How to combine multiple strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombineMode {
    /// All strategies must agree (AND)
    All,
    /// At least one strategy must agree (OR)
    Any,
    /// First strategy that decides wins
    First,
}

/// Combined strategy that applies multiple strategies using static dispatch
#[derive(Clone)]
pub struct CombinedStrategy {
    strategies: Vec<ExtractionStrategyImpl>,
    combine_mode: CombineMode,
}

impl CombinedStrategy {
    /// Create a new combined strategy
    pub fn new(strategies: Vec<ExtractionStrategyImpl>, mode: CombineMode) -> Self {
        Self {
            strategies,
            combine_mode: mode,
        }
    }

    /// Add a strategy
    pub fn add_strategy(&mut self, strategy: ExtractionStrategyImpl) {
        self.strategies.push(strategy);
    }

    /// Get combine mode
    pub fn combine_mode(&self) -> CombineMode {
        self.combine_mode
    }
}

impl ExtractionStrategy for CombinedStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        if self.strategies.is_empty() {
            return true;
        }

        match self.combine_mode {
            CombineMode::All => self
                .strategies
                .iter()
                .all(|s| s.should_extract(node_type, ctx)),
            CombineMode::Any => self
                .strategies
                .iter()
                .any(|s| s.should_extract(node_type, ctx)),
            CombineMode::First => self
                .strategies
                .first()
                .map(|s| s.should_extract(node_type, ctx))
                .unwrap_or(true),
        }
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        self.strategies
            .first()
            .map(|s| s.get_node_type(node_type))
            .unwrap_or(match node_type {
                StrategyNodeType::Comment => NodeType::Comment,
                _ => NodeType::DocString,
            })
    }

    fn name(&self) -> &str {
        "combined"
    }
}

/// Strategy that only extracts exported items using static dispatch
#[derive(Clone)]
pub struct ExportedOnlyStrategy {
    base: Box<ExtractionStrategyImpl>,
}

impl ExportedOnlyStrategy {
    /// Create a new exported-only strategy
    pub fn new(base: ExtractionStrategyImpl) -> Self {
        Self {
            base: Box::new(base),
        }
    }
}

impl ExtractionStrategy for ExportedOnlyStrategy {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        if !ctx.is_exported {
            return false;
        }
        self.base.should_extract(node_type, ctx)
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        self.base.get_node_type(node_type)
    }

    fn name(&self) -> &str {
        "exported_only"
    }
}

/// Create default strategy
pub fn default_strategy() -> ExtractionStrategyImpl {
    ExtractionStrategyImpl::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig::default()))
}

/// Static dispatch extraction strategy implementation enum
///
/// This enum provides static dispatch for all extraction strategy implementations,
#[derive(Clone)]
pub enum ExtractionStrategyImpl {
    ConfigBased(ConfigBasedStrategy),
    Combined(CombinedStrategy),
    ExportedOnly(ExportedOnlyStrategy),
}

impl ExtractionStrategy for ExtractionStrategyImpl {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        match self {
            Self::ConfigBased(s) => s.should_extract(node_type, ctx),
            Self::Combined(s) => s.should_extract(node_type, ctx),
            Self::ExportedOnly(s) => s.should_extract(node_type, ctx),
        }
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        match self {
            Self::ConfigBased(s) => s.get_node_type(node_type),
            Self::Combined(s) => s.get_node_type(node_type),
            Self::ExportedOnly(s) => s.get_node_type(node_type),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::ConfigBased(s) => s.name(),
            Self::Combined(s) => s.name(),
            Self::ExportedOnly(s) => s.name(),
        }
    }
}

impl ExtractionStrategyImpl {
    /// Create a default strategy
    pub fn default_config() -> Self {
        Self::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig::default()))
    }

    /// Create from configuration
    pub fn from_config(config: ExtractionConfig) -> Self {
        Self::ConfigBased(ConfigBasedStrategy::new(config))
    }

    /// Create a combined strategy
    pub fn combined(strategies: Vec<ExtractionStrategyImpl>, mode: CombineMode) -> Self {
        Self::Combined(CombinedStrategy::new(strategies, mode))
    }

    /// Create an exported-only strategy
    pub fn exported_only(base: ExtractionStrategyImpl) -> Self {
        Self::ExportedOnly(ExportedOnlyStrategy::new(base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_node_type_display() {
        assert_eq!(StrategyNodeType::Comment.to_string(), "comment");
        assert_eq!(StrategyNodeType::DocString.to_string(), "docstring");
        assert_eq!(StrategyNodeType::ErrorMessage.to_string(), "error_message");
    }

    #[test]
    fn test_extraction_context() {
        let ctx = ExtractionContext::new("test content")
            .with_function_name("test_fn")
            .with_exported(true)
            .with_metadata("key", "value");

        assert_eq!(ctx.content, "test content");
        assert_eq!(ctx.function_name, Some("test_fn".to_string()));
        assert!(ctx.is_exported);
        assert_eq!(ctx.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_config_based_strategy() {
        let config = ExtractionConfig {
            comments: true,
            docstrings: false,
            error_messages: true,
            format_strings: false,
            log_messages: true,
            custom_patterns: Vec::new(),
        };

        let strategy = ConfigBasedStrategy::new(config);
        let ctx = ExtractionContext::new("test");

        assert!(strategy.should_extract(StrategyNodeType::Comment, &ctx));
        assert!(!strategy.should_extract(StrategyNodeType::DocString, &ctx));
        assert!(strategy.should_extract(StrategyNodeType::ErrorMessage, &ctx));
        assert!(!strategy.should_extract(StrategyNodeType::FormatString, &ctx));
        assert!(strategy.should_extract(StrategyNodeType::LogMessage, &ctx));

        assert_eq!(
            strategy.get_node_type(StrategyNodeType::Comment),
            NodeType::Comment
        );
        assert_eq!(
            strategy.get_node_type(StrategyNodeType::DocString),
            NodeType::DocString
        );
        assert_eq!(strategy.name(), "config_based");
    }

    #[test]
    fn test_combined_strategy_all() {
        let strategy1 =
            ExtractionStrategyImpl::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig {
                comments: true,
                ..Default::default()
            }));

        let strategy2 =
            ExtractionStrategyImpl::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig {
                comments: false,
                ..Default::default()
            }));

        let combined = CombinedStrategy::new(vec![strategy1, strategy2], CombineMode::All);
        let ctx = ExtractionContext::new("test");

        // Both must agree (AND)
        assert!(!combined.should_extract(StrategyNodeType::Comment, &ctx));
    }

    #[test]
    fn test_combined_strategy_any() {
        let strategy1 =
            ExtractionStrategyImpl::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig {
                comments: true,
                ..Default::default()
            }));

        let strategy2 =
            ExtractionStrategyImpl::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig {
                comments: false,
                ..Default::default()
            }));

        let combined = CombinedStrategy::new(vec![strategy1, strategy2], CombineMode::Any);
        let ctx = ExtractionContext::new("test");

        // At least one must agree (OR)
        assert!(combined.should_extract(StrategyNodeType::Comment, &ctx));
    }

    #[test]
    fn test_exported_only_strategy() {
        let base =
            ExtractionStrategyImpl::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig {
                comments: true,
                ..Default::default()
            }));

        let strategy = ExportedOnlyStrategy::new(base);
        let exported_ctx = ExtractionContext::new("test").with_exported(true);
        let private_ctx = ExtractionContext::new("test").with_exported(false);

        assert!(strategy.should_extract(StrategyNodeType::Comment, &exported_ctx));
        assert!(!strategy.should_extract(StrategyNodeType::Comment, &private_ctx));
    }

    #[test]
    fn test_default_strategy() {
        let strategy = default_strategy();
        let ctx = ExtractionContext::new("test");

        assert!(strategy.should_extract(StrategyNodeType::Comment, &ctx));
        assert!(strategy.should_extract(StrategyNodeType::DocString, &ctx));
        assert_eq!(strategy.name(), "config_based");
    }
}
