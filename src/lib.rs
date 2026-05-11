//! Codebase Translate - Automatic code comment and documentation translator
//!
//! This crate provides functionality to automatically translate comments,
//! documentation strings, and error messages within codebases.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global quiet mode flag - when true, suppress non-essential terminal output
static QUIET_MODE: AtomicBool = AtomicBool::new(false);

/// Set the global quiet mode
pub fn set_quiet_mode(quiet: bool) {
    QUIET_MODE.store(quiet, Ordering::SeqCst);
}

/// Check if quiet mode is enabled
pub fn is_quiet_mode() -> bool {
    QUIET_MODE.load(Ordering::SeqCst)
}

/// Print to stdout only if not in quiet mode
#[macro_export]
macro_rules! quiet_print {
    ($($arg:tt)*) => {
        if !$crate::is_quiet_mode() {
            println!($($arg)*);
        }
    };
}

/// Print to stderr only if not in quiet mode
#[macro_export]
macro_rules! quiet_eprint {
    ($($arg:tt)*) => {
        if !$crate::is_quiet_mode() {
            eprintln!($($arg)*);
        }
    };
}

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
pub use cache::{CacheEntry, CacheStats, DirectoryCache};
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
