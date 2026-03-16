//! Core extraction framework
//!
//! This module provides generic extraction utilities that can be reused
//! across different language parsers.

pub mod extractor;
pub mod position_tracker;
pub mod query_executor;
pub mod string_processor;

pub use extractor::{ExtractionCandidate, ExtractionType, Extractor};
pub use position_tracker::PositionTracker;
pub use query_executor::{QueryExecutor, QueryMatch};
pub use string_processor::StringProcessor;
