//! File writer module
//!
//! This module provides functionality for writing translated content back to files.
//! It supports atomic writes, backup creation, preview mode, and concurrent writing.
//!
//! Note: All file writing operations are asynchronous and use Tokio runtime.

pub mod concurrent;
pub mod core;
pub mod file;
pub mod r#trait;

pub use concurrent::{ConcurrentWriteStats, ConcurrentWriter, WriteResult};
pub use file::{FileWriter, WriterConfig};

use crate::core::models::TranslationUnit;

/// Factory for creating writers
pub struct WriterFactory;

impl WriterFactory {
    /// Create a new file writer
    pub fn create_file_writer(config: WriterConfig) -> FileWriter {
        FileWriter::new(config)
    }

    /// Create a new file writer with project path
    pub fn create_file_writer_with_path(
        config: WriterConfig,
        project_path: std::path::PathBuf,
    ) -> FileWriter {
        FileWriter::with_project_path(config, project_path)
    }

    /// Create a new concurrent writer
    pub fn create_concurrent_writer(
        config: WriterConfig,
        max_concurrent: usize,
    ) -> ConcurrentWriter {
        ConcurrentWriter::new(config, max_concurrent)
    }

    /// Create a new concurrent writer with project path
    pub fn create_concurrent_writer_with_path(
        config: WriterConfig,
        max_concurrent: usize,
        project_path: std::path::PathBuf,
    ) -> ConcurrentWriter {
        ConcurrentWriter::with_project_path(config, max_concurrent, project_path)
    }
}

/// Helper function to apply translations to a file
///
/// This is a convenience function for simple use cases.
pub fn apply_translations(
    content: &str,
    units: &[TranslationUnit],
) -> crate::core::error::Result<String> {
    core::TranslationApplier::apply_translations(content, units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_config_default() {
        let config = WriterConfig::default();
        assert!(!config.preview_only);
        assert!(config.backup);
        assert!(config.backup_dir.is_none());
        assert!(!config.strict_encoding);
    }

    #[test]
    fn test_writer_config_validate() {
        let config = WriterConfig::default();
        assert!(config.validate().is_ok());

        // Test with valid absolute path (use current dir for cross-platform compatibility)
        let abs_path = std::env::current_dir().expect("Should get current dir");
        let config_with_backup = WriterConfig {
            backup_dir: Some(abs_path),
            ..Default::default()
        };
        assert!(config_with_backup.validate().is_ok());
    }
}
