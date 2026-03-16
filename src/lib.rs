//! Codebase Translate - Automatic code comment and documentation translator
//!
//! This crate provides functionality to automatically translate comments,
//! documentation strings, and error messages within codebases.

pub mod cache;
pub mod config;
pub mod core;
pub mod encoding;
pub mod logger;
pub mod parser;
pub mod reporter;
pub mod scanner;
pub mod translator;
pub mod writer;

// Re-export core types
pub use core::error::{Result, TranslateError};
pub use core::models::{File, FileEntry, LanguageInfo, NodeType, Position, TranslationUnit};

// Re-export traits from their respective modules
pub use cache::{Cache, CacheEntry, CacheStats};
pub use encoding::{Detector, Encoder, EncodingResult, EncodingType};
pub use parser::Parser;
pub use reporter::Reporter;
pub use scanner::Scanner;
pub use translator::{ProviderType, Translator};
pub use writer::r#trait::{AsyncWriter, Writer};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the library
pub const NAME: &str = env!("CARGO_PKG_NAME");
