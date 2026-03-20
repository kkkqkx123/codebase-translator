//! File processor for translation workflow
//!
//! Handles the processing of individual files including parsing,
//! translation, caching, and writing.

use crate::{
    cache::{Cache, CacheEntry},
    config::project::ProjectConfig,
    core::error::Result,
    core::models::{File, TranslationStats},
    encoding::{Detector, Encoder},
    parser::coordinator::ParserCoordinator,
    translator::service::TranslationService,
    utils::hash::calculate_hash,
    writer::file::FileWriter,
};
use std::path::Path;
use tracing::{debug, info};

/// Result of processing a single file
#[derive(Debug, Default)]
pub struct FileProcessResult {
    /// Total number of translation units
    pub total_units: usize,
    /// Number of units that were translated
    pub translated_units: usize,
    /// Number of files from cache (cache hit)
    pub cached_files: usize,
    /// Number of units skipped (should_translate = false)
    pub skipped_units: usize,
    /// Number of errors
    pub errors: usize,
    /// Whether the file was written
    pub was_written: bool,
}

impl FileProcessResult {
    /// Merge another result into this one
    pub fn merge(&mut self, other: &FileProcessResult) {
        self.total_units += other.total_units;
        self.translated_units += other.translated_units;
        self.cached_files += other.cached_files;
        self.skipped_units += other.skipped_units;
        self.errors += other.errors;
    }
}

impl From<FileProcessResult> for TranslationStats {
    fn from(result: FileProcessResult) -> Self {
        TranslationStats {
            total_files: 1,
            total_units: result.total_units,
            translated_units: result.translated_units,
            cached_files: result.cached_files,
            skipped_units: result.skipped_units,
            errors: result.errors,
        }
    }
}

/// Processor for individual files
pub struct FileProcessor<'a> {
    cache: &'a Box<dyn Cache>,
    translator: &'a TranslationService,
    parser: &'a ParserCoordinator,
    writer: &'a FileWriter,
    detector: &'a Detector,
    encoder: &'a Encoder,
    project_config: &'a ProjectConfig,
}

impl<'a> FileProcessor<'a> {
    /// Create a new file processor
    pub fn new(
        cache: &'a Box<dyn Cache>,
        translator: &'a TranslationService,
        parser: &'a ParserCoordinator,
        writer: &'a FileWriter,
        detector: &'a Detector,
        encoder: &'a Encoder,
        project_config: &'a ProjectConfig,
    ) -> Self {
        Self {
            cache,
            translator,
            parser,
            writer,
            detector,
            encoder,
            project_config,
        }
    }

    /// Process a single file
    pub fn process(&self, file_path: &Path, modified_time: i64) -> Result<FileProcessResult> {
        let mut result = FileProcessResult::default();

        debug!(
            file = %file_path.display(),
            "Processing file"
        );

        let content = std::fs::read(file_path)?;

        let file_hash = calculate_hash(&content);

        let cached_entry = self.cache.get(&file_hash)?;

        if let Some(entry) = cached_entry {
            if entry.is_valid(modified_time) && entry.is_translated {
                debug!("Cache hit - file already translated, skipping");
                result.cached_files = 1;
                return Ok(result);
            } else {
                debug!("Cache expired or file modified, re-translating");
            }
        } else {
            debug!("Cache miss, translating file");
        }

        let encoding_result = self.detector.detect_bytes(&content)?;
        let encoding = encoding_result.encoding;

        let utf8_content = if encoding != "UTF-8" {
            debug!(
                original_encoding = %encoding,
                "Converting to UTF-8"
            );
            self.encoder.to_utf8(&content, &encoding)?.into_bytes()
        } else {
            content.clone()
        };

        let file = File::new(file_path.to_path_buf(), utf8_content.clone(), "UTF-8");
        let mut units = self.parser.parse_file(&file)?;
        result.total_units = units.len();

        if units.is_empty() {
            debug!("No translatable content found");
            return Ok(result);
        }

        let units_to_translate: Vec<_> = units.iter().filter(|u| u.should_translate).collect();
        let num_to_translate = units_to_translate.len();

        if num_to_translate == 0 {
            debug!("All units filtered, nothing to translate");
            result.skipped_units = units.len();
            return Ok(result);
        }

        let texts: Vec<String> = units_to_translate
            .iter()
            .map(|u| u.content.clone())
            .collect();

        debug!(units_count = texts.len(), "Translating units");

        let translated_texts = self
            .translator
            .translate_batch(&texts, &self.project_config.translate.target_lang)?;

        let mut translate_idx = 0;
        for unit in units.iter_mut() {
            if unit.should_translate {
                if let Some(translated) = translated_texts.get(translate_idx) {
                    unit.set_translated(translated.clone());
                    translate_idx += 1;
                }
            }
        }

        result.translated_units = num_to_translate;

        // Check if any units were actually translated (content changed)
        let has_translations = units.iter().any(|u| {
            u.translated
                .as_ref()
                .map(|t| t != &u.content)
                .unwrap_or(false)
        });

        if !has_translations {
            debug!("No actual translations produced, skipping file write");
            return Ok(result);
        }

        if !self.project_config.writer.dry_run {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { self.writer.write(&file, &units).await })?;
            result.was_written = true;
        } else {
            info!("Dry run mode - not writing changes");
            for unit in &units {
                if let Some(translated) = &unit.translated {
                    info!(
                        node_type = %unit.node_type,
                        original = %unit.content,
                        translated = %translated,
                        "Translation preview"
                    );
                }
            }
        }

        let mut cache_entry = CacheEntry::new(
            &file_hash,
            file_path.to_string_lossy(),
            modified_time,
            &self.project_config.cache.mode.to_string(),
            "",
        );
        cache_entry.mark_as_translated();

        self.cache.set(&cache_entry)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_process_result_merge() {
        let mut result1 = FileProcessResult {
            total_units: 10,
            translated_units: 5,
            cached_files: 1,
            skipped_units: 2,
            errors: 0,
            was_written: true,
        };

        let result2 = FileProcessResult {
            total_units: 8,
            translated_units: 4,
            cached_files: 1,
            skipped_units: 2,
            errors: 1,
            was_written: false,
        };

        result1.merge(&result2);

        assert_eq!(result1.total_units, 18);
        assert_eq!(result1.translated_units, 9);
        assert_eq!(result1.cached_files, 2);
        assert_eq!(result1.skipped_units, 4);
        assert_eq!(result1.errors, 1);
        assert!(result1.was_written); // Should remain true
    }

    #[test]
    fn test_file_process_result_default() {
        let result = FileProcessResult::default();

        assert_eq!(result.total_units, 0);
        assert_eq!(result.translated_units, 0);
        assert_eq!(result.cached_files, 0);
        assert_eq!(result.skipped_units, 0);
        assert_eq!(result.errors, 0);
        assert!(!result.was_written);
    }
}
