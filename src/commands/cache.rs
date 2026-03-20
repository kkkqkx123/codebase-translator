use clap::Parser;
use tracing::{debug, info};

use crate::{
    cache::Cache,
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    factory::create_cache,
};

use super::Command;

#[derive(Parser, Debug)]
pub struct CacheArgs {
    #[arg(long, help = "Clear cache")]
    pub clear: bool,

    #[arg(long, help = "Show detailed cache entries")]
    pub detailed: bool,
}

impl Command for CacheArgs {
    fn execute(&self, _global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()> {
        execute_cache_command(project_config, self.clear, self.detailed)
    }
}

fn execute_cache_command(
    project_config: &ProjectConfig,
    clear: bool,
    detailed: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    debug!(
        cache_dir = %project_config.cache.directory,
        cache_enabled = project_config.cache.enabled,
        "Creating cache instance"
    );
    let cache = create_cache(
        &project_config.cache,
        current_dir.to_string_lossy().as_ref(),
    )?;

    if clear {
        info!("Starting cache clear operation");
        cache.clear()?;
        info!("Cache cleared successfully");
    } else {
        debug!("Retrieving cache statistics");
        let stats = cache.stats()?;
        info!("Cache statistics:");
        info!("  Total entries: {}", stats.entry_count);
        info!("  Total size: {} bytes", stats.total_size);

        if detailed {
            debug!("Retrieving detailed cache entries");
            let entries = cache.list_entries()?;
            info!("  Detailed entries:");
            for entry in entries {
                info!("    - {}: {}", entry.file_hash, entry.file_path);
            }
        }
    }

    Ok(())
}
