use clap::Parser;
use tracing::info;

use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    translator::{create_llm_multi_provider_translator, ProviderType, TranslatorImpl},
};

use super::Command;

#[derive(Parser, Debug)]
pub struct StatusArgs {
    #[arg(long, help = "Show detailed provider information")]
    pub detailed: bool,
}

impl Command for StatusArgs {
    fn execute(&self, global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()> {
        show_status(global_config, project_config, self.detailed)
    }
}

fn show_status(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
    detailed: bool,
) -> Result<()> {
    info!("=== Translation Service Status ===");
    info!("");

    info!("Project Configuration:");
    info!(
        "  Target Language: {}",
        project_config.translate.target_lang
    );
    info!(
        "  Source Languages: {:?}",
        project_config.translate.source_langs
    );
    info!("  Provider: {:?}", project_config.translate.provider);
    info!("");

    match project_config.translate.provider {
        ProviderType::DeepLX => {
            show_deeplx_status(global_config, detailed)?;
        }
        ProviderType::LLM => {
            show_llm_status(global_config, detailed)?;
        }
        ProviderType::Tencent => {
            show_tencent_status(global_config, detailed)?;
        }
    }

    info!("");
    info!("Cache Status:");
    info!("  Enabled: {}", project_config.cache.enabled);
    info!("  Directory: {}", project_config.cache.directory);
    info!("");

    Ok(())
}

fn show_deeplx_status(global_config: &GlobalConfig, detailed: bool) -> Result<()> {
    info!("=== DeepLX Provider ===");
    info!("  API URL: {}", global_config.deeplx.api_url);
    info!("  Max Input Chars: Unlimited");

    if detailed {
        info!("  Status: Active");
    }
    info!("");

    Ok(())
}

fn show_llm_status(global_config: &GlobalConfig, detailed: bool) -> Result<()> {
    info!("=== LLM Provider ===");
    info!("  Total Providers: {}", global_config.llm.providers.len());
    info!("");

    if global_config.llm.providers.is_empty() {
        info!("  No LLM providers configured");
        return Ok(());
    }

    match create_llm_multi_provider_translator(global_config) {
        Ok(TranslatorImpl::LLM(multi_translator)) => {
            let stats = multi_translator.get_router_stats();
            let capacity_threshold = stats["capacity_threshold"].as_u64().unwrap_or(0);
            let max_capacity = stats["max_capacity"].as_u64().unwrap_or(0);
            let strategy = stats["strategy"].as_str().unwrap_or("Unknown");

            info!("  Capacity Threshold: {} chars", capacity_threshold);
            info!("  Max Capacity: {} chars", max_capacity);
            info!("  Selection Strategy: {}", strategy);
            info!("");

            if detailed {
                info!("  Provider Details:");
                if let Some(providers) = stats["providers"].as_array() {
                    for (idx, provider) in providers.iter().enumerate() {
                        info!("    Provider {}:", idx + 1);
                        info!("      ID: {}", provider["id"].as_str().unwrap_or("Unknown"));
                        info!(
                            "      Max Chars: {}",
                            provider["max_chars"].as_u64().unwrap_or(0)
                        );
                        info!(
                            "      Rate Limit: {}",
                            provider["rate_limit"].as_u64().unwrap_or(0)
                        );
                    }
                }
                info!("");
            }

            info!("  Routing Logic:");
            info!(
                "    - Short texts (< {} chars): All providers",
                capacity_threshold
            );
            info!(
                "    - Long texts (>= {} chars): Capable providers only",
                capacity_threshold
            );
        }
        Ok(TranslatorImpl::DeepLX(_)) | Ok(TranslatorImpl::Tencent(_)) => {
            info!("  Unexpected translator type");
        }
        Err(e) => {
            info!("  Failed to initialize LLM translator: {}", e);
        }
    }

    info!("");

    Ok(())
}

fn show_tencent_status(global_config: &GlobalConfig, detailed: bool) -> Result<()> {
    info!("=== Tencent Cloud Provider ===");
    info!("  Region: {}", global_config.tencent.region);
    info!("  Max Input Chars: Unlimited");

    if detailed {
        info!(
            "  Secret ID: {}",
            global_config
                .tencent
                .secret_id
                .as_deref()
                .unwrap_or("Not configured")
        );
        info!("  Status: Active");
    }
    info!("");

    Ok(())
}
