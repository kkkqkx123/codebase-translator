//! Core traits module
//!
//! This module provides the core traits for the parser system,
//! including the main Parser trait and extraction strategies.

pub mod parser;
pub mod strategy;

pub use parser::Parser;
pub use strategy::{
    ExtractionConfig, ExtractionContext, ExtractionStrategy, StrategyNodeType,
};
