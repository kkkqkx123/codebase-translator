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
//! - `abstraction/`: Core abstractions including Parser trait and extraction strategies
//! - `core/`: Generic extraction framework reusable across languages
//! - `detection/`: Language detection capabilities
//! - `tree_sitter/`: Tree-sitter based parser implementation and query builder
//! - `filtering/`: Content filtering system with layered architecture
//! - `patterns/`: Function and macro pattern definitions
//! - `languages/`: Language-specific parser implementations and queries
//! - `coordinator/`: High-level coordination for parsing operations
//! - `regex/`: Regex-based fallback parsers
//! - `regex_parsers/`: Type-specific regex parsers for simple file types

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to extract comments
    pub extract_comments: bool,
    /// Whether to extract docstrings
    pub extract_docstrings: bool,
    /// Whether to extract string literals
    pub extract_strings: bool,
    /// Minimum content length to extract
    pub min_content_length: usize,
    /// Maximum content length to extract
    pub max_content_length: usize,
    /// Whether to trim whitespace from content
    pub trim_content: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            extract_comments: true,
            extract_docstrings: true,
            extract_strings: false,
            min_content_length: 0,
            max_content_length: 100000,
            trim_content: true,
        }
    }
}

// Core abstractions (Parser trait, strategies)
pub mod abstraction;

// Core extraction framework
pub mod core;

// Language detection
pub mod detection;

// Tree-sitter based parser
pub mod tree_sitter;

// Content filtering
pub mod filtering;

// Function patterns
pub mod patterns;

// Parser coordinator
pub mod coordinator;

// Note: QueryBuilder is now in tree_sitter module

// Language-specific parsers
pub mod languages;

// Regex-based parsers
pub mod regex;

// Type-specific regex parsers
pub mod regex_parsers;

// Re-export commonly used types from abstraction
pub use abstraction::{
    ExtractionConfig, ExtractionContext, ExtractionStrategy, StrategyNodeType, Parser,
};

// Re-export from core (strategies)
pub use core::{ConfigBasedStrategy, ExtractionStrategyImpl};

// Re-export from filtering
pub use filtering::{
    from_project_config, from_project_config_with_translator, ContentFilter, FilterConfig,
};

// Re-export from patterns
pub use patterns::{FunctionCategory, LanguageFunctionPatterns};

// Re-export from detection
pub use detection::{LanguageDetector, LanguageInfo, Script};

// Re-export from tree_sitter
pub use tree_sitter::{LanguageConfig, QueryBuilder, TreeSitterParser, TreeSitterParserFactory};

// Re-export from core
pub use core::{ExtractionCandidate, ExtractionType, Extractor, QueryExecutor, StringProcessor};

// Re-export from regex
pub use regex::{
    RegexParser, RegexParserConfig, StateMachineBuilder, StateMachineMatch, StateMachineMatcher,
};

// Re-export from regex_parsers
pub use regex_parsers::{FallbackParser, HtmlParser, ShellParser, SqlParser};

// Re-export from languages
pub use languages::RustParser;

// Re-export coordinator types
pub use coordinator::{ParserCoordinator, ParserType};

use crate::config::project::ProjectConfig;
use crate::core::error::Result;
use tracing::{debug, info};

/// Factory for creating parser instances
pub struct ParserFactory;

impl ParserFactory {
    /// Create parser coordinator
    pub fn create(project_config: &ProjectConfig) -> Result<ParserCoordinator> {
        info!(
            extract_comments = project_config.extraction.comments,
            extract_docstrings = project_config.extraction.doc_strings,
            extract_strings = project_config.extraction.format_strings,
            "Creating parser coordinator"
        );

        let parser_config = ParserConfig {
            extract_comments: project_config.extraction.comments,
            extract_docstrings: project_config.extraction.doc_strings,
            extract_strings: project_config.extraction.format_strings,
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
        };

        let parser = ParserCoordinator::from_project_config(parser_config, project_config)?;
        debug!("Parser coordinator created successfully");
        Ok(parser)
    }
}
