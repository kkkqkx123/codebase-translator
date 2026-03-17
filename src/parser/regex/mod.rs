//! Regex-based fallback parser
//!
//! This module provides a fallback parser that uses regular expressions
//! to extract comments and strings when tree-sitter parsers are not available
//! or for simple file types.
//!
//! # Architecture
//!
//! The regex parser module is organized into several submodules:
//!
//! - `config`: Parser configuration structures
//! - `parser`: Main regex parser implementation
//! - `state_machine`: State machine pattern matcher for complex extraction
//! - `utils`: Utility functions

// Configuration
pub mod config;

// Main parser
pub mod parser;

// State machine matcher
pub mod state_machine;

// Custom pattern matcher
pub mod custom_pattern_matcher;

// Utilities
pub mod utils;

// Re-exports
pub use config::RegexParserConfig;
pub use custom_pattern_matcher::{CustomPatternMatch, CustomPatternMatcher};
pub use parser::RegexParser;
pub use state_machine::{StateMachineBuilder, StateMachineMatch, StateMachineMatcher};
