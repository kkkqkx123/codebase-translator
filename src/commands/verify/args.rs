use clap::Parser;
use tracing::{debug, info};

use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::{Result, TranslateError},
    core::models::File,
    encoding::Detector,
    encoding::Encoder,
    parser::ParserCoordinator,
    scanner::r#trait::{ScanOptions, Scanner},
    scanner::FSScanner,
};

use super::{
    FilterOptions, MatchCollector, MatchFilter, OutputFormat, OutputFormatter, StatisticsGenerator,
};
use crate::commands::Command;

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    #[arg(default_value = ".", value_name = "PATH")]
    pub path: String,

    #[arg(short, long, value_name = "PATTERN")]
    pub pattern: Option<String>,

    #[arg(short, long, value_name = "EXT")]
    pub extension: Option<String>,

    #[arg(short = 'k', long, value_name = "CATEGORY")]
    pub category: Option<String>,

    #[arg(short, long, value_name = "TEXT")]
    pub search: Option<String>,

    #[arg(short, long, default_value = "table", value_name = "FORMAT")]
    pub format: OutputFormat,

    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    #[arg(short, long, default_value = "true")]
    pub detailed: bool,

    #[arg(short = 'S', long, default_value = "true")]
    pub show_stats: bool,

    /// Suppress log output
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,
}

impl Command for VerifyArgs {
    fn execute(&self, _global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()> {
        info!(
            path = %self.path,
            "Starting verification of extraction rules"
        );

        let root_path = std::path::Path::new(&self.path);

        if !root_path.exists() {
            return Err(TranslateError::Io(format!(
                "Path does not exist: {}",
                self.path
            )));
        }

        debug!("Configuring scan options");
        let scan_options = ScanOptions {
            root_path: self.path.clone(),
            include_patterns: project_config.include.patterns.clone(),
            exclude_patterns: project_config.exclude.patterns.clone(),
            follow_symlinks: false,
            respect_gitignore: project_config.exclude.respect_gitignore,
            gitignore_patterns: project_config.exclude.gitignore_patterns.clone(),
            gitignore_path: None,
        };

        debug!("Creating file scanner");
        let scanner = FSScanner::new();
        debug!("Starting directory scan");
        let file_entries = scanner.scan(scan_options)?;
        let total_files = file_entries.len();

        info!(files_found = total_files, "Scanned files");

        debug!("Creating parser for verification");
        let parser = ParserCoordinator::for_verification(project_config)?;

        let mut all_matches = Vec::new();

        debug!("Processing files for extraction");
        for (index, entry) in file_entries.iter().enumerate() {
            debug!(
                file = %entry.path.display(),
                progress = format!("{}/{}", index + 1, total_files),
                "Processing file"
            );
            let file = Self::load_file(&entry.path)?;
            let content = file.content_string().map_err(|e| {
                TranslateError::Parse(format!(
                    "Failed to decode file {}: {}",
                    entry.path.display(),
                    e
                ))
            })?;

            let units = parser.parse_file(&file)?;
            let matches = MatchCollector::collect_from_units(entry.path.clone(), units, &content);
            debug!(
                file = %entry.path.display(),
                matches = matches.len(),
                "Extracted matches from file"
            );
            all_matches.extend(matches);
        }

        info!(total_matches = all_matches.len(), "Extracted matches");

        debug!("Applying filters");
        let filter_options = FilterOptions::new()
            .with_pattern_name(self.pattern.clone().unwrap_or_default())
            .with_extension(self.extension.clone().unwrap_or_default())
            .with_category(self.category.clone().unwrap_or_default())
            .with_search_text(self.search.clone().unwrap_or_default());

        let filtered_matches = MatchFilter::filter(all_matches, &filter_options);

        info!(
            filtered_matches = filtered_matches.len(),
            "Filtered matches"
        );

        debug!("Generating statistics");
        let summary = StatisticsGenerator::generate(&filtered_matches, total_files);

        debug!("Formatting output");
        let output = OutputFormatter::format(
            &filtered_matches,
            &summary,
            self.format,
            self.detailed,
            self.show_stats,
        )?;

        if let Some(output_path) = &self.output {
            debug!(
                output_path = %output_path,
                "Writing results to file"
            );
            std::fs::write(output_path, output).map_err(|e| {
                TranslateError::Io(format!("Failed to write output to {}: {}", output_path, e))
            })?;
            info!(
                output_path = %output_path,
                "Results written to file"
            );
        } else {
            debug!("Outputting results to console");
            println!("{}", output);
        }

        info!("Verification completed successfully");
        Ok(())
    }

    fn get_project_path(&self) -> Option<&str> {
        // If path is a file, return its parent directory for logger initialization
        // This ensures the log directory is created in the correct location
        let path = std::path::Path::new(&self.path);
        if path.is_file() {
            path.parent()
                .map(|p| p.to_str())
                .unwrap_or(Some("."))
        } else {
            Some(&self.path)
        }
    }
}

impl VerifyArgs {
    fn load_file(path: &std::path::Path) -> Result<File> {
        let content_bytes = std::fs::read(path).map_err(|e| {
            TranslateError::Io(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        let detector = Detector::default();
        let encoding_result = detector.detect_bytes(&content_bytes).map_err(|e| {
            TranslateError::Parse(format!(
                "Failed to detect encoding for file {}: {}",
                path.display(),
                e
            ))
        })?;
        let encoding = encoding_result.encoding;

        let encoder = Encoder::default();
        let content = encoder.to_utf8(&content_bytes, &encoding).map_err(|e| {
            TranslateError::Parse(format!("Failed to decode file {}: {}", path.display(), e))
        })?;

        Ok(File::new(
            path.to_path_buf(),
            content.into_bytes(),
            encoding,
        ))
    }
}
