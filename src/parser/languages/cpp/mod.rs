//! C++ language parser
//!
//! This module provides specialized parsing for C++ source files,
//! handling comments, string literals, and C++-specific features.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::CppParser;
pub use patterns::CppPatterns;
