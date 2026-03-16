//! Go language parser
//!
//! This module provides specialized parsing for Go source files,
//! handling Go-specific features like doc comments and format strings.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::GoParser;
pub use patterns::GoPatterns;
