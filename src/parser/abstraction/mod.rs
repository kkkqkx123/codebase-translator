//! Parser abstraction layer
//!
//! This module provides the core abstractions and traits for the parser system,
//! including the main Parser trait and extraction strategies.

pub mod parser;
pub mod strategy;

pub use parser::Parser;
pub use strategy::{
    ExtractionConfig, ExtractionContext, ExtractionStrategy, StrategyNodeType,
};
