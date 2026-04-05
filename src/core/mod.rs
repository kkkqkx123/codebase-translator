//! Core domain models and error types

pub mod error;
pub mod models;

pub use error::{Result, TranslateError};
pub use models::{
    File, FileEntry, LanguageInfo, NodeType, PatternType, Position, StrategyNodeType,
    TranslationUnit,
};
