//! TypeScript language parser
//!
//! This module provides specialized parsing for TypeScript source files,
//! handling TS-specific features like JSDoc comments, type annotations, and template strings.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::TypeScriptParser;
pub use patterns::TypeScriptPatterns;

