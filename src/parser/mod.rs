//! Code parsing with tree-sitter and regex fallback
//!
//! This module provides parsers for extracting translatable content from source files.
//! It uses tree-sitter for accurate parsing of various programming languages,
//! with regex-based fallback parsers for simpler file types.
//!
//! # Architecture
//!
//! The parser module is organized into several submodules:
//!
//! - `core/`: Generic extraction framework reusable across languages
//! - `queries/`: Tree-sitter query builders and predefined queries
//! - `languages/`: Language-specific parser implementations
//! - `strategy/`: Extraction strategies for filtering content
//! - `filter/`: Content filters for translation units
//! - `patterns/`: Function/macro pattern classification
//! - `tree_sitter/`: Tree-sitter integration
//! - `regex/`: Regex-based fallback parsers

// Core extraction framework
pub mod core;

// Parser coordinator
pub mod coordinator;

// Query builders and predefined queries
pub mod queries;

// Language-specific parsers
pub mod languages;

// Extraction strategies
pub mod strategy;

// Content filters
pub mod filter;

// Function/macro patterns
pub mod function_patterns;

// Language detection
pub mod language;

// Tree-sitter integration
pub mod tree_sitter;

// Regex-based parsers
pub mod regex;

// Type-specific regex parsers
pub mod regex_parsers;

// String literal extractor (deprecated - not currently used)
#[deprecated(
    since = "0.1.0",
    note = "This module is not currently used and may be removed in a future version"
)]
pub mod string_extractor;

// Parser trait
pub mod r#trait;

// Re-export commonly used types
pub use core::{ExtractionCandidate, ExtractionType, Extractor, QueryExecutor, StringProcessor};
pub use filter::{
    from_project_config, from_project_config_with_translator, ContentFilter, FilterConfig,
};
pub use function_patterns::{FunctionCategory, LanguageFunctionPatterns};
pub use language::{LanguageDetector, LanguageInfo};
pub use languages::RustParser;
pub use queries::{CommentQueries, FunctionQueries, QueryBuilder, StringQueries};
pub use r#trait::Parser;
pub use regex::{
    RegexParser, RegexParserConfig, StateMachineBuilder, StateMachineMatch, StateMachineMatcher,
};
pub use regex_parsers::{FallbackParser, HtmlParser, ShellParser, SqlParser};
pub use strategy::{
    default_strategy, ConfigBasedStrategy, ExtractionConfig, ExtractionContext, ExtractionStrategy,
    ExtractionStrategyImpl, StrategyNodeType,
};
pub use tree_sitter::{LanguageConfig, ParserConfig, TreeSitterParser, TreeSitterParserFactory};

// Re-export coordinator types
pub use coordinator::{ParserCoordinator, ParserType};
