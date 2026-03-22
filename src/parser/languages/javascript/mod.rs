//! JavaScript language parser
//!
//! This module provides specialized parsing for JavaScript source files,
//! handling JS-specific features like JSDoc comments, template strings, and console methods.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::JavaScriptParser;
pub use patterns::JavaScriptPatterns;
