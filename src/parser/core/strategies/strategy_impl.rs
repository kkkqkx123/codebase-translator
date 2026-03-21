//! Extraction strategy implementation enum
//!
//! This module provides a static dispatch enum for extraction strategies.
//! It wraps all concrete strategy implementations for efficient dispatch.

use crate::core::models::NodeType;
use crate::parser::abstraction::strategy::{
    ExtractionContext, ExtractionStrategy, StrategyNodeType,
};

use super::config_based::ConfigBasedStrategy;

/// Static dispatch extraction strategy implementation enum
///
/// This enum provides static dispatch for all extraction strategy implementations,
/// avoiding the overhead of dynamic dispatch while maintaining flexibility.
#[derive(Clone)]
pub enum ExtractionStrategyImpl {
    /// Config-based strategy
    ConfigBased(ConfigBasedStrategy),
}

impl ExtractionStrategy for ExtractionStrategyImpl {
    fn should_extract(&self, node_type: StrategyNodeType, ctx: &ExtractionContext) -> bool {
        match self {
            Self::ConfigBased(s) => s.should_extract(node_type, ctx),
        }
    }

    fn get_node_type(&self, node_type: StrategyNodeType) -> NodeType {
        match self {
            Self::ConfigBased(s) => s.get_node_type(node_type),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::ConfigBased(s) => s.name(),
        }
    }
}

impl ExtractionStrategyImpl {
    /// Create a default strategy
    pub fn default_config() -> Self {
        use crate::parser::abstraction::strategy::ExtractionConfig;
        Self::ConfigBased(ConfigBasedStrategy::new(ExtractionConfig::default()))
    }
}

impl Default for ExtractionStrategyImpl {
    fn default() -> Self {
        Self::default_config()
    }
}
