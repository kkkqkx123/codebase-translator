//! Parser abstraction layer
//!
//! This module provides the core abstractions and traits for the parser system,
//! including the main Parser trait, extraction strategies, content filters,
//! and function pattern definitions.

pub mod filter;
pub mod function_patterns;
pub mod parser;
pub mod strategy;

pub use filter::{
    from_project_config, from_project_config_with_translator, ContentFilter, FilterConfig,
};
pub use function_patterns::{FunctionCategory, LanguageFunctionPatterns};
pub use parser::Parser;
pub use strategy::{
    default_strategy, ConfigBasedStrategy, ExtractionConfig, ExtractionContext, ExtractionStrategy,
    ExtractionStrategyImpl, StrategyNodeType,
};
