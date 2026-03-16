//! Parser coordinator implementation

use std::sync::Arc;

use crate::core::error::{Result, TranslateError};
use crate::core::models::{File, TranslationUnit};
use crate::parser::filter::ContentFilter;

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

        Ok(Self {
            tree_sitter_parsers,
            fallback_parser,
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
        }
    }

    /// Parses a file using the appropriate parser.
    pub fn parse_file(&self, file: &File) -> Result<Vec<TranslationUnit>> {
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
