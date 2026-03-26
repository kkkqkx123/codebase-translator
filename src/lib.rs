//! Codebase Translate - Automatic code comment and documentation translator
//!
//! This crate provides functionality to automatically translate comments,
//! documentation strings, and error messages within codebases.

pub mod cache;
pub mod commands;
pub mod config;
pub mod core;
pub mod encoding;
pub mod logger;
pub mod parser;
pub mod reporter;
pub mod scanner;
pub mod translator;
pub mod utils;
pub mod workflow;
pub mod writer;

// Re-export core types
pub use core::error::{Result, TranslateError};
pub use core::models::{File, FileEntry, LanguageInfo, NodeType, Position, TranslationUnit};

// Re-export types from cache module
pub use cache::{CacheEntry, CacheStats, HierarchicalCache};
pub use encoding::{Detector, Encoder, EncodingResult, EncodingType};
pub use parser::Parser;
pub use reporter::Reporter;
pub use scanner::Scanner;
pub use translator::{ProviderType, Translator};
pub use writer::r#trait::{AsyncWriter, Writer};

// Re-export workflow types
pub use workflow::{
    FileProcessResult, FileProcessor, TranslationWorkflow, WorkflowBuilder, WorkflowConfig,
    WorkflowResult,
};

// Re-export factory functions from their respective modules
pub use cache::CacheFactory;
pub use parser::ParserFactory;
pub use translator::create_translation_service;
pub use writer::WriterFactory;

// Re-export utility functions
pub use utils::hash::calculate_hash;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the library
pub const NAME: &str = env!("CARGO_PKG_NAME");
