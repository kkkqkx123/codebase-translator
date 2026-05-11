//! Core domain models, error types, and shared utilities

pub mod error;
pub mod models;
pub mod reader;

pub use error::{Result, TranslateError};
pub use models::{File, FileEntry, LanguageInfo, NodeType, PatternType, Position, TranslationUnit};
