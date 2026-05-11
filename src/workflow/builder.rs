//! Workflow builder for coordinating component creation
//!
//! This module provides a builder pattern for creating all components
//! needed for the translation workflow.

use crate::{
    cache::{CacheFactory, DirectoryCache},
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    encoding::{Detector, Encoder},
    parser::ParserFactory,
    reporter::stats::SharedStats,
    reporter::{create_reporter_with_stats, Reporter},
    translator::create_translation_service_with_stats,
    writer::WriterFactory,
};
use std::sync::Arc;

/// Workflow components container
pub struct WorkflowComponents {
    pub cache: DirectoryCache,
    pub translator: crate::translator::service::TranslationService,
    pub parser: crate::parser::ParserCoordinator,
    pub writer: crate::writer::FileWriter,
    pub detector: Detector,
    pub encoder: Encoder,
    pub shared_stats: Arc<SharedStats>,
}

/// Builder for creating workflow components
pub struct WorkflowBuilder {
    global_config: GlobalConfig,
    project_config: ProjectConfig,
    root_path: String,
    reporter: Option<Arc<dyn Reporter>>,
}

impl WorkflowBuilder {
    /// Create a new workflow builder
    pub fn new(
        global_config: GlobalConfig,
        project_config: ProjectConfig,
        root_path: impl Into<String>,
    ) -> Self {
        Self {
            global_config,
            project_config,
            root_path: root_path.into(),
            reporter: None,
        }
    }

    /// Set the reporter for this workflow
    pub fn with_reporter(mut self, reporter: Arc<dyn Reporter>) -> Self {
        self.reporter = Some(reporter);
        self
    }

    /// Build all workflow components
    pub fn build(&self) -> Result<WorkflowComponents> {
        let cache = CacheFactory::create_directory(&self.project_config.cache, &self.root_path)?;

        let translator =
            create_translation_service_with_stats(&self.global_config, &self.project_config, None)?;
        let parser = ParserFactory::create(&self.project_config)?;
        let writer =
            WriterFactory::from_project_config(&self.project_config, Some(&self.root_path))?;
        let detector = Detector::default();
        let encoder = Encoder::default();
        let shared_stats = Arc::new(SharedStats::new());

        Ok(WorkflowComponents {
            cache,
            translator,
            parser,
            writer,
            detector,
            encoder,
            shared_stats,
        })
    }

    /// Build all workflow components with reporter
    pub fn build_with_reporter(&self) -> Result<(WorkflowComponents, Arc<dyn Reporter>)> {
        let cache = CacheFactory::create_directory(&self.project_config.cache, &self.root_path)?;

        let shared_stats = Arc::new(SharedStats::new());
        let translator = create_translation_service_with_stats(
            &self.global_config,
            &self.project_config,
            Some(shared_stats.clone()),
        )?;
        let parser = ParserFactory::create(&self.project_config)?;
        let writer =
            WriterFactory::from_project_config(&self.project_config, Some(&self.root_path))?;
        let detector = Detector::default();
        let encoder = Encoder::default();
        let reporter = create_reporter_with_stats(shared_stats.clone());

        let components = WorkflowComponents {
            cache,
            translator,
            parser,
            writer,
            detector,
            encoder,
            shared_stats,
        };

        Ok((components, reporter))
    }

    /// Get the global config
    pub fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }

    /// Get the project config
    pub fn project_config(&self) -> &ProjectConfig {
        &self.project_config
    }

    /// Get the root path
    pub fn root_path(&self) -> &str {
        &self.root_path
    }

    /// Get the reporter
    pub fn reporter(&self) -> Option<&Arc<dyn Reporter>> {
        self.reporter.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder_creation() {
        let global_config = GlobalConfig::default();
        let project_config = ProjectConfig::default();
        let builder = WorkflowBuilder::new(global_config, project_config, ".");
        assert_eq!(builder.root_path(), ".");
    }

    #[test]
    fn test_workflow_builder_with_reporter() {
        let global_config = GlobalConfig::default();
        let project_config = ProjectConfig::default();
        let builder = WorkflowBuilder::new(global_config, project_config, ".");
        let reporter = Arc::new(crate::reporter::default::DefaultReporter::new());
        let builder = builder.with_reporter(reporter);
        assert!(builder.reporter().is_some());
    }
}
