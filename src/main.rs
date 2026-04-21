use clap::{Parser as ClapParser, Subcommand};
use std::path::Path;
use tracing::info;

use codebase_translate::{
    commands::{cache, clean, detect, init, status, translate, validate, verify, Command},
    config::loader::ConfigLoader,
    core::error::Result,
    logger, set_quiet_mode,
    workflow::TranslationWorkflow,
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
    #[arg(short, long)]
    log_level: Option<String>,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,

    /// Suppress all non-error output to terminal
    #[arg(short, long, global = true)]
    quiet: bool,

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

    /// Verify extraction rules
    Verify(verify::args::VerifyArgs),

    /// Clean cache and backup files
    Clean(clean::CleanArgs),

    /// Detect language content in files
    Detect(detect::DetectArgs),

    /// Show translation service status
    Status(status::StatusArgs),
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

    let (mut global_config, project_config) = loader.load()?;

    // Apply quiet mode: suppress non-error output
    if cli.quiet {
        global_config.logging.level = "error".to_string();
        set_quiet_mode(true);
    }

    // Only override log level if explicitly provided by user via command line
    if let Some(log_level) = &cli.log_level {
        global_config.logging.level = log_level.clone();
    }

    match cli.command {
        Some(Commands::Translate(args)) => {
            let project_path = args.get_project_path();
            logger::init(&global_config.logging, project_path.map(Path::new))?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Init(args)) => {
            logger::init(&global_config.logging, None)?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Cache(args)) => {
            logger::init(&global_config.logging, None)?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Validate(args)) => {
            logger::init(&global_config.logging, None)?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Verify(args)) => {
            let project_path = args.get_project_path();
            logger::init(&global_config.logging, project_path.map(Path::new))?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Clean(args)) => {
            logger::init(&global_config.logging, None)?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Detect(args)) => {
            let project_path = args.get_project_path();
            logger::init(&global_config.logging, project_path.map(Path::new))?;
            args.execute(&global_config, &project_config)?
        }
        Some(Commands::Status(args)) => {
            logger::init(&global_config.logging, None)?;
            args.execute(&global_config, &project_config)?
        }
        None => {
            logger::init(&global_config.logging, None)?;
            info!("No command specified, translating current directory");
            info!(
                target_lang = %project_config.translate.target_lang,
                "Default translation target"
            );
            let workflow =
                TranslationWorkflow::from_configs_with_path(global_config, project_config, ".");
            workflow.execute()?;
        }
    }

    Ok(())
}
