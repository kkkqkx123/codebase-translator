//! Translation workflow executor
//!
//! Orchestrates the complete translation workflow from file scanning to completion.

use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    core::models::{FileEntry, TranslationStats},
    reporter::Reporter,
    scanner::r#trait::{ScanOptions, Scanner},
    scanner::FSScanner,
    workflow::file_processor::FileProcessor,
    workflow::WorkflowBuilder,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Configuration for the translation workflow
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    /// Root path to scan
    pub root_path: String,
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Whether to follow symlinks
    pub follow_symlinks: bool,
    /// Whether to respect gitignore
    pub respect_gitignore: bool,
    /// Gitignore patterns
    pub gitignore_patterns: Vec<String>,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            root_path: ".".to_string(),
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![],
            follow_symlinks: false,
            respect_gitignore: true,
            gitignore_patterns: vec![],
        }
    }
}

impl From<&ProjectConfig> for WorkflowConfig {
    fn from(config: &ProjectConfig) -> Self {
        Self {
            root_path: ".".to_string(),
            include_patterns: config.include.patterns.clone(),
            exclude_patterns: config.exclude.patterns.clone(),
            follow_symlinks: false,
            respect_gitignore: config.exclude.respect_gitignore,
            gitignore_patterns: config.exclude.gitignore_patterns.clone(),
        }
    }
}

/// Result of a workflow execution
#[derive(Debug, Default)]
pub struct WorkflowResult {
    /// Total statistics
    pub stats: TranslationStats,
    /// Number of files processed
    pub files_processed: usize,
    /// Duration of execution
    pub duration_secs: f64,
}

/// Translation workflow executor
pub struct TranslationWorkflow {
    global_config: GlobalConfig,
    project_config: ProjectConfig,
    workflow_config: WorkflowConfig,
    reporter: Option<Arc<dyn Reporter>>,
}

impl TranslationWorkflow {
    /// Create a new translation workflow
    pub fn new(
        global_config: GlobalConfig,
        project_config: ProjectConfig,
        workflow_config: WorkflowConfig,
    ) -> Self {
        Self {
            global_config,
            project_config,
            workflow_config,
            reporter: None,
        }
    }

    /// Create a workflow from config with a specific root path
    pub fn from_configs_with_path(
        global_config: GlobalConfig,
        project_config: ProjectConfig,
        root_path: impl Into<String>,
    ) -> Self {
        let mut workflow_config = WorkflowConfig::from(&project_config);
        workflow_config.root_path = root_path.into();

        Self {
            global_config,
            project_config,
            workflow_config,
            reporter: None,
        }
    }

    /// Set the reporter for this workflow
    pub fn with_reporter(mut self, reporter: Arc<dyn Reporter>) -> Self {
        self.reporter = Some(reporter);
        self
    }

    /// Execute the complete translation workflow
    pub fn execute(&self) -> Result<WorkflowResult> {
        let start_time = std::time::Instant::now();
        let mut result = WorkflowResult::default();

        info!(
            root_path = %self.workflow_config.root_path,
            include_patterns = self.workflow_config.include_patterns.len(),
            exclude_patterns = self.workflow_config.exclude_patterns.len(),
            respect_gitignore = self.workflow_config.respect_gitignore,
            follow_symlinks = self.workflow_config.follow_symlinks,
            "Starting translation workflow"
        );

        info!(
            path = %self.workflow_config.root_path,
            "Step 1: Scanning directory"
        );

        let files = self.scan_files()?;
        info!(files_count = files.len(), "File scan completed");

        if files.is_empty() {
            info!("No files found to translate");
            return Ok(result);
        }

        let builder = WorkflowBuilder::new(
            self.global_config.clone(),
            self.project_config.clone(),
            self.workflow_config.root_path.clone(),
        );

        let components = builder.build()?;

        let processor = FileProcessor::new(
            &components.cache,
            &components.translator,
            &components.parser,
            &components.writer,
            &components.detector,
            &components.encoder,
            &self.project_config,
            self.reporter.clone(),
        );

        for (idx, file_entry) in files.iter().enumerate() {
            debug!(
                index = idx + 1,
                total = files.len(),
                file = %file_entry.path.display(),
                "Processing file"
            );

            if let Some(ref reporter) = self.reporter {
                reporter.report_progress(idx + 1, files.len());
            }

            let modified_time = file_entry
                .modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            match processor.process(&file_entry.path, modified_time) {
                Ok(file_result) => {
                    result.stats.merge(&file_result.into());
                    result.files_processed += 1;
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        file = %file_entry.path.display(),
                        "Failed to process file, continuing"
                    );
                    result.stats.errors += 1;
                    if let Some(ref reporter) = self.reporter {
                        reporter.report_error(&file_entry.path, &e);
                    }
                }
            }
        }

        let elapsed = start_time.elapsed();
        result.duration_secs = elapsed.as_secs_f64();

        info!(
            duration_ms = elapsed.as_millis(),
            files_processed = result.files_processed,
            "Workflow execution completed"
        );

        if let Some(ref reporter) = self.reporter {
            reporter.finalize();
        }

        // Print summary
        info!("========================================");
        info!(
            duration_secs = result.duration_secs,
            "Translation completed"
        );
        info!(
            total_files = result.stats.total_files,
            cached_files = result.stats.cached_files,
            processed_files = result.stats.total_files - result.stats.cached_files,
            "Files"
        );
        info!(
            total_units = result.stats.total_units,
            translated_units = result.stats.translated_units,
            skipped_units = result.stats.skipped_units,
            "Units"
        );
        info!(errors = result.stats.errors, "Errors");
        info!("========================================");

        Ok(result)
    }

    /// Scan directory for files to translate
    fn scan_files(&self) -> Result<Vec<FileEntry>> {
        let scanner = FSScanner::new();

        let opts = ScanOptions {
            root_path: self.workflow_config.root_path.clone(),
            include_patterns: self.workflow_config.include_patterns.clone(),
            exclude_patterns: self.workflow_config.exclude_patterns.clone(),
            follow_symlinks: self.workflow_config.follow_symlinks,
            respect_gitignore: self.workflow_config.respect_gitignore,
            gitignore_patterns: self.workflow_config.gitignore_patterns.clone(),
            gitignore_path: None,
        };

        scanner.scan(opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_config_default() {
        let config = WorkflowConfig::default();

        assert_eq!(config.root_path, ".");
        assert_eq!(config.include_patterns, vec!["**/*"]);
        assert!(config.exclude_patterns.is_empty());
        assert!(!config.follow_symlinks);
        assert!(config.respect_gitignore);
        assert!(config.gitignore_patterns.is_empty());
    }

    #[test]
    fn test_workflow_result_default() {
        let result = WorkflowResult::default();

        assert_eq!(result.files_processed, 0);
        assert_eq!(result.duration_secs, 0.0);
        assert_eq!(result.stats.total_files, 0);
    }
}
