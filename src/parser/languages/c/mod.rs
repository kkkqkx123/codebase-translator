//! C language parser
//!
//! This module provides specialized parsing for C source files,
//! handling comments and string literals.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::CParser;
pub use patterns::CPatterns;

