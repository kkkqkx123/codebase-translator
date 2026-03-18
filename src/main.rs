use std::path::PathBuf;

use clap::{Parser as ClapParser, Subcommand};
use tracing::{error, info};

use codebase_translate::{
    cache::{file::FileCache, Cache},
    config::{global::GlobalConfig, loader::ConfigLoader, project::ProjectConfig},
    core::error::Result,
    core::models::{CacheEntry, File, FileEntry, TranslationStats, TranslatedUnit},
    encoding::{Detector, Encoder},
    logger,
    parser::coordinator::ParserCoordinator,
    parser::tree_sitter::ParserConfig,
    scanner::r#trait::{ScanOptions, Scanner},
    scanner::FSScanner,
    translator::factory::TranslatorConfig,
    translator::service::TranslationService,
    translator::ProviderType,
    writer::file::{FileWriter, WriterConfig},
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

            // Execute translation workflow
            execute_translation_workflow(&path, &global_config, &project_config)?;
        }

        Some(Commands::Init { global, force }) => {
            if global {
                init_global_config(&loader, force)?;
            } else {
                init_project_config(&loader, force)?;
            }
        }

        Some(Commands::Cache { clear, detailed }) => {
            execute_cache_command(&project_config, clear, detailed)?;
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
            execute_translation_workflow(".", &global_config, &project_config)?;
        }
    }

    Ok(())
}

/// Execute the complete translation workflow
fn execute_translation_workflow(
    path: &str,
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    // Step 1: Scan directory for files
    info!("Step 1: Scanning directory...");
    let files = scan_files(path, project_config)?;
    info!("Found {} files to process", files.len());

    if files.is_empty() {
        info!("No files found to translate. Exiting.");
        return Ok(());
    }

    // Step 2: Initialize components
    let cache = create_cache(&project_config.cache, path)?;
    let translator = create_translator(global_config, project_config)?;
    let parser = create_parser(project_config)?;
    let writer = create_writer(project_config)?;
    let detector = Detector::default();
    let encoder = Encoder::default();

    // Step 3: Process files
    info!("Step 2: Processing files...");
    let mut stats = TranslationStats::default();

    for (idx, file_entry) in files.iter().enumerate() {
        info!("[{}/{}] Processing: {}", idx + 1, files.len(), file_entry.path.display());

        match process_file(
            file_entry,
            &cache,
            &translator,
            &parser,
            &writer,
            &detector,
            &encoder,
            project_config,
        ) {
            Ok(file_stats) => {
                stats.merge(&file_stats);
            }
            Err(e) => {
                error!("Failed to process file {}: {}", file_entry.path.display(), e);
                stats.errors += 1;
            }
        }
    }

    // Step 4: Print summary
    let elapsed = start_time.elapsed();
    info!("========================================");
    info!("Translation completed in {:.2}s", elapsed.as_secs_f64());
    info!("Total files: {}", stats.total_files);
    info!("Total units: {}", stats.total_units);
    info!("Translated: {}", stats.translated_units);
    info!("From cache: {}", stats.cached_units);
    info!("Skipped: {}", stats.skipped_units);
    info!("Errors: {}", stats.errors);
    info!("========================================");

    Ok(())
}

/// Scan directory for files to translate
fn scan_files(path: &str, project_config: &ProjectConfig) -> Result<Vec<FileEntry>> {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: path.to_string(),
        include_patterns: project_config.include.patterns.clone(),
        exclude_patterns: project_config.exclude.patterns.clone(),
        follow_symlinks: false,
        respect_gitignore: project_config.exclude.respect_gitignore,
        gitignore_patterns: project_config.exclude.gitignore_patterns.clone(),
        gitignore_path: None,
    };

    scanner.scan(opts)
}

/// Create cache instance
fn create_cache(
    cache_config: &codebase_translate::config::project::CacheConfig,
    project_path: &str,
) -> Result<Box<dyn Cache>> {
    let cache_dir = PathBuf::from(project_path).join(&cache_config.cache_dir);

    // Create a core CacheConfig from project CacheConfig
    let core_cache_config = codebase_translate::core::models::CacheConfig {
        enabled: cache_config.cache_type != "none",
        mode: codebase_translate::core::models::CacheMode::Local,
        directory: cache_dir.to_string_lossy().to_string(),
        format: cache_config.cache_type.clone(),
    };

    let cache: Box<dyn Cache> = match cache_config.cache_type.as_str() {
        "file" | "json" => Box::new(FileCache::new(core_cache_config, project_path)?),
        "binary" => {
            // For binary cache, we would use BinaryCache, but let's use FileCache for simplicity
            Box::new(FileCache::new(core_cache_config, project_path)?)
        }
        _ => Box::new(FileCache::new(core_cache_config, project_path)?),
    };

    Ok(cache)
}

/// Create translator instance
fn create_translator(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    let translator_config = TranslatorConfig {
        provider: project_config.translate.provider,
        deeplx: Some(codebase_translate::translator::common::DeepLXConfig {
            api_url: global_config.deeplx.api_url.clone(),
            api_key: global_config.deeplx.api_key.clone(),
            proxy_url: global_config.deeplx.proxy_url.clone(),
            max_retries: global_config.deeplx.max_retries as usize,
        }),
        llm: None,
        tencent: None,
    };

    TranslationService::new(translator_config)
}

/// Create parser coordinator
fn create_parser(project_config: &ProjectConfig) -> Result<ParserCoordinator> {
    let parser_config = ParserConfig {
        extract_comments: project_config.extraction.comments,
        extract_docstrings: project_config.extraction.doc_strings,
        extract_strings: project_config.extraction.format_strings,
        min_content_length: 2,
        max_content_length: 10000,
        trim_content: true,
    };

    ParserCoordinator::from_project_config(parser_config, project_config)
}

/// Create file writer
fn create_writer(project_config: &ProjectConfig) -> Result<FileWriter> {
    let writer_config = WriterConfig {
        preview_only: project_config.writer.dry_run,
        backup: project_config.writer.backup,
        backup_dir: project_config.writer.backup_dir.as_ref().map(PathBuf::from),
        strict_encoding: false,
    };

    writer_config.validate()?;
    Ok(FileWriter::new(writer_config))
}

/// Process a single file
fn process_file(
    file_entry: &FileEntry,
    cache: &Box<dyn Cache>,
    translator: &TranslationService,
    parser: &ParserCoordinator,
    writer: &FileWriter,
    detector: &Detector,
    encoder: &Encoder,
    project_config: &ProjectConfig,
) -> Result<TranslationStats> {
    let mut stats = TranslationStats::default();
    stats.total_files = 1;

    // Read file content
    let content = std::fs::read(&file_entry.path)?;

    // Detect encoding
    let encoding_result = detector.detect_bytes(&content)?;
    let encoding = encoding_result.encoding;

    // Convert to UTF-8 if needed
    let utf8_content = if encoding != "UTF-8" {
        encoder.to_utf8(&content, &encoding)?.into_bytes()
    } else {
        content.clone()
    };

    // Calculate file hash for cache
    let file_hash = calculate_hash(&utf8_content);

    // Check cache
    let cached_entry = cache.get(&file_hash)?;
    let modified_time = file_entry
        .modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if let Some(entry) = cached_entry {
        if entry.is_valid(modified_time) {
            info!("  Cache hit, using cached translations");
            stats.cached_units = entry.translation_units.len();
            return Ok(stats);
        }
        // Cache expired, need to re-translate
    }

    // Parse file to extract translatable units
    let file = File::new(file_entry.path.clone(), utf8_content.clone(), "UTF-8");
    let mut units = parser.parse_file(&file)?;
    stats.total_units = units.len();

    if units.is_empty() {
        info!("  No translatable content found");
        return Ok(stats);
    }

    // Filter units that need translation
    let units_to_translate: Vec<_> = units.iter().filter(|u| u.should_translate).collect();
    let num_to_translate = units_to_translate.len();

    if num_to_translate == 0 {
        info!("  All units filtered, nothing to translate");
        stats.skipped_units = units.len();
        return Ok(stats);
    }

    // Prepare texts for translation
    let texts: Vec<String> = units_to_translate.iter().map(|u| u.content.clone()).collect();

    info!("  Translating {} units...", texts.len());

    // Translate
    let translated_texts =
        translator.translate_batch(&texts, &project_config.translate.target_lang)?;

    // Update units with translations
    let mut translate_idx = 0;
    for unit in units.iter_mut() {
        if unit.should_translate {
            if let Some(translated) = translated_texts.get(translate_idx) {
                unit.set_translated(translated.clone());
                translate_idx += 1;
            }
        }
    }

    stats.translated_units = num_to_translate;

    // Write translations back to file
    if !project_config.writer.dry_run {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { writer.write(&file, &units).await })?;
    } else {
        info!("  Dry run mode - not writing changes");
        for unit in &units {
            if let Some(translated) = &unit.translated {
                info!("  [{}] '{}' -> '{}'", unit.node_type, unit.content, translated);
            }
        }
    }

    // Update cache
    let mut cache_entry = CacheEntry::new(
        &file_hash,
        file_entry.path.to_string_lossy(),
        modified_time,
        &project_config.cache.cache_type,
        "", // project fingerprint
    );

    for unit in &units {
        if let Some(translated) = &unit.translated {
            cache_entry.add_translated_unit(TranslatedUnit::new(
                &unit.id,
                &unit.content,
                translated.clone(),
                "AUTO",
                &project_config.translate.target_lang,
            ));
        }
    }

    cache.set(&cache_entry)?;

    Ok(stats)
}

/// Calculate simple hash for content
fn calculate_hash(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Execute cache command
fn execute_cache_command(
    project_config: &ProjectConfig,
    clear: bool,
    detailed: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let cache = create_cache(&project_config.cache, current_dir.to_string_lossy().as_ref())?;

    if clear {
        info!("Clearing cache...");
        cache.clear()?;
        info!("Cache cleared successfully");
    } else {
        let stats = cache.stats()?;
        info!("Cache statistics:");
        info!("  Total entries: {}", stats.entry_count);
        info!("  Total size: {} bytes", stats.total_size);

        if detailed {
            let entries = cache.list_entries()?;
            info!("  Detailed entries:");
            for entry in entries {
                info!("    - {}: {}", entry.file_hash, entry.file_path);
            }
        }
    }

    Ok(())
}

fn init_global_config(_loader: &ConfigLoader, force: bool) -> Result<()> {
    // Check if any global config already exists in search paths
    let existing_config = ConfigLoader::find_global_config_path();
    
    if let Some(ref path) = existing_config {
        if !force {
            info!("Global config already exists at: {}", path.display());
            info!("Use --force to overwrite");
            return Ok(());
        }
    }

    // Always use user config directory for init
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

    // Check if config exists in user config directory specifically
    if config_path.exists() && !force {
        info!("Global config already exists at: {}", config_path.display());
        info!("Use --force to overwrite");
        return Ok(());
    }

    // Create default config and save directly to the user config directory
    let config = GlobalConfig::default();
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Serialize and save config directly
    let content = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;

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
