//! Java language parser
//!
//! This module provides specialized parsing for Java source files,
//! handling Java-specific features like Javadoc comments and string literals.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::JavaParser;
pub use patterns::JavaPatterns;

