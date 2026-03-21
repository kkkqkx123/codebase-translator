//! Exported-only strategy implementation
//!
//! This module provides a strategy that only extracts exported/public items.

use crate::core::models::NodeType;
use crate::parser::abstraction::strategy::{
    ExtractionContext, ExtractionStrategy, StrategyNodeType,
};

/// Strategy that only extracts exported items
#[derive(Clone)]
pub struct ExportedOnlyStrategy<S: ExtractionStrategy> {
    base: S,
}

impl<S: ExtractionStrategy> ExportedOnlyStrategy<S> {
    /// Create a new exported-only strategy
    pub fn new(base: S) -> Self {
        Self { base }
    }

    /// Get the base strategy
    pub fn base(&self) -> &S {
        &self.base
    }
}

impl<S: ExtractionStrategy> ExtractionStrategy for ExportedOnlyStrategy<S> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::abstraction::strategy::ExtractionConfig;
    use crate::parser::core::strategies::ConfigBasedStrategy;

    #[test]
    fn test_exported_only_strategy() {
        let base = ConfigBasedStrategy::new(ExtractionConfig::default());
        let strategy = ExportedOnlyStrategy::new(base);

        let exported_ctx = ExtractionContext::new("test").with_exported(true);
        let private_ctx = ExtractionContext::new("test").with_exported(false);

        assert!(strategy.should_extract(StrategyNodeType::Comment, &exported_ctx));
        assert!(!strategy.should_extract(StrategyNodeType::Comment, &private_ctx));
    }
}
