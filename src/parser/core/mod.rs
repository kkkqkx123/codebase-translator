//! Core extraction framework
//!
//! This module provides generic extraction utilities that can be reused
//! across different language parsers.

pub mod extractor;
pub mod language_parser;
pub mod position_tracker;
pub mod query_executor;
pub mod string_processor;
pub mod traits;
pub mod types;

pub use extractor::{ExtractionCandidate, ExtractionType, Extractor};
pub use language_parser::LanguageParser;
pub use position_tracker::PositionTracker;
pub use query_executor::{QueryExecutor, QueryMatch};
pub use string_processor::{CommentType, StringProcessor};
pub use traits::{ExtractionConfig, Parser, StrategyNodeType};
pub use types::{FunctionCategory, LanguageFunctionPatterns};
