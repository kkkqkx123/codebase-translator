//! Config-based strategy implementation
//!
//! This module provides a strategy implementation based on configuration settings.

use crate::core::models::NodeType;
use crate::parser::core::traits::{
    ExtractionConfig, ExtractionContext, ExtractionStrategy, StrategyNodeType,
};
use tracing::debug;

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
        let should_extract = match node_type {
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
        };

        debug!(
            node_type = %node_type,
            should_extract,
            strategy = "config_based",
            "Extraction strategy decision"
        );

        should_extract
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_based_strategy() {
        let config = ExtractionConfig {
            comments: true,
            docstrings: false,
            ..Default::default()
        };
        let strategy = ConfigBasedStrategy::new(config);
        let ctx = ExtractionContext::new("test");

        assert!(strategy.should_extract(StrategyNodeType::Comment, &ctx));
        assert!(!strategy.should_extract(StrategyNodeType::DocString, &ctx));
    }

    #[test]
    fn test_default_strategy() {
        let strategy = ConfigBasedStrategy::new(ExtractionConfig::default());
        let ctx = ExtractionContext::new("test");

        assert!(strategy.should_extract(StrategyNodeType::Comment, &ctx));
        assert!(strategy.should_extract(StrategyNodeType::DocString, &ctx));
    }
}
