//! Scanner trait definition
//!
//! This module defines the Scanner trait for file system scanning.

use std::path::PathBuf;

use crate::core::error::Result;
use crate::core::models::FileEntry;

/// Scan options for file system scanning
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// Root path to scan
    pub root_path: String,
    /// Include patterns (glob patterns)
    pub include_patterns: Vec<String>,
    /// Exclude patterns (glob patterns)
    pub exclude_patterns: Vec<String>,
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    /// Whether to respect .gitignore files
    pub respect_gitignore: bool,
    /// Additional .gitignore-style patterns
    pub gitignore_patterns: Vec<String>,
    /// Path to .gitignore file (if not in root)
    pub gitignore_path: Option<PathBuf>,
}

/// Scanner trait for file system scanning
pub trait Scanner: Send + Sync {
    /// Scan a directory and return file entries
    fn scan(&self, opts: ScanOptions) -> Result<Vec<FileEntry>>;
}
