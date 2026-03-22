//! File writer module
//!
//! This module provides functionality for writing translated content back to files.
//! It supports atomic writes, backup creation, preview mode, and concurrent writing.
//!
//! Note: All file writing operations are asynchronous and use Tokio runtime.

pub mod applier;
pub mod concurrent;
pub mod core;
pub mod file;
pub mod format;
pub mod r#trait;

pub use concurrent::{ConcurrentWriteStats, ConcurrentWriter, WriteResult};
pub use file::{FileWriter, WriterConfig};

use crate::config::project::ProjectConfig;
use crate::core::error::Result;
use crate::core::models::TranslationUnit;
use tracing::{debug, info};

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

    /// Create file writer from project config
    pub fn from_project_config(
        project_config: &ProjectConfig,
        project_path: Option<&str>,
    ) -> Result<FileWriter> {
        info!(
            dry_run = project_config.writer.dry_run,
            backup = project_config.writer.backup,
            "Creating file writer"
        );

        let writer_config = WriterConfig {
            preview_only: project_config.writer.dry_run,
            backup: project_config.writer.backup,
            backup_dir: project_config
                .writer
                .backup_dir
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(std::path::PathBuf::from),
            strict_encoding: false,
        };

        let writer = if let Some(path) = project_path {
            FileWriter::with_project_path(writer_config, std::path::PathBuf::from(path))
        } else {
            FileWriter::new(writer_config)
        };

        debug!("File writer created successfully");
        Ok(writer)
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
}
