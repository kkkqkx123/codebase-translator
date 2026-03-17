//! Parser coordinator implementation

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::filter::ContentFilter;
use crate::parser::regex::state_machine::StateMachineMatcher;

use crate::parser::regex_parsers::FallbackParser;
use crate::parser::strategy::{ExtractionConfig, ExtractionStrategyImpl};
use crate::parser::tree_sitter::{ParserConfig, TreeSitterParser, TreeSitterParserFactory};
use crate::parser::Parser as ParserTrait;

use super::ParserType;

/// Parser coordinator that manages multiple parsers and routes files appropriately.
///
/// The coordinator maintains a collection of parsers and selects the appropriate
/// one based on file extension. It tries tree-sitter parsers first (for accuracy),
/// then falls back to regex-based parsers for unsupported file types.
pub struct ParserCoordinator {
    tree_sitter_parsers: Vec<TreeSitterParser>,
    fallback_parser: FallbackParser,
    /// State machine matchers for custom pattern extraction
    state_machine_matchers: Vec<StateMachineMatcher>,
    /// Map from file extension to state machine matcher indices
    extension_to_matchers: HashMap<String, Vec<usize>>,
}

impl ParserCoordinator {
    /// Creates a new parser coordinator with default configuration.
    pub fn with_defaults(config: ParserConfig) -> Result<Self> {
        use crate::parser::filter::default_filter;
        use crate::parser::strategy::default_strategy;

        let strategy = Arc::new(default_strategy());
        let filter = Arc::new(default_filter()?);

        Self::new(config, strategy, filter)
    }

    /// Creates a new parser coordinator with unified configuration.
    ///
    /// This method ensures consistency between ParserConfig and ExtractionConfig
    /// by deriving the strategy configuration from the parser configuration.
    pub fn with_unified_config(config: ParserConfig) -> Result<Self> {
        use crate::parser::filter::default_filter;

        // Derive ExtractionConfig from ParserConfig to ensure consistency
        let extraction_config = ExtractionConfig {
            comments: config.extract_comments,
            docstrings: config.extract_docstrings,
            string_literals: config.extract_strings,
            ..Default::default()
        };

        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            crate::parser::strategy::ConfigBasedStrategy::new(extraction_config),
        ));
        let filter = Arc::new(default_filter()?);

        Self::new(config, strategy, filter)
    }

    /// Creates a new parser coordinator with custom strategy and filter.
    pub fn new(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Self::with_extraction_config(config, strategy, filter, None)
    }

    /// Creates a new parser coordinator with extraction config for state machine patterns.
    pub fn with_extraction_config(
        config: ParserConfig,
        strategy: Arc<ExtractionStrategyImpl>,
        filter: Arc<ContentFilter>,
        extraction_config: Option<crate::config::project::ExtractionConfig>,
    ) -> Result<Self> {
        let mut tree_sitter_parsers: Vec<TreeSitterParser> = Vec::new();

        for parser_result in
            TreeSitterParserFactory::create_all_parsers(config.clone(), strategy, filter)
        {
            match parser_result {
                Ok(parser) => tree_sitter_parsers.push(parser),
                Err(e) => {
                    tracing::warn!("Failed to create parser: {}", e);
                }
            }
        }

        let fallback_parser = FallbackParser::new(config);

        // Load state machine patterns from extraction config
        let state_machine_patterns = extraction_config
            .and_then(|cfg| {
                if cfg.state_machine_patterns.is_empty() {
                    None
                } else {
                    Some(cfg.state_machine_patterns)
                }
            })
            .unwrap_or_default();

        // Create state machine matchers
        let state_machine_matchers: Vec<_> = state_machine_patterns
            .iter()
            .filter_map(|pattern| {
                StateMachineMatcher::from_config(
                    pattern.name.clone(),
                    pattern.initial_state.clone(),
                    pattern.accepting_states.clone(),
                    &pattern.states,
                    pattern.extraction_rule.clone(),
                )
                .ok()
            })
            .collect();

        // Build extension to matchers mapping
        let mut extension_to_matchers = HashMap::new();
        for (idx, pattern) in state_machine_patterns.iter().enumerate() {
            let extensions = if pattern.file_extensions.is_empty() {
                vec!["*".to_string()]
            } else {
                pattern.file_extensions.clone()
            };

            for ext in extensions {
                extension_to_matchers
                    .entry(ext.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        Ok(Self {
            tree_sitter_parsers,
            fallback_parser,
            state_machine_matchers,
            extension_to_matchers,
        })
    }

    /// Creates a coordinator with pre-built parsers.
    pub fn with_parsers(
        tree_sitter_parsers: Vec<TreeSitterParser>,
        fallback_parser: FallbackParser,
    ) -> Self {
        Self {
            tree_sitter_parsers,
            fallback_parser,
            state_machine_matchers: Vec::new(),
            extension_to_matchers: HashMap::new(),
        }
    }

    /// Parses a file using the appropriate parser and applies state machines.
    pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        // 1. Use appropriate parser to parse file
        let mut units = self.parse_with_parser(file)?;

        // 2. Apply state machine patterns (only for matching file extensions)
        let file_ext = file.extension().unwrap_or("").to_lowercase();

        // Check if there are applicable state machines
        if let Some(matcher_indices) = self
            .extension_to_matchers
            .get(&file_ext)
            .or_else(|| self.extension_to_matchers.get("*"))
        {
            let content = file.content_string().map_err(|e| {
                TranslateError::Parse(format!("Failed to decode file content: {}", e))
            })?;

            for &idx in matcher_indices {
                let matcher = &self.state_machine_matchers[idx];

                tracing::debug!(
                    matcher_name = %matcher.name,
                    file_extension = %file_ext,
                    "Applying state machine pattern"
                );

                let matches = matcher.find_matches(&content)?;

                for m in matches {
                    // Use extracted text
                    let text = &m.extracted_text;

                    // Filter by length (using default values if not in config)
                    let min_length = 2;
                    let max_length = 10000;

                    if text.len() >= min_length && text.len() <= max_length {
                        let id = format!(
                            "{}_sm_{}_{}",
                            file.path.display(),
                            matcher.name,
                            units.len()
                        );

                        let unit = TranslationUnit::new(
                            id,
                            crate::core::models::NodeType::StringLiteral,
                            text.clone(),
                            m.start_pos,
                            m.end_pos,
                        );
                        units.push(unit);
                    }
                }
            }
        }

        // 3. Sort by position
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }

    /// Parse file with appropriate parser only (without state machines).
    fn parse_with_parser(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let filename = file.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for parser in &self.tree_sitter_parsers {
            if parser.supports(filename) {
                return parser.parse(file);
            }
        }

        if self.fallback_parser.supports(filename) {
            return self.fallback_parser.parse(file);
        }

        Err(TranslateError::Parse(format!(
            "No parser found for file: {}",
            file.path.display()
        )))
    }

    /// Parses multiple files in parallel using Rayon.
    ///
    /// This method is CPU-intensive and benefits from parallel processing.
    /// Suitable for projects with many files.
    ///
    /// # Arguments
    /// * `files` - Slice of files to parse
    ///
    /// # Returns
    /// Vector of tuples containing (File, TranslationUnits)
    pub fn parse_files_parallel(
        &self,
        files: &[File],
    ) -> Result<Vec<(File, Vec<TranslationUnit>)>> {
        let results: Result<Vec<_>> = files
            .par_iter()
            .map(|file| {
                let units = self.parse_file(file)?;
                Ok((file.clone(), units))
            })
            .collect();

        results
    }

    /// Checks if this coordinator can parse a given file.
    pub fn can_parse(&self, filename: &str) -> bool {
        self.find_parser(filename).is_some()
    }

    /// Finds the appropriate parser for a file.
    pub fn find_parser(&self, filename: &str) -> Option<ParserType> {
        for (index, parser) in self.tree_sitter_parsers.iter().enumerate() {
            if parser.supports(filename) {
                return Some(ParserType::TreeSitter(index));
            }
        }

        if self.fallback_parser.supports(filename) {
            return Some(ParserType::Regex);
        }

        None
    }

    /// Returns all supported file extensions.
    pub fn supported_extensions(&self) -> Vec<String> {
        let mut extensions = Vec::new();

        for parser in &self.tree_sitter_parsers {
            extensions.extend(parser.supported_extensions().iter().map(|s| s.to_string()));
        }

        extensions.extend(
            self.fallback_parser
                .supported_extensions()
                .iter()
                .map(|s| s.to_string()),
        );

        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Returns the number of tree-sitter parsers.
    pub fn tree_sitter_parser_count(&self) -> usize {
        self.tree_sitter_parsers.len()
    }
}

impl Default for ParserCoordinator {
    fn default() -> Self {
        Self::with_defaults(ParserConfig::default())
            .expect("Failed to create default parser coordinator")
    }
}
