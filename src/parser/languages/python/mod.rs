//! Python language parser
//!
//! This module provides specialized parsing for Python source files,
//! handling Python-specific features like docstrings and f-strings.

pub mod parser;
pub mod patterns;
pub mod queries;

pub use parser::PythonParser;
pub use patterns::PythonPatterns;

