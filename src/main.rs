use clap::{Parser as ClapParser, Subcommand};
use tracing::info;

use codebase_translate::{
    config::{global::GlobalConfig, loader::ConfigLoader, project::ProjectConfig},
    core::error::Result,
    logger,
    translator::ProviderType,
};

/// Codebase Translate - Automatic code comment translator
#[derive(ClapParser)]
#[command(name = "translator")]
#[command(about = "Automatic code comment and documentation translator")]
#[command(version)]
struct Cli {
    /// Config file path
    #[arg(short, long)]
    config: Option<String>,

    /// Global config file path
    #[arg(long)]
    global_config: Option<String>,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,

    /// Subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate files in a directory
    Translate {
        /// Target directory
        #[arg(default_value = ".")]
        path: String,

        /// Target language
        #[arg(short, long)]
        target_lang: Option<String>,

        /// Source languages (comma-separated)
        #[arg(short, long)]
        source_langs: Option<String>,

        /// Translation provider
        #[arg(short, long)]
        provider: Option<String>,

        /// Include patterns (comma-separated globs)
        #[arg(long)]
        include: Option<String>,

        /// Exclude patterns (comma-separated globs)
        #[arg(long)]
        exclude: Option<String>,
    },

    /// Initialize configuration
    Init {
        /// Initialize global config
        #[arg(long)]
        global: bool,

        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },

    /// Show cache statistics
    Cache {
        /// Clear cache
        #[arg(long)]
        clear: bool,

        /// Show detailed cache entries
        #[arg(long)]
        detailed: bool,
    },

    /// Validate configuration
    Validate,
}

/// Main entry point - synchronous
///
/// Note: We use a synchronous main function to avoid async complexity
/// in the application layer. The only async operations are translation
/// API calls, which are isolated in TranslationService.
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut loader = ConfigLoader::new();

    if let Some(config_path) = &cli.config {
        loader = loader.with_project_config(config_path);
    }
    if let Some(global_config_path) = &cli.global_config {
        loader = loader.with_global_config(global_config_path);
    }

    let (mut global_config, mut project_config) = loader.load()?;

    // Override log level from CLI
    global_config.logging.level = cli.log_level.clone();

    // Initialize logging
    logger::init(&global_config.logging)?;

    info!(
        "Starting {} v{}",
        codebase_translate::NAME,
        codebase_translate::VERSION
    );

    // Override dry run from CLI
    if cli.dry_run {
        project_config.writer.dry_run = true;
    }

    // Execute command
    match cli.command {
        Some(Commands::Translate {
            path,
            target_lang,
            source_langs,
            provider,
            include,
            exclude,
        }) => {
            // Override config with CLI arguments
            if let Some(lang) = target_lang {
                project_config.translate.target_lang = lang;
            }
            if let Some(langs) = source_langs {
                project_config.translate.source_langs =
                    langs.split(',').map(|s| s.trim().to_string()).collect();
            }
            if let Some(prov) = provider {
                project_config.translate.provider = prov.parse().map_err(|e| {
                    codebase_translate::core::error::TranslateError::InvalidArgument(e)
                })?;
            }
            if let Some(inc) = include {
                project_config.include.patterns =
                    inc.split(',').map(|s| s.trim().to_string()).collect();
            }
            if let Some(exc) = exclude {
                project_config.exclude.patterns =
                    exc.split(',').map(|s| s.trim().to_string()).collect();
            }

            info!("Translating directory: {}", path);
            info!("Target language: {}", project_config.translate.target_lang);
            info!("Provider: {}", project_config.translate.provider);

            // TODO: Implement translation workflow in Phase 2
            info!("Translation workflow will be implemented in Phase 2");
        }

        Some(Commands::Init { global, force }) => {
            if global {
                init_global_config(&loader, force)?;
            } else {
                init_project_config(&loader, force)?;
            }
        }

        Some(Commands::Cache { clear, detailed }) => {
            if clear {
                info!("Clearing cache...");
                // TODO: Implement cache clearing in Phase 2
            } else {
                info!("Cache statistics:");
                // TODO: Implement cache statistics in Phase 2
                if detailed {
                    info!("Detailed cache information will be shown here");
                }
            }
        }

        Some(Commands::Validate) => {
            info!("Validating configuration...");
            validate_config(&global_config, &project_config)?;
            info!("Configuration is valid!");
        }

        None => {
            // Default: translate current directory
            info!("No command specified, translating current directory");
            info!("Target language: {}", project_config.translate.target_lang);
        }
    }

    Ok(())
}

fn init_global_config(loader: &ConfigLoader, force: bool) -> Result<()> {
    let config_path = dirs::config_dir()
        .map(|mut p| {
            p.push("codebase-translate");
            p.push("config.toml");
            p
        })
        .ok_or_else(|| {
            codebase_translate::core::error::TranslateError::Config(
                "Could not determine config directory".to_string(),
            )
        })?;

    if config_path.exists() && !force {
        info!("Global config already exists at: {}", config_path.display());
        info!("Use --force to overwrite");
        return Ok(());
    }

    let config = GlobalConfig::default();
    loader.save_global(&config)?;

    info!("Created global config at: {}", config_path.display());
    Ok(())
}

fn init_project_config(loader: &ConfigLoader, force: bool) -> Result<()> {
    let config_path = std::env::current_dir()?.join(".translator.toml");

    if config_path.exists() && !force {
        info!(
            "Project config already exists at: {}",
            config_path.display()
        );
        info!("Use --force to overwrite");
        return Ok(());
    }

    let config = ProjectConfig::default();
    loader.save_project(&config, &config_path)?;

    info!("Created project config at: {}", config_path.display());
    Ok(())
}

fn validate_config(global: &GlobalConfig, project: &ProjectConfig) -> Result<()> {
    // Validate target language
    if project.translate.target_lang.is_empty() {
        return Err(codebase_translate::core::error::TranslateError::Config(
            "Target language cannot be empty".to_string(),
        ));
    }

    // Validate provider configuration
    match project.translate.provider {
        ProviderType::DeepLX => {
            // DeepLX doesn't require authentication for local instances
            info!("Using DeepLX provider at: {}", global.deeplx.api_url);
        }
        ProviderType::LLM => {
            if global.llm.providers.is_empty() {
                return Err(codebase_translate::core::error::TranslateError::Config(
                    "No LLM providers configured".to_string(),
                ));
            }
        }
        ProviderType::Tencent => {
            if global.tencent.secret_id.is_none() || global.tencent.secret_key.is_none() {
                return Err(codebase_translate::core::error::TranslateError::Config(
                    "Tencent Cloud requires secret_id and secret_key".to_string(),
                ));
            }
        }
    }

    Ok(())
}
