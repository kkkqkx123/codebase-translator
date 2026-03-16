//! File writer implementation
//!
//! This module provides a safe file writer with encoding conversion,
//! preview mode, backup mechanism, and atomic file writing.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};

/// Configuration for file writer
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Preview mode - only show changes without writing
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
    /// Create a new writer config with validation
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(ref dir) = self.backup_dir {
            if !dir.is_absolute() {
                return Err(TranslateError::Config(
                    "Backup directory must be absolute".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// File writer for writing translated content
#[derive(Debug, Clone)]
pub struct FileWriter {
    config: Arc<RwLock<WriterConfig>>,
}

impl FileWriter {
    /// Create a new file writer
    pub fn new(config: WriterConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Write translations to file
    pub fn write(
        &self,
        file: &File,
        units: &[TranslationUnit],
        results: &HashMap<String, String>,
    ) -> Result<()> {
        info!(
            file = %file.path.display(),
            translation_units = units.len(),
            "Starting file write"
        );

        let config = self.config.read().map_err(|_| {
            TranslateError::Lock("Failed to acquire read lock on config".to_string())
        })?;

        if config.preview_only {
            return self.write_preview(file, units, results);
        }

        let content = String::from_utf8_lossy(&file.content);
        let line_ending = detect_line_ending(&content);

        let modified_content = self.apply_translations(&content, units, results);
        let modified_content = normalize_line_ending(&modified_content, line_ending);

        self.write_file_atomically(file, &content, &modified_content)?;

        info!(file = %file.path.display(), "File write completed successfully");
        Ok(())
    }

    /// Write preview of translations (without modifying file)
    pub fn write_preview(
        &self,
        file: &File,
        units: &[TranslationUnit],
        results: &HashMap<String, String>,
    ) -> Result<()> {
        println!("\n=== File: {} ===", file.path.display());

        for unit in units {
            if let Some(translated) = results.get(&unit.id) {
                println!("\n[{}] Line {}", unit.node_type, unit.start_pos.line);
                println!("Original:   {}", unit.content);
                println!("Translated: {}", translated);
            }
        }

        Ok(())
    }

    /// Apply translations to content
    fn apply_translations(
        &self,
        content: &str,
        units: &[TranslationUnit],
        results: &HashMap<String, String>,
    ) -> String {
        if units.is_empty() {
            return content.to_string();
        }

        if self.is_markdown_file(content) {
            return self.apply_markdown_translations(content, units, results);
        }

        let mut unit_map: HashMap<usize, Vec<&TranslationUnit>> = HashMap::new();
        for unit in units {
            if unit.start_pos.line >= 1 {
                unit_map.entry(unit.start_pos.line).or_default().push(unit);
            }
        }

        let line_ending = detect_line_ending(content);
        let lines: Vec<&str> = content.split('\n').collect();

        let mut builder = String::with_capacity(content.len());

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(line_units) = unit_map.get(&(line_num + 1)) {
                builder.push_str(&self.apply_translations_to_line(line, line_units, results));
            } else {
                builder.push_str(line);
            }
            if line_num < lines.len() - 1 {
                builder.push_str(line_ending);
            }
        }

        builder
    }

    /// Apply translations to a single line
    fn apply_translations_to_line(
        &self,
        line: &str,
        units: &[&TranslationUnit],
        results: &HashMap<String, String>,
    ) -> String {
        if units.is_empty() {
            return line.to_string();
        }

        #[derive(Debug)]
        struct Replacement {
            start_char: usize,
            end_char: usize,
            text: String,
        }

        let mut replacements: Vec<Replacement> = units
            .iter()
            .filter_map(|unit| {
                results.get(&unit.id).map(|translated| Replacement {
                    start_char: unit.start_pos.column.saturating_sub(1),
                    end_char: unit.end_pos.column.saturating_sub(1),
                    text: translated.clone(),
                })
            })
            .collect();

        if replacements.is_empty() {
            return line.to_string();
        }

        replacements.sort_by_key(|r| r.start_char);

        let runes: Vec<char> = line.chars().collect();
        let mut result = String::with_capacity(line.len());
        let mut last_end = 0;

        for repl in replacements {
            let start_char = repl.start_char;
            let end_char = if repl.end_char > runes.len() {
                runes.len()
            } else {
                repl.end_char
            };

            if start_char >= end_char {
                continue;
            }

            if start_char > last_end {
                result.extend(&runes[last_end..start_char]);
            }
            result.push_str(&repl.text);
            last_end = end_char;
        }

        if last_end < runes.len() {
            result.extend(&runes[last_end..]);
        }

        result
    }

    /// Check if content appears to be markdown
    fn is_markdown_file(&self, content: &str) -> bool {
        content.contains("# ") || content.contains("## ") || content.contains("### ")
    }

    /// Apply translations for markdown files
    fn apply_markdown_translations(
        &self,
        content: &str,
        units: &[TranslationUnit],
        results: &HashMap<String, String>,
    ) -> String {
        if units.is_empty() {
            return content.to_string();
        }

        let mut unit_map: HashMap<usize, Vec<&TranslationUnit>> = HashMap::new();
        for unit in units {
            if unit.start_pos.line >= 1 {
                unit_map.entry(unit.start_pos.line).or_default().push(unit);
            }
        }

        let line_ending = detect_line_ending(content);
        let lines: Vec<&str> = content.split('\n').collect();

        let mut builder = String::with_capacity(content.len());

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(line_units) = unit_map.get(&(line_num + 1)) {
                builder
                    .push_str(&self.apply_markdown_translations_to_line(line, line_units, results));
            } else {
                builder.push_str(line);
            }
            if line_num < lines.len() - 1 {
                builder.push_str(line_ending);
            }
        }

        builder
    }

    /// Apply translations to a markdown line
    fn apply_markdown_translations_to_line(
        &self,
        line: &str,
        units: &[&TranslationUnit],
        results: &HashMap<String, String>,
    ) -> String {
        if units.is_empty() {
            return line.to_string();
        }

        #[derive(Debug)]
        struct Replacement {
            start_char: usize,
            end_char: usize,
            text: String,
        }

        let mut replacements: Vec<Replacement> = units
            .iter()
            .filter_map(|unit| {
                results.get(&unit.id).map(|translated| Replacement {
                    start_char: unit.start_pos.column.saturating_sub(1),
                    end_char: unit.end_pos.column.saturating_sub(1),
                    text: translated.clone(),
                })
            })
            .collect();

        if replacements.is_empty() {
            return line.to_string();
        }

        replacements.sort_by_key(|r| r.start_char);

        let runes: Vec<char> = line.chars().collect();
        let mut result = String::with_capacity(line.len());
        let mut last_end = 0;

        for repl in replacements {
            let start_char = repl.start_char;
            let end_char = if repl.end_char > runes.len() {
                runes.len()
            } else {
                repl.end_char
            };

            if start_char >= end_char {
                continue;
            }

            if start_char > last_end {
                result.extend(&runes[last_end..start_char]);
            }
            result.push_str(&repl.text);
            last_end = end_char;
        }

        if last_end < runes.len() {
            result.extend(&runes[last_end..]);
        }

        result
    }

    /// Write file atomically with backup support
    fn write_file_atomically(
        &self,
        file: &File,
        original_content: &str,
        modified_content: &str,
    ) -> Result<()> {
        let file_path = &file.path;
        debug!(file = %file_path.display(), "Starting atomic file write");

        let config = self.config.read().map_err(|_| {
            TranslateError::Lock("Failed to acquire read lock on config".to_string())
        })?;

        // Create backup if enabled
        if config.backup {
            if let Err(e) = self.create_backup(file_path, original_content) {
                warn!(
                    file = %file_path.display(),
                    error = %e,
                    "Failed to create backup, continuing without backup"
                );
            }
        }

        // Create temporary file
        let temp_path = file_path.with_extension("tmp");

        // Write to temporary file
        let mut temp_file = fs::File::create(&temp_path).map_err(|e| {
            error!(
                file = %file_path.display(),
                error = %e,
                "Failed to create temporary file"
            );
            TranslateError::Io(format!("Failed to create temporary file: {e}"))
        })?;

        temp_file
            .write_all(modified_content.as_bytes())
            .map_err(|e| {
                error!(
                    file = %file_path.display(),
                    error = %e,
                    "Failed to write temporary file"
                );
                let _ = fs::remove_file(&temp_path);
                TranslateError::Io(format!("Failed to write temporary file: {e}"))
            })?;

        drop(temp_file);

        // Preserve metadata
        if let Err(e) = self.preserve_metadata(file_path, &temp_path) {
            error!(
                file = %file_path.display(),
                error = %e,
                "Failed to preserve file metadata"
            );
            let _ = fs::remove_file(&temp_path);
            return Err(TranslateError::Io(format!(
                "Failed to preserve file metadata: {e}"
            )));
        }

        // Atomic rename
        fs::rename(&temp_path, file_path).map_err(|e| {
            error!(
                file = %file_path.display(),
                error = %e,
                "Failed to replace original file"
            );
            let _ = fs::remove_file(&temp_path);
            TranslateError::Io(format!("Failed to replace original file: {e}"))
        })?;

        debug!(file = %file_path.display(), "Atomic file write completed");
        Ok(())
    }

    /// Preserve file metadata (permissions)
    fn preserve_metadata(&self, src_path: &Path, dst_path: &Path) -> Result<()> {
        let metadata = fs::metadata(src_path)
            .map_err(|e| TranslateError::Io(format!("Failed to get file metadata: {e}")))?;

        let permissions = metadata.permissions();
        fs::set_permissions(dst_path, permissions)
            .map_err(|e| TranslateError::Io(format!("Failed to set file permissions: {e}")))?;

        Ok(())
    }

    /// Create backup of original file
    pub fn create_backup(&self, file_path: &Path, original_content: &str) -> Result<PathBuf> {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| TranslateError::Io(format!("Failed to get timestamp: {e}")))?;
        let timestamp_str = format!("{}", timestamp.as_secs());

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let base = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");

        let config = self.config.read().map_err(|_| {
            TranslateError::Lock("Failed to acquire read lock on config".to_string())
        })?;

        // Determine backup path
        let backup_path: PathBuf = if let Some(ref backup_dir) = config.backup_dir {
            // Create backup directory if needed
            fs::create_dir_all::<&Path>(backup_dir).map_err(|e| {
                TranslateError::Io(format!("Failed to create backup directory: {e}"))
            })?;

            // Preserve relative directory structure
            let rel_dir = file_path.parent().map(|p| {
                if let Ok(rel) = p.strip_prefix(".") {
                    rel
                } else {
                    p
                }
            });

            if let Some(rel_dir) = rel_dir {
                let target_dir = backup_dir.join(rel_dir);
                fs::create_dir_all::<&Path>(&target_dir).map_err(|e| {
                    TranslateError::Io(format!("Failed to create backup subdirectory: {e}"))
                })?;
                target_dir.join(format!("{}_{}.bak.{}", base, timestamp_str, ext))
            } else {
                backup_dir.join(format!("{}_{}.bak.{}", base, timestamp_str, ext))
            }
        } else {
            // Same directory as original file
            file_path
                .parent()
                .map(|p| p.join(format!("{}_{}.bak.{}", base, timestamp_str, ext)))
                .unwrap_or_else(|| PathBuf::from(format!("{}_{}.bak.{}", base, timestamp_str, ext)))
        };

        // Get original file info for metadata
        let src_info = fs::metadata(file_path).ok();

        // Write backup content with original permissions
        let perm = src_info.as_ref().map(|m| m.permissions());
        fs::write(&backup_path, original_content)
            .map_err(|e| TranslateError::Io(format!("Failed to write backup file: {e}")))?;

        if let Some(perm) = perm {
            let _ = fs::set_permissions(&backup_path, perm);
        }

        info!(
            file = %file_path.display(),
            backup = %backup_path.display(),
            "Backup created"
        );

        Ok(backup_path)
    }

    /// Set preview mode
    pub fn set_preview_mode(&self, preview: bool) {
        if let Ok(mut config) = self.config.write() {
            config.preview_only = preview;
        }
    }

    /// Set backup mode
    pub fn set_backup_mode(&self, backup: bool) {
        if let Ok(mut config) = self.config.write() {
            config.backup = backup;
        }
    }

    /// Get current config
    pub fn config(&self) -> Result<WriterConfig> {
        self.config
            .read()
            .map(|c| c.clone())
            .map_err(|_| TranslateError::Lock("Failed to acquire read lock on config".to_string()))
    }
}

/// Detect line ending style
fn detect_line_ending(content: &str) -> &str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Normalize line endings
fn normalize_line_ending(content: &str, line_ending: &str) -> String {
    if line_ending == "\r\n" {
        content.replace("\r\n", "\n").replace("\n", "\r\n")
    } else {
        content.replace("\r\n", "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{NodeType, Position};

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

    #[test]
    fn test_file_writer_apply_translations() {
        let config = WriterConfig::default();
        let writer = FileWriter::new(config);

        let content = "Hello world\nThis is a test";
        let units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
        }];

        let mut results = HashMap::new();
        results.insert("1".to_string(), "你好".to_string());

        let result = writer.apply_translations(content, &units, &results);
        assert!(result.contains("你好"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_file_writer_preview_mode() {
        let config = WriterConfig {
            preview_only: true,
            ..Default::default()
        };
        let writer = FileWriter::new(config);

        let file = File::new(PathBuf::from("test.txt"), b"Hello world".to_vec(), "UTF-8");

        let units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
        }];

        let mut results = HashMap::new();
        results.insert("1".to_string(), "你好".to_string());

        // Should not fail in preview mode
        let result = writer.write(&file, &units, &results);
        assert!(result.is_ok());
    }
}
