use clap::Parser;
use tracing::info;

use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::{Result, TranslateError},
    translator::ProviderType,
};

use super::Command;

#[derive(Parser, Debug)]
pub struct ValidateArgs;

impl Command for ValidateArgs {
    fn execute(&self, global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()> {
        info!("Validating configuration");
        validate_config(global_config, project_config)?;
        info!("Configuration is valid");

        Ok(())
    }
}

fn validate_config(global: &GlobalConfig, project: &ProjectConfig) -> Result<()> {
    if project.translate.target_lang.is_empty() {
        return Err(TranslateError::Config(
            "Target language cannot be empty".to_string(),
        ));
    }

    match project.translate.provider {
        ProviderType::DeepLX => {
            info!("Using DeepLX provider at: {}", global.deeplx.api_url);
        }
        ProviderType::LLM => {
            if global.llm.providers.is_empty() {
                return Err(TranslateError::Config(
                    "No LLM providers configured".to_string(),
                ));
            }
        }
        ProviderType::Tencent => {
            if global.tencent.secret_id.is_none() || global.tencent.secret_key.is_none() {
                return Err(TranslateError::Config(
                    "Tencent Cloud requires secret_id and secret_key".to_string(),
                ));
            }
        }
    }

    Ok(())
}
