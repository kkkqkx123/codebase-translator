use clap::{Parser as ClapParser, Subcommand};
use tracing::info;

use codebase_translate::{
    config::loader::ConfigLoader,
    core::error::Result,
    logger,
    NAME, VERSION,
};

mod commands;
use commands::{cache, init, translate, validate, Command};

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
    Translate(translate::TranslateArgs),

    /// Initialize configuration
    Init(init::InitArgs),

    /// Show cache statistics
    Cache(cache::CacheArgs),

    /// Validate configuration
    Validate(validate::ValidateArgs),
}

/// Main entry point - synchronous
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut loader = ConfigLoader::new();

    if let Some(config_path) = &cli.config {
        loader = loader.with_project_config(config_path);
    }
    if let Some(global_config_path) = &cli.global_config {
        loader = loader.with_global_config(global_config_path);
    }

    let (mut global_config, mut project_config) = loader.load()?;

    global_config.logging.level = cli.log_level.clone();

    logger::init(&global_config.logging)?;

    info!(name = NAME, version = VERSION, "Starting application");

    if cli.dry_run {
        project_config.writer.dry_run = true;
    }

    match cli.command {
        Some(Commands::Translate(args)) => args.execute(&global_config, &project_config)?,
        Some(Commands::Init(args)) => args.execute(&global_config, &project_config)?,
        Some(Commands::Cache(args)) => args.execute(&global_config, &project_config)?,
        Some(Commands::Validate(args)) => args.execute(&global_config, &project_config)?,
        None => {
            info!("No command specified, translating current directory");
            info!(
                target_lang = %project_config.translate.target_lang,
                "Default translation target"
            );
            let workflow =
                codebase_translate::workflow::TranslationWorkflow::from_configs_with_path(
                    global_config,
                    project_config,
                    ".",
                );
            workflow.execute()?;
        }
    }

    Ok(())
}
