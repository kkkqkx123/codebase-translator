//! Parser coordinator implementation

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, PatternType, TranslationUnit};
use crate::parser::abstraction::parser::Parser as ParserTrait;
use crate::parser::abstraction::strategy::ExtractionConfig;
use crate::parser::filtering::traits::Filter;
use crate::parser::{ConfigBasedStrategy, ContentFilter, ExtractionStrategyImpl};
use crate::parser::engine::{ParserConfig, TreeSitterParser, TreeSitterParserFactory};
use crate::parser::regex::custom_pattern_matcher::CustomPatternMatcher;
use crate::parser::regex::state_machine::StateMachineMatcher;
use crate::parser::regex_parsers::FallbackParser;

use super::ParserType;

/// Parser coordinator that manages multiple parsers and routes files appropriately.
///
/// The coordinator maintains a collection of parsers and selects the appropriate
/// one based on file extension. It tries tree-sitter parsers first (for accuracy),
/// then falls back to regex-based parsers for unsupported file types.
///
/// After parsing, it applies additional extraction patterns:
/// - Custom regex patterns (simple single-step matching)
/// - State machine patterns (complex multi-step matching)
pub struct ParserCoordinator {
    tree_sitter_parsers: Vec<TreeSitterParser>,
    fallback_parser: FallbackParser,
    /// Custom pattern matchers for simple regex-based extraction
    custom_pattern_matchers: Vec<CustomPatternMatcher>,
    /// Map from file extension to custom pattern matcher indices
    extension_to_custom_patterns: HashMap<String, Vec<usize>>,
    /// State machine matchers for custom pattern extraction
    state_machine_matchers: Vec<StateMachineMatcher>,
    /// Map from file extension to state machine matcher indices
    extension_to_matchers: HashMap<String, Vec<usize>>,
    /// Content filter for filtering extracted text
    filter: Arc<ContentFilter>,
}

impl ParserCoordinator {
    /// Creates a new parser coordinator with default configuration.
    pub fn with_defaults(config: ParserConfig) -> Result<Self> {
        use crate::parser::core::strategies::strategy_impl::ExtractionStrategyImpl;
        use crate::parser::filtering::default_filter;

        let strategy = Arc::new(ExtractionStrategyImpl::default_config());
        let filter = Arc::new(default_filter()?);

        Self::new(config, strategy, filter)
    }

    /// Creates a new parser coordinator from project configuration.
    ///
    /// This method creates a coordinator with filter configuration derived from
    /// the project's filter settings, ensuring that max_length is properly configured
    /// based on the project configuration rather than hardcoded values.
    pub fn from_project_config(
        config: ParserConfig,
        project_config: &crate::config::project::ProjectConfig,
    ) -> Result<Self> {
        use crate::parser::core::strategies::strategy_impl::ExtractionStrategyImpl;
        use crate::parser::filtering::from_project_config;

        let strategy = Arc::new(ExtractionStrategyImpl::default_config());
        let filter = Arc::new(from_project_config(
            &project_config.filter,
            &project_config.translate,
        )?);

        Self::new(config, strategy, filter)
    }

    /// Creates a new parser coordinator from project and translator configuration.
    ///
    /// This method creates a coordinator with filter configuration derived from
    /// both project's filter settings and translator's max length limit.
    /// The max_length will be the minimum of project config and translator limit.
    pub fn from_project_and_translator_config(
        config: ParserConfig,
        project_config: &crate::config::project::ProjectConfig,
        translator_max_length: Option<usize>,
    ) -> Result<Self> {
        use crate::parser::core::strategies::strategy_impl::ExtractionStrategyImpl;
        use crate::parser::filtering::from_project_config_with_translator;

        let strategy = Arc::new(ExtractionStrategyImpl::default_config());
        let filter = Arc::new(from_project_config_with_translator(
            &project_config.filter,
            &project_config.translate,
            translator_max_length,
        )?);

        Self::new(config, strategy, filter)
    }

    /// Creates a new parser coordinator with unified configuration.
    ///
    /// This method ensures consistency between ParserConfig and ExtractionConfig
    /// by deriving the strategy configuration from the parser configuration.
    pub fn with_unified_config(config: ParserConfig) -> Result<Self> {
        use crate::parser::filtering::default_filter;

        // Derive ExtractionConfig from ParserConfig to ensure consistency
        let extraction_config = ExtractionConfig {
            comments: config.extract_comments,
            docstrings: config.extract_docstrings,
            string_literals: config.extract_strings,
            ..Default::default()
        };

        let strategy = Arc::new(ExtractionStrategyImpl::ConfigBased(
            ConfigBasedStrategy::new(extraction_config),
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
            TreeSitterParserFactory::create_all_parsers(config.clone(), strategy, filter.clone())
        {
            match parser_result {
                Ok(parser) => tree_sitter_parsers.push(parser),
                Err(e) => {
                    tracing::warn!("Failed to create parser: {}", e);
                }
            }
        }

        let fallback_parser = FallbackParser::new(config.clone());

        // Load custom patterns from extraction config
        let custom_patterns = extraction_config
            .as_ref()
            .and_then(|cfg| {
                if cfg.custom_patterns.is_empty() {
                    None
                } else {
                    Some(cfg.custom_patterns.clone())
                }
            })
            .unwrap_or_default();

        // Create custom pattern matchers
        let custom_pattern_matchers: Vec<_> = custom_patterns
            .iter()
            .filter_map(|pattern| CustomPatternMatcher::from_config(pattern).ok())
            .collect();

        // Build extension to custom patterns mapping
        let mut extension_to_custom_patterns = HashMap::new();
        for (idx, matcher) in custom_pattern_matchers.iter().enumerate() {
            let extensions = if matcher.file_extensions().is_empty() {
                vec!["*".to_string()]
            } else {
                matcher.file_extensions().to_vec()
            };

            for ext in extensions {
                extension_to_custom_patterns
                    .entry(ext.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

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
            custom_pattern_matchers,
            extension_to_custom_patterns,
            state_machine_matchers,
            extension_to_matchers,
            filter: filter.clone(),
        })
    }

    /// Creates a coordinator with pre-built parsers.
    pub fn with_parsers(
        tree_sitter_parsers: Vec<TreeSitterParser>,
        fallback_parser: FallbackParser,
    ) -> Self {
        use crate::parser::{ContentFilter, FilterConfig};

        Self {
            tree_sitter_parsers,
            fallback_parser,
            custom_pattern_matchers: Vec::new(),
            extension_to_custom_patterns: HashMap::new(),
            state_machine_matchers: Vec::new(),
            extension_to_matchers: HashMap::new(),
            filter: Arc::new(
                ContentFilter::new(FilterConfig::default())
                    .expect("Failed to create default filter"),
            ),
        }
    }

    /// Parses a file using the appropriate parser and applies additional patterns.
    ///
    /// The extraction process:
    /// 1. Parse file with appropriate parser (Tree-sitter or Regex)
    /// 2. Apply custom regex patterns (simple single-step matching)
    /// 3. Apply state machine patterns (complex multi-step matching)
    /// 4. Deduplicate overlapping units
    /// 5. Sort all units by position
    pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        // 1. Use appropriate parser to parse file and get decoded content
        let (mut units, content) = self.parse_with_parser(file)?;

        let file_ext = file.extension().unwrap_or("").to_lowercase();

        // 2. Apply custom regex patterns (simple single-step matching)
        let mut custom_units = Vec::new();
        if let Some(matcher_indices) = self
            .extension_to_custom_patterns
            .get(&file_ext)
            .or_else(|| self.extension_to_custom_patterns.get("*"))
        {
            for &idx in matcher_indices {
                let matcher = &self.custom_pattern_matchers[idx];

                tracing::debug!(
                    pattern_name = %matcher.name,
                    file_extension = %file_ext,
                    "Applying custom regex pattern"
                );

                match matcher.find_matches(&content) {
                    Ok(matches) => {
                        for m in matches {
                            let text = &m.extracted_text;

                            // Apply content filter (includes length check)
                            if !self.filter.should_translate(text) {
                                continue;
                            }

                            let id = format!(
                                "{}_cp_{}_{}",
                                file.path.display(),
                                matcher.name,
                                custom_units.len()
                            );

                            let mut unit = TranslationUnit::new_with_pattern(
                                id,
                                crate::core::models::NodeType::StringLiteral,
                                text.clone(),
                                m.start_pos,
                                m.end_pos,
                                PatternType::CustomRegex,
                                matcher.name.clone(),
                            );
                            unit.raw_match = Some(m.raw_content);
                            custom_units.push(unit);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            pattern_name = %matcher.name,
                            file_extension = %file_ext,
                            error = %e,
                            "Failed to apply custom pattern, skipping"
                        );
                    }
                }
            }
        }

        // 3. Apply state machine patterns (complex multi-step matching)
        let mut sm_units = Vec::new();
        if let Some(matcher_indices) = self
            .extension_to_matchers
            .get(&file_ext)
            .or_else(|| self.extension_to_matchers.get("*"))
        {
            for &idx in matcher_indices {
                let matcher = &self.state_machine_matchers[idx];

                tracing::debug!(
                    matcher_name = %matcher.name,
                    file_extension = %file_ext,
                    "Applying state machine pattern"
                );

                match matcher.find_matches(&content) {
                    Ok(matches) => {
                        for m in matches {
                            let text = &m.extracted_text;

                            // Apply content filter (includes length check)
                            if !self.filter.should_translate(text) {
                                continue;
                            }

                            let id = format!(
                                "{}_sm_{}_{}",
                                file.path.display(),
                                matcher.name,
                                sm_units.len()
                            );

                            let mut unit = TranslationUnit::new_with_pattern(
                                id,
                                crate::core::models::NodeType::StringLiteral,
                                text.clone(),
                                m.start_pos,
                                m.end_pos,
                                PatternType::StateMachine,
                                matcher.name.clone(),
                            );
                            unit.raw_match = Some(m.raw_content);
                            sm_units.push(unit);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            matcher_name = %matcher.name,
                            file_extension = %file_ext,
                            error = %e,
                            "Failed to apply state machine pattern, skipping"
                        );
                    }
                }
            }
        }

        // 4. Deduplicate overlapping units
        units = self.deduplicate_units(units, custom_units, sm_units);

        // 5. Sort by position
        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }

    /// Deduplicate translation units to avoid overlapping or duplicate entries.
    ///
    /// This method combines units from three sources:
    /// - Base units from the primary parser
    /// - Custom regex pattern matches
    /// - State machine pattern matches
    ///
    /// Units are considered duplicates if they have the same position range and content.
    fn deduplicate_units(
        &self,
        base: Vec<TranslationUnit>,
        custom: Vec<TranslationUnit>,
        sm: Vec<TranslationUnit>,
    ) -> Vec<TranslationUnit> {
        use std::collections::HashSet;

        let mut result = base;
        let mut seen: HashSet<(usize, usize, String)> = HashSet::new();

        // Mark base units as seen
        for unit in &result {
            let key = (
                unit.start_pos.offset,
                unit.end_pos.offset,
                unit.content.clone(),
            );
            seen.insert(key);
        }

        // Add custom pattern units if not duplicates
        for unit in custom {
            let key = (
                unit.start_pos.offset,
                unit.end_pos.offset,
                unit.content.clone(),
            );
            if seen.insert(key) {
                result.push(unit);
            }
        }

        // Add state machine units if not duplicates
        for unit in sm {
            let key = (
                unit.start_pos.offset,
                unit.end_pos.offset,
                unit.content.clone(),
            );
            if seen.insert(key) {
                result.push(unit);
            }
        }

        result
    }

    /// Parse file with appropriate parser only (without additional patterns).
    /// Returns both translation units and decoded content to avoid redundant decoding.
    fn parse_with_parser(&self, file: &File) -> Result<(Vec<TranslationUnit>, String)> {
        let filename = file.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Decode content once and reuse it
        let content = file
            .content_string()
            .map_err(|e| TranslateError::Parse(format!("Failed to decode file content: {}", e)))?;

        for parser in &self.tree_sitter_parsers {
            if parser.supports(filename) {
                let units = parser.parse(file)?;
                return Ok((units, content));
            }
        }

        if self.fallback_parser.supports(filename) {
            let units = self.fallback_parser.parse(file)?;
            return Ok((units, content));
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
