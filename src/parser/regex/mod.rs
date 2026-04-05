//! Regex-based pattern matchers for custom extraction
//!
//! This module provides pattern matching utilities for extracting translatable
//! content using regular expressions and state machines.
//!
//! # Components
//!
//! - `state_machine`: State machine pattern matcher for complex multi-step extraction
//! - `custom_pattern_matcher`: Simple regex-based pattern matcher

// State machine matcher
pub mod state_machine;

// Custom pattern matcher
pub mod custom_pattern_matcher;

// Re-exports
pub use custom_pattern_matcher::{CustomPatternMatch, CustomPatternMatcher};
pub use state_machine::{StateMachineBuilder, StateMachineMatch, StateMachineMatcher};
