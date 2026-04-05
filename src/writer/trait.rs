//! Writer trait definition
//!
//! This module defines the Writer trait for writing translated content.
//!
//! Note: This trait is currently not used by the main implementations
//! (FileWriter and ConcurrentWriter use async methods instead).
//! It is kept for potential future use or third-party implementations.

use std::path::PathBuf;

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};

/// Writer trait for writing translated content
///
/// # Note
/// This trait defines a synchronous interface. For async operations,
/// use [`FileWriter`](super::FileWriter) or [`ConcurrentWriter`](super::ConcurrentWriter) directly.
pub trait Writer: Send + Sync {
    /// Write translation units back to file
    ///
    /// # Arguments
    /// * `file` - The file to write to
    /// * `units` - Translation units with translated content in the `translated` field
    fn write(&self, file: &File, units: &[TranslationUnit]) -> Result<()>;

    /// Create a backup of the file
    fn backup(&self, file: &File) -> Result<PathBuf>;

    /// Check if preview only mode
    fn is_preview_only(&self) -> bool;
}

/// AsyncWriter trait for async writing operations
///
/// This trait mirrors the Writer trait but for async contexts.
#[async_trait::async_trait]
pub trait AsyncWriter: Send + Sync {
    /// Write translation units back to file asynchronously
    ///
    /// # Arguments
    /// * `file` - The file to write to
    /// * `units` - Translation units with translated content in the `translated` field
    async fn write(&self, file: &File, units: &[TranslationUnit]) -> Result<()>;

    /// Check if preview mode
    fn is_preview(&self) -> bool;
}
