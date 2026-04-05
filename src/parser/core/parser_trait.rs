//! Parser trait definition
//!
//! This module defines the Parser trait for extracting translation units from files.

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};

/// Parser trait for extracting translation units from files
pub trait Parser: Send + Sync {
    /// Parse a file and extract translation units
    fn parse(&self, file: &File) -> Result<Vec<TranslationUnit>>;

    /// Check if this parser supports the given file
    fn supports(&self, filename: &str) -> bool;

    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&str];
}
