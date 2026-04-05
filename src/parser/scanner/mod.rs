//! Character-based text scanner for extracting translatable content
//!
//! This module provides a character-by-character scanning approach for extracting
//! translatable text regions from source files, replacing the tree-sitter based
//! approach which had fundamental limitations.
//!
//! # Architecture
//!
//! The scanner module is organized into several components:
//!
//! - `region`: Core data structures for text regions
//! - `language`: Language-specific configuration
//! - `character_scanner`: Core text scanner implementation
//! - `placeholder`: Placeholder protection for template strings
//! - `replacer`: Translation application based on byte offsets
//!
//! # Advantages over tree-sitter
//!
//! - Single-pass O(n) scanning
//! - Precise byte offset-based replacement
//! - No need to reconstruct formatting
//! - Extracts all text containing target languages
//! - Simple language configuration maintenance
//!
//! # Example
//!
//! ```ignore
//! use parser::scanner::{TextScanner, ScannerConfig, ScannerLanguageConfig};
//!
//! let config = ScannerConfig::new(vec!["zh".to_string()])
//!     .with_comments(true)
//!     .with_doc_strings(true);
//!
//! let scanner = TextScanner::from_extension("js", config).unwrap();
//! let regions = scanner.scan("// 这是注释\n");
//!
//! for region in regions {
//!     println!("Found: {:?}", region.region_type);
//! }
//! ```

mod character_scanner;
mod language;
mod placeholder;
mod region;
mod replacer;

pub use character_scanner::{ScannerConfig, TextScanner};
pub use language::ScannerLanguageConfig;
pub use placeholder::{FormatProtector, PlaceholderProtector};
pub use region::{PlaceholderSpan, TextRegion, TextRegionType, TranslatedRegion};
pub use replacer::{Change, ContentDiff, TranslationReplacer};
