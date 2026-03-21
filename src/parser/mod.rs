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
//! - `engine/`: Parsing engines (tree-sitter based)
//! - `filtering/`: Content filtering system with layered architecture
//! - `patterns/`: Function and macro pattern definitions
//! - `queries/`: Tree-sitter query builders and predefined queries
//! - `languages/`: Language-specific parser implementations
//! - `coordinator/`: High-level coordination for parsing operations
//! - `regex/`: Regex-based fallback parsers
//! - `regex_parsers/`: Type-specific regex parsers for simple file types

// Core abstractions (Parser trait, strategies)
pub mod abstraction;

// Core extraction framework
pub mod core;

// Language detection
pub mod detection;

// Parsing engines
pub mod engine;

// Content filtering
pub mod filtering;

// Function patterns
pub mod patterns;

// Parser coordinator
pub mod coordinator;

// Query builders and predefined queries
pub mod queries;

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

// Re-export from engine
pub use engine::{LanguageConfig, ParserConfig, TreeSitterParser, TreeSitterParserFactory};

// Re-export from core
pub use core::{ExtractionCandidate, ExtractionType, Extractor, QueryExecutor, StringProcessor};

// Re-export from queries
pub use queries::{CommentQueries, FunctionQueries, QueryBuilder, StringQueries};

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
