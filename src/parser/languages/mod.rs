//! Language-specific parsers
//!
//! This module contains parsers for specific programming languages.
//! Each language has its own subdirectory with:
//! - `parser.rs`: Main parser implementation
//! - `queries.rs`: Tree-sitter queries
//! - `patterns.rs`: Language-specific patterns (macros, functions, etc.)
//! - `tests.rs`: Parser tests

pub mod c;
pub mod cpp;
pub mod csharp;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use c::CParser;
pub use cpp::CppParser;
pub use csharp::CSharpParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use javascript::JavaScriptParser;
pub use python::PythonParser;
pub use rust::RustParser;
pub use typescript::TypeScriptParser;
