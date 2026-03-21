//! Rust language parser
//!
//! This module provides specialized parsing for Rust source files,
//! handling Rust-specific features like macros, attributes, and doc comments.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::RustParser;
pub use patterns::RustPatterns;

