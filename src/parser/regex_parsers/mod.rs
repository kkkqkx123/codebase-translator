//! Regex-based parsers for specific file types
//!
//! This module provides type-specific regex parsers for simple file types
//! that don't have tree-sitter parsers available.
//!
//! # Available Parsers
//!
//! - `FallbackParser`: Generic fallback for txt, md, yaml, toml files
//! - `ShellParser`: Shell script parser for sh, bash, zsh, fish files
//! - `HtmlParser`: HTML/XML parser for html, htm, xml, svg files
//! - `SqlParser`: SQL parser for sql, mysql, pgsql files

pub mod fallback;
pub mod html;
pub mod shell;
pub mod sql;

pub use fallback::FallbackParser;
pub use html::HtmlParser;
pub use shell::ShellParser;
pub use sql::SqlParser;
