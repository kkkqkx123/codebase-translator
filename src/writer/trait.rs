//! Writer trait definition
//!
//! This module defines the Writer trait for writing translated content.

use std::path::PathBuf;

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};

/// Writer trait for writing translated content
pub trait Writer: Send + Sync {
    /// Write translation units back to file
    fn write(&self, file: &File, units: &[TranslationUnit]) -> Result<()>;

    /// Create a backup of the file
    fn backup(&self, file: &File) -> Result<PathBuf>;

    /// Check if dry run mode
    fn is_dry_run(&self) -> bool;
}
