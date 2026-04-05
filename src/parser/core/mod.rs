//! Core extraction framework
//!
//! This module provides generic extraction utilities that can be reused
//! across different language parsers.

pub mod parser_trait;
pub mod position_tracker;
pub mod string_processor;
pub mod types;

pub use parser_trait::Parser;
pub use position_tracker::PositionTracker;
pub use string_processor::{CommentType, StringProcessor};
pub use types::{FunctionCategory, LanguageFunctionPatterns};

// Re-export types from other modules for convenience
pub use crate::config::project::ExtractionConfig;
