//! Code parsing with character-based scanning
//!
//! This module provides parsers for extracting translatable content from source files.
//! It uses character-based scanning for text extraction, with regex-based fallback
//! parsers for simpler file types.
//!
//! # Architecture
//!
//! The parser module is organized into several submodules:
//!
//! - `core/`: Core extraction framework including traits, types, and implementations
//! - `filtering/`: Content filtering system with layered architecture, including language detection
//! - `coordinator/`: High-level coordination for parsing operations
//! - `regex/`: Regex-based fallback parsers
//! - `regex_parsers/`: Type-specific regex parsers for simple file types
//! - `scanner/`: Character-based text scanner (primary extraction method)

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

// Core extraction framework (traits, types, and implementations)
pub mod core;

// Content filtering (includes language detection)
pub mod filtering;

// Parser coordinator
pub mod coordinator;

// Regex-based parsers
pub mod regex;

// Type-specific regex parsers
pub mod regex_parsers;

// Character-based text scanner
pub mod scanner;

// Re-export from core (traits and types)
pub use core::{ExtractionConfig, FunctionCategory, LanguageFunctionPatterns, Parser};

// Re-export from filtering
pub use filtering::{
    from_project_config, from_project_config_with_translator, ContentFilter, FilterConfig,
};

// Re-export language detection types from filtering
pub use filtering::checks::{
    LanguageDetector, LanguageInfo, QuickDetector, SampledDetector, Script,
};

// Re-export from core (utilities)
pub use core::StringProcessor;

// Re-export from regex
pub use regex::{
    RegexParser, RegexParserConfig, StateMachineBuilder, StateMachineMatch, StateMachineMatcher,
};

// Re-export from regex_parsers
pub use regex_parsers::{FallbackParser, HtmlParser, ShellParser, SqlParser};

// Re-export coordinator types
pub use coordinator::{ParserCoordinator, ParserType};

// Re-export from scanner
pub use scanner::{
    Change, ContentDiff, FormatProtector, PlaceholderProtector, PlaceholderSpan, ScannerConfig,
    ScannerLanguageConfig, TextRegion, TextRegionType, TextScanner, TranslatedRegion,
    TranslationReplacer,
};

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
            extract_strings = project_config.extraction.string_literals,
            "Creating parser coordinator"
        );

        let parser_config = ParserConfig {
            extract_comments: project_config.extraction.comments,
            extract_docstrings: project_config.extraction.doc_strings,
            extract_strings: project_config.extraction.string_literals,
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
        };

        let parser = ParserCoordinator::from_project_config(parser_config, project_config)?;
        debug!("Parser coordinator created successfully");
        Ok(parser)
    }
}
