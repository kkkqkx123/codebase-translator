//! Async file writer implementation
//!
//! This module provides async file writing capabilities using Tokio,
//! with encoding conversion, preview mode, backup mechanism, and atomic file writing.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task;
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};

use super::core::TranslationApplier;

/// Configuration for file writer
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Preview only mode - only show changes without writing
    pub preview_only: bool,
    /// Whether to create backups
    pub backup: bool,
    /// Backup directory (empty means same directory as original file)
    pub backup_dir: Option<PathBuf>,
    /// Strict encoding mode
    pub strict_encoding: bool,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            preview_only: false,
            backup: true,
            backup_dir: None,
            strict_encoding: false,
        }
    }
}

impl WriterConfig {
    /// Create a new writer config
    pub fn new() -> Self {
        Self::default()
    }
}

/// Async file writer for writing translated content
#[derive(Debug, Clone)]
pub struct FileWriter {
    config: Arc<RwLock<WriterConfig>>,
    project_path: Option<PathBuf>,
}

impl FileWriter {
    /// Create a new async file writer
    pub fn new(config: WriterConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            project_path: None,
        }
    }

    /// Create a new async file writer with project path
    pub fn with_project_path(config: WriterConfig, project_path: PathBuf) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            project_path: Some(project_path),
        }
    }

    /// Write translations to file asynchronously
    ///
    /// # Arguments
    /// * `file` - The file to write to
    /// * `units` - Translation units with translated content
    pub async fn write(&self, file: &File, units: &[TranslationUnit]) -> Result<()> {
        info!(
            file = %file.path.display(),
            translation_units = units.len(),
            "Starting async file write"
        );

        // Check preview only mode first to avoid unnecessary processing
        let is_preview_only = {
            let config = self.config.read().await;
            config.preview_only
        };

        if is_preview_only {
            return self.write_preview(file, units);
        }

        // Clone data for spawn_blocking to avoid lifetime issues
        let file_content = file.content.clone();
        let units_owned: Vec<TranslationUnit> = units.to_vec();

        // Offload CPU-intensive translation application to blocking thread pool
        let (original_content, modified_content) = task::spawn_blocking(move || {
            let content = String::from_utf8_lossy(&file_content);
            let line_ending = detect_line_ending(&content);

            let modified = TranslationApplier::apply_translations(&content, &units_owned)?;
            let modified = normalize_line_ending(&modified, line_ending);

            Ok::<(String, String), crate::core::error::TranslateError>((
                content.to_string(),
                modified,
            ))
        })
        .await
        .map_err(|e| {
            TranslateError::Io(format!(
                "Failed to apply translations in blocking task: {e}"
            ))
        })??;

        // Perform async file I/O with timeout
        const WRITE_TIMEOUT: Duration = Duration::from_secs(30);
        match tokio::time::timeout(
            WRITE_TIMEOUT,
            self.write_file_atomically(file, &original_content, &modified_content),
        )
        .await
        {
            Ok(result) => {
                result?;
                info!(
                    file = %file.path.display(),
                    "Async file write completed successfully"
                );
                Ok(())
            }
            Err(_) => Err(TranslateError::Io(format!(
                "File write operation timed out after {WRITE_TIMEOUT:?}"
            ))),
        }
    }

    /// Write preview of translations (without modifying file)
    fn write_preview(&self, file: &File, units: &[TranslationUnit]) -> Result<()> {
        info!(
            file = %file.path.display(),
            units_count = units.len(),
            "Previewing translations"
        );

        // Generate the modified content
        let content = String::from_utf8_lossy(&file.content);
        let line_ending = detect_line_ending(&content);
        let modified_content = TranslationApplier::apply_translations(&content, units)?;
        let modified_content = normalize_line_ending(&modified_content, line_ending);

        // Check if there are any actual changes
        if content == modified_content {
            quiet_print!("\n=== File: {} ===", file.path.display());
            quiet_print!("No changes would be made to this file.");
            return Ok(());
        }

        // Print diff-style output
        quiet_print!("\n========================================");
        quiet_print!("  FILE: {}", file.path.display());
        quiet_print!("========================================");
        quiet_print!("\n--- ORIGINAL ---");
        quiet_print!("{}", content);
        quiet_print!("\n+++ TRANSLATED +++");
        quiet_print!("{}", modified_content);
        quiet_print!("\n========================================");
        quiet_print!("  SUMMARY");
        quiet_print!("========================================");

        // Count changes
        let changed_units: Vec<&TranslationUnit> = units
            .iter()
            .filter(|u| {
                u.translated
                    .as_ref()
                    .map(|t| t != &u.content)
                    .unwrap_or(false)
            })
            .collect();

        quiet_print!("Total translation units: {}", units.len());
        quiet_print!("Units with changes: {}", changed_units.len());

        if !changed_units.is_empty() {
            quiet_print!("\n--- Changes Detail ---");
            for unit in &changed_units {
                if let Some(translated) = &unit.translated {
                    quiet_print!(
                        "\n[{}] Line {}-{}:",
                        unit.node_type, unit.start_pos.line, unit.end_pos.line
                    );
                    quiet_print!("  - {}", unit.content.replace('\n', "\n    "));
                    quiet_print!("  + {}", translated.replace('\n', "\n    "));
                }
            }
        }

        quiet_print!("\n========================================\n");

        Ok(())
    }

    /// Write file atomically with backup support (async version)
    async fn write_file_atomically(
        &self,
        file: &File,
        original_content: &str,
        modified_content: &str,
    ) -> Result<()> {
        let file_path = &file.path;
        debug!(file = %file_path.display(), "Starting async atomic file write");

        if original_content == modified_content {
            debug!(file = %file_path.display(), "No changes detected, skipping write");
            return Ok(());
        }

        // Get backup config in a minimal scope to avoid holding lock across await points
        let backup_enabled = {
            let config = self.config.read().await;
            config.backup
        };

        // Create backup if enabled
        if backup_enabled {
            if let Err(e) = self.create_backup(file_path, original_content).await {
                warn!(
                    file = %file_path.display(),
                    error = %e,
                    "Failed to create backup, continuing without backup"
                );
            }
        }

        // Create temporary file
        let temp_path = file_path.with_extension("tmp");

        debug!(
            file = %file_path.display(),
            temp_file = %temp_path.display(),
            "Creating temporary file"
        );

        // Write to temporary file
        if let Err(e) = tokio::fs::write(&temp_path, modified_content).await {
            error!(
                file = %file_path.display(),
                error = %e,
                "Failed to write temporary file"
            );
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(TranslateError::Io(format!(
                "Failed to write temporary file: {e}"
            )));
        }

        // Preserve metadata
        debug!(
            file = %file_path.display(),
            "Preserving file metadata"
        );
        if let Err(e) = self.preserve_metadata(file_path, &temp_path).await {
            error!(
                file = %file_path.display(),
                error = %e,
                "Failed to preserve file metadata"
            );
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(TranslateError::Io(format!(
                "Failed to preserve file metadata: {e}"
            )));
        }

        // Atomic rename
        debug!(
            file = %file_path.display(),
            "Performing atomic rename"
        );
        if let Err(e) = tokio::fs::rename(&temp_path, file_path).await {
            error!(
                file = %file_path.display(),
                error = %e,
                "Failed to replace original file"
            );
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(TranslateError::Io(format!(
                "Failed to replace original file: {e}"
            )));
        }

        debug!(file = %file_path.display(), "Async atomic file write completed");
        Ok(())
    }

    /// Preserve file metadata (permissions)
    async fn preserve_metadata(&self, src_path: &Path, dst_path: &Path) -> Result<()> {
        let metadata = tokio::fs::metadata(src_path)
            .await
            .map_err(|e| TranslateError::Io(format!("Failed to get file metadata: {e}")))?;

        let permissions = metadata.permissions();
        tokio::fs::set_permissions(dst_path, permissions)
            .await
            .map_err(|e| TranslateError::Io(format!("Failed to set file permissions: {e}")))?;

        Ok(())
    }

    /// Create backup of original file
    async fn create_backup(&self, file_path: &Path, original_content: &str) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let base = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");

        let config = self.config.read().await;

        // Determine backup directory
        let backup_base_dir: Option<PathBuf> = if let Some(ref backup_dir) = config.backup_dir {
            // User specified backup directory
            Some(backup_dir.clone())
        } else {
            // Use .translator/backups subdirectory in project path, or None if no project path
            self.project_path
                .as_ref()
                .map(|project_path| project_path.join(".translator").join("backups"))
        };

        // Determine backup path
        let backup_path: PathBuf = if let Some(ref base_dir) = backup_base_dir {
            // Create backup directory if needed
            tokio::fs::create_dir_all(base_dir).await.map_err(|e| {
                TranslateError::Io(format!("Failed to create backup directory: {e}"))
            })?;

            // Calculate relative path from project path to file
            let rel_path = if let Some(ref project_path) = self.project_path {
                if let Ok(rel) = file_path.strip_prefix(project_path) {
                    rel
                } else {
                    // If file is not under project path, use just the filename
                    Path::new(file_path.file_name().unwrap_or_default())
                }
            } else {
                // Preserve relative directory structure from file's parent
                file_path.parent().unwrap_or(Path::new(""))
            };

            // Create backup directory structure in backup directory
            let backup_dir = if let Some(parent) = rel_path.parent() {
                base_dir.join(parent)
            } else {
                base_dir.clone()
            };

            tokio::fs::create_dir_all(&backup_dir).await.map_err(|e| {
                TranslateError::Io(format!("Failed to create backup subdirectory: {e}"))
            })?;

            backup_dir.join(format!("{}_{}.bak.{}", base, timestamp, ext))
        } else {
            // Same directory as original file (fallback)
            file_path
                .parent()
                .map(|p| p.join(format!("{}_{}.bak.{}", base, timestamp, ext)))
                .unwrap_or_else(|| PathBuf::from(format!("{}_{}.bak.{}", base, timestamp, ext)))
        };

        // Get original file info for metadata
        let src_info = tokio::fs::metadata(file_path).await.ok();

        info!(
            file = %file_path.display(),
            backup = %backup_path.display(),
            "Creating backup"
        );

        // Write backup content with original permissions
        let perm = src_info.as_ref().map(|m| m.permissions());
        tokio::fs::write(&backup_path, original_content)
            .await
            .map_err(|e| TranslateError::Io(format!("Failed to write backup file: {e}")))?;

        if let Some(perm) = perm {
            let _ = tokio::fs::set_permissions(&backup_path, perm).await;
        }

        debug!(
            file = %file_path.display(),
            backup = %backup_path.display(),
            "Backup created"
        );

        Ok(backup_path)
    }

    /// Set preview only mode
    pub async fn set_preview_only_mode(&self, preview_only: bool) {
        let mut config = self.config.write().await;
        config.preview_only = preview_only;
    }

    /// Set backup mode
    pub async fn set_backup_mode(&self, backup: bool) {
        let mut config = self.config.write().await;
        config.backup = backup;
    }

    /// Get current config
    pub async fn config(&self) -> Result<WriterConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }
}

/// Detect line ending style
pub fn detect_line_ending(content: &str) -> &str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Normalize line endings
pub fn normalize_line_ending(content: &str, line_ending: &str) -> String {
    if line_ending == "\r\n" {
        content.replace("\r\n", "\n").replace("\n", "\r\n")
    } else {
        content.replace("\r\n", "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::Position;

    #[test]
    fn test_detect_line_ending() {
        assert_eq!(detect_line_ending("line1\nline2"), "\n");
        assert_eq!(detect_line_ending("line1\r\nline2"), "\r\n");
        assert_eq!(detect_line_ending("line1\r\nline2\nline3"), "\r\n");
    }

    #[test]
    fn test_normalize_line_ending() {
        assert_eq!(
            normalize_line_ending("line1\nline2", "\r\n"),
            "line1\r\nline2"
        );
        assert_eq!(
            normalize_line_ending("line1\r\nline2", "\n"),
            "line1\nline2"
        );
    }

    #[tokio::test]
    async fn test_file_writer_write() {
        let config = WriterConfig::default();
        let writer = FileWriter::new(config);

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_writer.txt");
        let content = b"Hello world";

        // Create the file first
        tokio::fs::write(&file_path, content).await.unwrap();

        let file = File::new(file_path.clone(), content.to_vec(), "UTF-8");

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: crate::core::models::NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("Hello".to_string()),
        }];

        units[0].set_translated("你好");

        let result = writer.write(&file, &units).await;
        assert!(result.is_ok());

        // Cleanup
        let _ = tokio::fs::remove_file(&file_path).await;
    }

    #[tokio::test]
    async fn test_file_writer_preview_mode() {
        let config = WriterConfig {
            preview_only: true,
            ..Default::default()
        };
        let writer = FileWriter::new(config);

        let file = File::new(PathBuf::from("test.txt"), b"Hello world".to_vec(), "UTF-8");

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: crate::core::models::NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("Hello".to_string()),
        }];

        units[0].set_translated("你好");

        // Should not fail in preview mode
        let result = writer.write(&file, &units).await;
        assert!(result.is_ok());
    }
}
