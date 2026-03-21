//! Parser engine module
//!
//! This module provides the parsing engines for extracting translatable content.
//! It includes tree-sitter based parsing and regex-based fallback parsing.

pub mod tree_sitter;

pub use tree_sitter::{LanguageConfig, ParserConfig, TreeSitterParser, TreeSitterParserFactory};
