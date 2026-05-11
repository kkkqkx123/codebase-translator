//! File processor for translation workflow
//!
//! Handles the processing of individual files including parsing,
//! translation, caching, and writing.

use crate::{
    cache::{CacheEntry, DirectoryCache},
    config::{calculate_config_hash, project::ProjectConfig},
    core::error::Result,
    core::models::File,
    encoding::{Detector, Encoder},
    parser::coordinator::ParserCoordinator,
    reporter::{Reporter, TranslationStats},
    translator::service::TranslationService,
    utils::hash::calculate_hash,
    writer::file::FileWriter,
};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

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
    /// Number of API calls made
    pub api_calls: usize,
    /// Number of cache misses
    pub cache_misses: usize,
}

impl FileProcessResult {
    /// Merge another result into this one
    pub fn merge(&mut self, other: &FileProcessResult) {
        self.total_units += other.total_units;
        self.translated_units += other.translated_units;
        self.cached_files += other.cached_files;
        self.skipped_units += other.skipped_units;
        self.errors += other.errors;
        self.api_calls += other.api_calls;
        self.cache_misses += other.cache_misses;
    }
}

impl From<FileProcessResult> for TranslationStats {
    fn from(result: FileProcessResult) -> Self {
        let mut stats = TranslationStats::new();
        stats.total_files = 1;
        stats.total_units = result.total_units;
        stats.translated_units = result.translated_units;
        stats.processed_files = if result.cached_files > 0 { 0 } else { 1 };
        stats.cache_hit_count = result.cached_files;
        stats.cache_miss_count = result.cache_misses;
        stats.skipped_files = if result.cached_files > 0
            || (result.skipped_units > 0 && result.translated_units == 0)
        {
            1
        } else {
            0
        };
        stats.error_count = result.errors;
        stats.api_call_count = result.api_calls;

        stats
    }
}

/// Processor for individual files
pub struct FileProcessor<'a> {
    cache: &'a DirectoryCache,
    translator: &'a TranslationService,
    parser: &'a ParserCoordinator,
    writer: &'a FileWriter,
    detector: &'a Detector,
    encoder: &'a Encoder,
    project_config: &'a ProjectConfig,
    reporter: Option<Arc<dyn Reporter>>,
}

impl<'a> FileProcessor<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cache: &'a DirectoryCache,
        translator: &'a TranslationService,
        parser: &'a ParserCoordinator,
        writer: &'a FileWriter,
        detector: &'a Detector,
        encoder: &'a Encoder,
        project_config: &'a ProjectConfig,
        reporter: Option<Arc<dyn Reporter>>,
    ) -> Self {
        Self {
            cache,
            translator,
            parser,
            writer,
            detector,
            encoder,
            project_config,
            reporter,
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

        // Calculate config hash for cache validation
        let config_hash = calculate_config_hash(self.project_config);

        let cached_entry = self.cache.get(&file_hash, &config_hash)?;

        if let Some(entry) = cached_entry {
            if entry.is_valid(modified_time) && entry.is_translated {
                info!(
                    file = %file_path.display(),
                    "Cache hit - file already translated"
                );
                result.cached_files = 1;
                if let Some(ref reporter) = self.reporter {
                    reporter.report_cache_hit();
                    reporter.report_skipped(file_path);
                }
                return Ok(result);
            } else {
                debug!(
                    file = %file_path.display(),
                    "Cache expired or file modified, re-translating"
                );
            }
        } else {
            debug!(
                file = %file_path.display(),
                "Cache miss"
            );
            result.cache_misses = 1;
            if let Some(ref reporter) = self.reporter {
                reporter.report_cache_miss();
            }
        }

        let encoding_result = self.detector.detect_bytes(&content)?;
        let encoding = encoding_result.encoding;

        let utf8_content = if encoding != "UTF-8" {
            info!(
                file = %file_path.display(),
                original_encoding = %encoding,
                "Converting encoding to UTF-8"
            );
            self.encoder.to_utf8(&content, &encoding)?.into_bytes()
        } else {
            content.clone()
        };

        let file = File::new(file_path.to_path_buf(), utf8_content.clone(), "UTF-8");
        let mut units = self.parser.parse_file(&file)?;
        result.total_units = units.len();

        if let Some(ref reporter) = self.reporter {
            reporter.report_file(file_path, result.total_units);
        }

        info!(
            file = %file_path.display(),
            total_units = result.total_units,
            translatable_units = units.iter().filter(|u| u.should_translate).count(),
            "File parsed"
        );

        // Always save cache entry for files that have been processed
        // This allows the file to be skipped on subsequent runs even if it has no translatable content
        let save_cache = || -> Result<()> {
            info!(file = %file_path.display(), "Saving cache entry");
            let mut cache_entry = CacheEntry::new(
                &file_hash,
                file_path.to_string_lossy(),
                modified_time,
                self.project_config.cache.mode.to_string(),
                self.cache.project_fingerprint(),
                &config_hash,
            );
            cache_entry.mark_as_translated();
            self.cache.set(&cache_entry)?;
            info!(file = %file_path.display(), "Cache entry saved successfully");
            Ok(())
        };

        if units.is_empty() {
            debug!("No translatable content found");
            if let Some(ref reporter) = self.reporter {
                reporter.report_skipped(file_path);
            }
            // Save cache even for files with no translatable content
            if let Err(e) = save_cache() {
                warn!(error = %e, "Failed to save cache entry");
            }
            return Ok(result);
        }

        let units_to_translate: Vec<_> = units.iter().filter(|u| u.should_translate).collect();
        let num_to_translate = units_to_translate.len();

        if num_to_translate == 0 {
            debug!("All units filtered, nothing to translate");
            result.skipped_units = units.len();
            if let Some(ref reporter) = self.reporter {
                reporter.report_skipped(file_path);
            }
            // Save cache even for files with no units to translate
            if let Err(e) = save_cache() {
                warn!(error = %e, "Failed to save cache entry");
            }
            return Ok(result);
        }

        // Filter out empty content to avoid sending empty strings to LLM
        let texts: Vec<String> = units_to_translate
            .iter()
            .filter(|u| !u.content.trim().is_empty())
            .map(|u| u.content.clone())
            .collect();

        // Update count after filtering empty content
        let num_non_empty = texts.len();
        if num_non_empty < num_to_translate {
            let empty_count = num_to_translate - num_non_empty;
            warn!(
                file = %file_path.display(),
                empty_count = empty_count,
                "Filtered out translation units with empty content"
            );
        }

        debug!(units_count = texts.len(), "Translating units");

        // Skip translation if all content is empty after filtering
        if num_non_empty == 0 {
            warn!(
                file = %file_path.display(),
                "All translation units have empty content, skipping translation"
            );
            result.skipped_units = units.len();
            if let Some(ref reporter) = self.reporter {
                reporter.report_skipped(file_path);
            }
            // Save cache even for files with empty content
            if let Err(e) = save_cache() {
                warn!(error = %e, "Failed to save cache entry");
            }
            return Ok(result);
        }

        if num_non_empty > 0 {
            info!(
                file = %file_path.display(),
                units_to_translate = num_non_empty,
                empty_filtered = num_to_translate - num_non_empty,
                "Translating units"
            );
        }

        // Get source language for translation - use first source lang or default to "auto"
        let source_lang = self
            .project_config
            .translate
            .source_langs
            .first()
            .map(|s| s.as_str())
            .unwrap_or("auto");

        let batch_result = self.translator.translate_batch_with_result(
            &texts,
            source_lang,
            &self.project_config.translate.target_lang,
        )?;

        // API calls for cost calculation: use actual translated units count
        // This reflects the true cost as LLM APIs charge per unit/token processed
        result.api_calls = batch_result.total_batches; // Now contains actual unit count, not batch count
        if let Some(ref reporter) = self.reporter {
            reporter.report_api_call(batch_result.total_batches);
        }

        info!(
            file = %file_path.display(),
            translated_units = batch_result.results.len(),
            api_calls = batch_result.total_batches,
            "Translation completed"
        );

        // Match translation results to units, skipping empty content
        let mut translate_idx = 0;
        for unit in units.iter_mut() {
            if unit.should_translate && !unit.content.trim().is_empty() {
                if let Some(translated) = batch_result
                    .results
                    .get(translate_idx)
                    .map(|r| r.translated_text.as_str())
                {
                    unit.set_translated(translated.to_string());
                    translate_idx += 1;
                }
            }
        }

        // Update translated count to reflect actual non-empty translations
        result.translated_units = num_non_empty;

        // Check if any units were actually translated (content changed)
        let has_translations = units.iter().any(|u| {
            u.translated
                .as_ref()
                .map(|t| t != &u.content)
                .unwrap_or(false)
        });

        // Save cache entry even if no actual translations were produced
        // This allows the file to be skipped on subsequent runs
        let mut cache_entry = CacheEntry::new(
            &file_hash,
            file_path.to_string_lossy(),
            modified_time,
            self.project_config.cache.mode.to_string(),
            self.cache.project_fingerprint(),
            &config_hash,
        );
        cache_entry.mark_as_translated();

        debug!(
            file = %file_path.display(),
            "Updating cache"
        );
        self.cache.set(&cache_entry)?;

        if !has_translations {
            debug!("No actual translations produced, skipping file write");
            return Ok(result);
        }

        if !self.project_config.writer.preview_only {
            info!(
                file = %file_path.display(),
                "Writing file"
            );
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { self.writer.write(&file, &units).await })?;
            info!(
                file = %file_path.display(),
                "File written successfully"
            );
            result.was_written = true;
        } else {
            info!(
                file = %file_path.display(),
                "Dry run mode - showing preview"
            );
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { self.writer.write(&file, &units).await })?;
        }

        info!(
            file = %file_path.display(),
            total_units = result.total_units,
            translated_units = result.translated_units,
            "File processing completed"
        );

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
            api_calls: 1,
            cache_misses: 1,
        };

        let result2 = FileProcessResult {
            total_units: 8,
            translated_units: 4,
            cached_files: 1,
            skipped_units: 2,
            errors: 1,
            was_written: false,
            api_calls: 1,
            cache_misses: 1,
        };

        result1.merge(&result2);

        assert_eq!(result1.total_units, 18);
        assert_eq!(result1.translated_units, 9);
        assert_eq!(result1.cached_files, 2);
        assert_eq!(result1.skipped_units, 4);
        assert_eq!(result1.errors, 1);
        assert_eq!(result1.api_calls, 2);
        assert_eq!(result1.cache_misses, 2);
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
        assert_eq!(result.api_calls, 0);
        assert_eq!(result.cache_misses, 0);
    }
}
