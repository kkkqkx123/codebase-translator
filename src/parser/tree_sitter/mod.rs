//! Tree-sitter based parser module
//!
//! This module provides tree-sitter based parser for extracting translatable content.

pub mod builder;
pub mod parser;

pub use builder::QueryBuilder;
pub use parser::{LanguageConfig, TreeSitterParser, TreeSitterParserFactory};
