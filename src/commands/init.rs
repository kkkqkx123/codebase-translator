use clap::Parser;
use tracing::{debug, info};

use crate::{
    config::{global::GlobalConfig, loader::ConfigLoader, project::ProjectConfig},
    core::error::{Result, TranslateError},
};

use super::Command;

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[arg(long, help = "Initialize global config")]
    pub global: bool,

    #[arg(short, long, help = "Force overwrite existing config")]
    pub force: bool,
}

impl Command for InitArgs {
    fn execute(
        &self,
        _global_config: &GlobalConfig,
        _project_config: &ProjectConfig,
    ) -> Result<()> {
        let loader = ConfigLoader::new();

        if self.global {
            init_global_config(&loader, self.force)?;
        } else {
            init_project_config(&loader, self.force)?;
        }

        Ok(())
    }
}

fn init_global_config(_loader: &ConfigLoader, force: bool) -> Result<()> {
    info!("Initializing global configuration");

    let existing_config = ConfigLoader::find_global_config_path();

    if let Some(ref path) = existing_config {
        if !force {
            info!("Global config already exists at: {}", path.display());
            info!("Use --force to overwrite");
            return Ok(());
        }
        debug!("Overwriting existing global config");
    }

    let config_path = dirs::config_dir()
        .map(|mut p| {
            p.push("codebase-translate");
            p.push("config.toml");
            p
        })
        .ok_or_else(|| {
            TranslateError::Config("Could not determine config directory".to_string())
        })?;

    debug!(config_path = %config_path.display(), "Config path");

    if config_path.exists() && !force {
        info!("Global config already exists at: {}", config_path.display());
        info!("Use --force to overwrite");
        return Ok(());
    }

    let config = GlobalConfig::default();
    debug!("Creating default global configuration");

    if let Some(parent) = config_path.parent() {
        debug!(
            parent_dir = %parent.display(),
            "Creating config directory"
        );
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(&config)?;
    debug!("Writing configuration file");
    std::fs::write(&config_path, content)?;

    info!("Created global config at: {}", config_path.display());
    Ok(())
}

fn init_project_config(loader: &ConfigLoader, force: bool) -> Result<()> {
    info!("Initializing project configuration");

    let config_path = std::env::current_dir()?.join(".translator.toml");
    debug!(config_path = %config_path.display(), "Project config path");

    if config_path.exists() && !force {
        info!(
            "Project config already exists at: {}",
            config_path.display()
        );
        info!("Use --force to overwrite");
        return Ok(());
    }

    let config = ProjectConfig::default();
    debug!("Creating default project configuration");
    loader.save_project(&config, &config_path)?;

    info!("Created project config at: {}", config_path.display());
    Ok(())
}
