//! File writer module
//!
//! This module provides functionality for writing translated content back to files.
//! It supports atomic writes, backup creation, preview mode, and concurrent writing.

pub mod concurrent;
pub mod file;
pub mod r#trait;

pub use concurrent::ConcurrentWriter;
pub use file::{FileWriter, WriterConfig};
pub use r#trait::Writer;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::error::Result;
use crate::core::models::{File, TranslationUnit};

/// Default implementation of the Writer trait using FileWriter
#[derive(Debug, Clone)]
pub struct DefaultWriter {
    inner: FileWriter,
}

impl DefaultWriter {
    /// Create a new default writer
    pub fn new(config: WriterConfig) -> Self {
        Self {
            inner: FileWriter::new(config),
        }
    }

    /// Get the inner file writer
    pub fn inner(&self) -> &FileWriter {
        &self.inner
    }
}

impl Writer for DefaultWriter {
    fn write(&self, file: &File, units: &[TranslationUnit]) -> Result<()> {
        // Build results map from translated units
        let results: HashMap<String, String> = units
            .iter()
            .filter_map(|unit| {
                unit.translated
                    .clone()
                    .map(|translated| (unit.id.clone(), translated))
            })
            .collect();

        self.inner.write(file, units, &results)
    }

    fn backup(&self, file: &File) -> Result<PathBuf> {
        // Create backup using the file writer's backup functionality
        let content = String::from_utf8_lossy(&file.content);
        self.inner.create_backup(&file.path, &content)
    }

    fn is_dry_run(&self) -> bool {
        // This would need to be determined from config
        // For now, return false as default
        false
    }
}

/// Factory for creating writers
pub struct WriterFactory;

impl WriterFactory {
    /// Create a new file writer
    pub fn create_file_writer(config: WriterConfig) -> FileWriter {
        FileWriter::new(config)
    }

    /// Create a new default writer
    pub fn create_default_writer(config: WriterConfig) -> DefaultWriter {
        DefaultWriter::new(config)
    }

    /// Create a new concurrent writer
    pub fn create_concurrent_writer(
        config: WriterConfig,
        max_concurrent: usize,
    ) -> ConcurrentWriter {
        ConcurrentWriter::new(config, max_concurrent)
    }
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

    #[tokio::test]
    async fn test_default_writer_creation() {
        let config = WriterConfig::default();
        let writer = DefaultWriter::new(config);
        assert!(!writer.is_dry_run());
    }
}
