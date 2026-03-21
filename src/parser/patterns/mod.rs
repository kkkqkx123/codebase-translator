//! Function patterns module
//!
//! This module provides common types for function and macro classification
//! across different programming languages. It categorizes functions into
//! error, format, log, and debug categories to help determine how to handle
//! string arguments.
//!
//! Language-specific patterns should be defined in the corresponding
//! `src/parser/languages/*/patterns.rs` files.

pub mod function_patterns;

pub use function_patterns::{FunctionCategory, LanguageFunctionPatterns};
