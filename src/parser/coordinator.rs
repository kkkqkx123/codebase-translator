//! Parser coordinator module
//!
//! Provides high-level coordination for parsing operations using character-based
//! scanning for text extraction.

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::project::ExtractionConfig;
use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, PatternType, Position, TranslationUnit};
use crate::parser::core::Parser;
use crate::parser::filtering::traits::Filter;
use crate::parser::regex::custom_pattern_matcher::CustomPatternMatcher;
use crate::parser::regex::state_machine::StateMachineMatcher;
use crate::parser::regex_parsers::FallbackParser;
use crate::parser::scanner::{ScannerConfig, ScannerLanguageConfig, TextRegionType, TextScanner};
use crate::parser::{ContentFilter, ParserConfig};

/// Indicates which type of parser will handle a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    /// The character-based scanner
    Scanner,
    /// The regex-based fallback parser
    Regex,
}

/// Parser coordinator that manages parsing and text extraction.
///
/// The coordinator uses character-based scanning for text extraction,
/// with regex-based fallback parsers for unsupported file types.
///
/// After parsing, it applies additional extraction patterns:
/// - Custom regex patterns (simple single-step matching)
/// - State machine patterns (complex multi-step matching)
pub struct ParserCoordinator {
    /// Fallback parser for unsupported file types
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
    /// Scanner configuration
    scanner_config: ScannerConfig,
}

impl ParserCoordinator {
    /// Creates a new parser coordinator with default configuration.
    pub fn with_defaults(config: ParserConfig) -> Result<Self> {
        use crate::parser::filtering::default_filter;

        let extraction_config = ExtractionConfig::default();
        let filter = Arc::new(default_filter()?);

        Self::new(config, extraction_config, filter)
    }

    /// Creates a new parser coordinator from project configuration.
    pub fn from_project_config(
        config: ParserConfig,
        project_config: &crate::config::project::ProjectConfig,
    ) -> Result<Self> {
        use crate::parser::filtering::from_project_config;

        let extraction_config = project_config.extraction.clone();
        let filter = Arc::new(from_project_config(
            &project_config.filter,
            &project_config.translate,
        )?);

        Self::with_extraction_config(
            config,
            extraction_config,
            filter,
            &project_config.translate.target_lang,
        )
    }

    /// Creates a new parser coordinator from project and translator configuration.
    pub fn from_project_and_translator_config(
        config: ParserConfig,
        project_config: &crate::config::project::ProjectConfig,
        translator_max_length: Option<usize>,
    ) -> Result<Self> {
        use crate::parser::filtering::from_project_config_with_translator;

        let extraction_config = project_config.extraction.clone();
        let filter = Arc::new(from_project_config_with_translator(
            &project_config.filter,
            &project_config.translate,
            translator_max_length,
        )?);

        Self::with_extraction_config(
            config,
            extraction_config,
            filter,
            &project_config.translate.target_lang,
        )
    }

    /// Creates a new parser coordinator with unified configuration.
    pub fn with_unified_config(config: ParserConfig) -> Result<Self> {
        use crate::parser::filtering::default_filter;

        let extraction_config = ExtractionConfig {
            comments: config.extract_comments,
            doc_strings: config.extract_docstrings,
            string_literals: config.extract_strings,
            ..Default::default()
        };

        let filter = Arc::new(default_filter()?);

        Self::with_extraction_config(config, extraction_config, filter, "en")
    }

    /// Creates a new parser coordinator with custom extraction config and filter.
    pub fn new(
        config: ParserConfig,
        extraction_config: ExtractionConfig,
        filter: Arc<ContentFilter>,
    ) -> Result<Self> {
        Self::with_extraction_config(config, extraction_config, filter, "en")
    }

    /// Creates a new parser coordinator with extraction config.
    pub fn with_extraction_config(
        config: ParserConfig,
        extraction_config: ExtractionConfig,
        filter: Arc<ContentFilter>,
        target_lang: &str,
    ) -> Result<Self> {
        let fallback_parser = FallbackParser::new(config.clone());

        let custom_patterns = if extraction_config.custom_patterns.is_empty() {
            Vec::new()
        } else {
            extraction_config.custom_patterns.clone()
        };

        let custom_pattern_matchers: Vec<_> = custom_patterns
            .iter()
            .filter_map(|pattern| CustomPatternMatcher::from_config(pattern).ok())
            .collect();

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

        let state_machine_patterns = if extraction_config.state_machine_patterns.is_empty() {
            Vec::new()
        } else {
            extraction_config.state_machine_patterns.clone()
        };

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

        let target_languages = vec![target_lang.to_string()];

        let scanner_config = ScannerConfig::new(target_languages)
            .with_comments(extraction_config.comments)
            .with_doc_strings(extraction_config.doc_strings)
            .with_strings(extraction_config.string_literals)
            .with_min_length(config.min_content_length)
            .with_max_length(config.max_content_length);

        Ok(Self {
            fallback_parser,
            custom_pattern_matchers,
            extension_to_custom_patterns,
            state_machine_matchers,
            extension_to_matchers,
            filter: filter.clone(),
            scanner_config,
        })
    }

    /// Parses a file using the appropriate parser and applies additional patterns.
    pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
        let content = file
            .content_string()
            .map_err(|e| TranslateError::Parse(format!("Failed to decode file content: {}", e)))?;

        let (mut units, content) = self.parse_with_scanner(file, &content)?;

        let file_ext = file.extension().unwrap_or("").to_lowercase();

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

        units = self.deduplicate_units(units, custom_units, sm_units);

        units.sort_by(|a, b| a.start_pos.offset.cmp(&b.start_pos.offset));

        Ok(units)
    }

    /// Deduplicate translation units to avoid overlapping or duplicate entries.
    fn deduplicate_units(
        &self,
        base: Vec<TranslationUnit>,
        custom: Vec<TranslationUnit>,
        sm: Vec<TranslationUnit>,
    ) -> Vec<TranslationUnit> {
        use std::collections::HashSet;

        let mut result = base;
        let mut seen: HashSet<(usize, usize, String)> = HashSet::new();

        for unit in &result {
            let key = (
                unit.start_pos.offset,
                unit.end_pos.offset,
                unit.content.clone(),
            );
            seen.insert(key);
        }

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

    /// Parse file with scanner.
    fn parse_with_scanner(
        &self,
        file: &File,
        content: &str,
    ) -> Result<(Vec<TranslationUnit>, String)> {
        let filename = file.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let ext = file.extension().unwrap_or("");

        if let Some(scanner) = TextScanner::from_extension(ext, self.scanner_config.clone()) {
            let regions = scanner.scan(content);
            let units = self.regions_to_units(&regions, content, &file.path.display().to_string());
            return Ok((units, content.to_string()));
        }

        if self.fallback_parser.supports(filename) {
            let units = self.fallback_parser.parse(file)?;
            return Ok((units, content.to_string()));
        }

        Err(TranslateError::Parse(format!(
            "No parser found for file: {}",
            file.path.display()
        )))
    }

    /// Convert text regions to translation units.
    fn regions_to_units(
        &self,
        regions: &[crate::parser::scanner::TextRegion],
        content: &str,
        file_path: &str,
    ) -> Vec<TranslationUnit> {
        let mut units = Vec::new();

        for (idx, region) in regions.iter().enumerate() {
            let text = match region.extract_content(content) {
                Some(t) => t,
                None => continue,
            };

            if !self.filter.should_translate(text) {
                continue;
            }

            let node_type = self.region_type_to_node_type(region.region_type);

            let start_pos = Position::new(
                self.byte_offset_to_line(content, region.content_start),
                self.byte_offset_to_column(content, region.content_start),
                region.content_start,
            );

            let end_pos = Position::new(
                self.byte_offset_to_line(content, region.content_end),
                self.byte_offset_to_column(content, region.content_end),
                region.content_end,
            );

            let id = format!("{}_{}_{}", file_path, region.region_type, idx);

            let mut unit = TranslationUnit::new(id, node_type, text, start_pos, end_pos);

            if !region.placeholders.is_empty() {
                let placeholder_text = region
                    .placeholders
                    .iter()
                    .map(|p| p.original.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                unit.raw_match = Some(format!("placeholders: {}", placeholder_text));
            }

            units.push(unit);
        }

        units
    }

    /// Convert region type to node type.
    fn region_type_to_node_type(
        &self,
        region_type: TextRegionType,
    ) -> crate::core::models::NodeType {
        use crate::core::models::NodeType;
        match region_type {
            TextRegionType::LineComment => NodeType::Comment,
            TextRegionType::BlockComment => NodeType::Comment,
            TextRegionType::DocComment => NodeType::DocString,
            TextRegionType::SingleQuotedString => NodeType::StringLiteral,
            TextRegionType::DoubleQuotedString => NodeType::StringLiteral,
            TextRegionType::TemplateString => NodeType::StringLiteral,
            TextRegionType::RawString => NodeType::StringLiteral,
            TextRegionType::MultiLineString => NodeType::StringLiteral,
        }
    }

    /// Convert byte offset to line number (1-based).
    fn byte_offset_to_line(&self, content: &str, offset: usize) -> usize {
        content[..offset.min(content.len())]
            .chars()
            .filter(|&c| c == '\n')
            .count()
            + 1
    }

    /// Convert byte offset to column number (1-based).
    fn byte_offset_to_column(&self, content: &str, offset: usize) -> usize {
        let content_before = &content[..offset.min(content.len())];
        if let Some(last_newline) = content_before.rfind('\n') {
            offset - last_newline
        } else {
            offset + 1
        }
    }

    /// Parses multiple files in parallel using Rayon.
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
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if ScannerLanguageConfig::from_extension(ext).is_some() {
            return Some(ParserType::Scanner);
        }

        if self.fallback_parser.supports(filename) {
            return Some(ParserType::Regex);
        }

        None
    }

    /// Returns all supported file extensions.
    pub fn supported_extensions(&self) -> Vec<String> {
        let mut extensions: Vec<String> = ScannerLanguageConfig::all_extensions()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        extensions.extend(
            self.fallback_parser
                .supported_extensions()
                .iter()
                .map(|s: &&str| s.to_string()),
        );

        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Returns the scanner configuration.
    pub fn scanner_config(&self) -> &ScannerConfig {
        &self.scanner_config
    }
}

impl Default for ParserCoordinator {
    fn default() -> Self {
        Self::with_defaults(ParserConfig::default())
            .expect("Failed to create default parser coordinator")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::core::models::File;
    use crate::parser::ParserConfig;
    use crate::parser::ParserCoordinator;

    fn create_test_file(content: &str, path: &str) -> File {
        File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
    }

    #[test]
    fn test_parser_coordinator_creation() {
        let config = ParserConfig::default();
        let coordinator =
            ParserCoordinator::with_defaults(config).expect("Failed to create coordinator");

        assert!(!coordinator.supported_extensions().is_empty());
    }

    #[test]
    fn test_parse_rust_file() {
        let config = ParserConfig::default();
        let coordinator =
            ParserCoordinator::with_defaults(config).expect("Failed to create coordinator");

        let content = r#"
/// 这是一个文档注释
fn main() {
    // 这是一个普通注释
    let x = 5;
}
"#;

        let file = create_test_file(content, "test.rs");
        let units = coordinator
            .parse_file(&file)
            .expect("Parsing should succeed");

        assert!(!units.is_empty());
    }

    #[test]
    fn test_can_parse() {
        let coordinator = ParserCoordinator::default();

        assert!(coordinator.can_parse("test.rs"));
        assert!(coordinator.can_parse("readme.md"));
        assert!(!coordinator.can_parse("test.unknown_extension"));
    }

    #[test]
    fn test_find_parser() {
        let coordinator = ParserCoordinator::default();

        let parser_type = coordinator.find_parser("test.rs");
        assert!(parser_type.is_some());

        let parser_type = coordinator.find_parser("test.md");
        assert!(parser_type.is_some());
    }

    #[test]
    fn test_supported_extensions() {
        let coordinator = ParserCoordinator::default();
        let extensions = coordinator.supported_extensions();

        assert!(!extensions.is_empty());
    }

    #[test]
    fn test_parse_unsupported_file() {
        let coordinator = ParserCoordinator::default();

        let file = create_test_file("content", "test.unknown_extension");
        let result = coordinator.parse_file(&file);

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("No parser found"));
    }
}
