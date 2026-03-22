use clap::Parser;
use std::path::PathBuf;
use tracing::info;

use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::{Result, TranslateError},
    reporter::{create_reporter, ReportFormat},
    workflow::TranslationWorkflow,
};

use crate::{NAME, VERSION};

use super::Command;

#[derive(Parser, Debug)]
pub struct TranslateArgs {
    #[arg(default_value = ".", value_name = "PATH")]
    pub path: String,

    #[arg(short, long, value_name = "LANG")]
    pub target_lang: Option<String>,

    #[arg(short, long, value_name = "LANGS")]
    pub source_langs: Option<String>,

    #[arg(short, long, value_name = "PROVIDER")]
    pub provider: Option<String>,

    #[arg(long, value_name = "PATTERNS")]
    pub include: Option<String>,

    #[arg(long, value_name = "PATTERNS")]
    pub exclude: Option<String>,
}

impl Command for TranslateArgs {
    fn execute(&self, global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()> {
        info!(name = NAME, version = VERSION, "Starting application");

        let mut project_config = project_config.clone();

        if let Some(lang) = &self.target_lang {
            project_config.translate.target_lang = lang.clone();
        }
        if let Some(langs) = &self.source_langs {
            project_config.translate.source_langs =
                langs.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Some(prov) = &self.provider {
            project_config.translate.provider =
                prov.parse().map_err(TranslateError::InvalidArgument)?;
        }
        if let Some(inc) = &self.include {
            project_config.include.patterns =
                inc.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Some(exc) = &self.exclude {
            project_config.exclude.patterns =
                exc.split(',').map(|s| s.trim().to_string()).collect();
        }

        info!(
            path = %self.path,
            "Translating directory"
        );
        info!(
            target_lang = %project_config.translate.target_lang,
            provider = %project_config.translate.provider,
            "Translation configuration"
        );

        let reporter = create_reporter();

        let workflow = TranslationWorkflow::from_configs_with_path(
            global_config.clone(),
            project_config,
            &self.path,
        )
        .with_reporter(reporter.clone());
        let result = workflow.execute()?;

        // Save translation report to target project's .translator directory
        let report_dir = PathBuf::from(&self.path).join(".translator");
        match reporter.save_report_with_template(
            &report_dir,
            "report_{timestamp}.txt",
            &result.stats,
            ReportFormat::Text,
        ) {
            Ok(path) => {
                info!(path = %path.display(), "Translation report saved");
            }
            Err(e) => {
                info!(error = %e, "Failed to save translation report");
            }
        }

        Ok(())
    }

    fn get_project_path(&self) -> Option<&str> {
        Some(&self.path)
    }
}
