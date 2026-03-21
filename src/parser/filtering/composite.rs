//! Composite filter module
//!
//! This module provides a composite filter that orchestrates all filter layers.
//! Layers are applied in order of complexity for optimal performance.

use crate::parser::filtering::config::FilterConfig;
use crate::parser::filtering::layers::{BasicFilter, ContentFilter, LanguageFilter, PatternFilter};
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Composite filter that orchestrates all filter layers
///
/// Layers are applied in order:
/// 1. BasicFilter - O(1) checks (empty, length)
/// 2. LanguageFilter - O(k) language detection
/// 3. PatternFilter - O(n) regex matching
/// 4. ContentFilter - O(len) content analysis
pub struct CompositeFilter {
    basic: BasicFilter,
    language: LanguageFilter,
    pattern: PatternFilter,
    content: ContentFilter,
}

impl CompositeFilter {
    /// Create a new composite filter
    pub fn new(config: FilterConfig) -> crate::core::error::Result<Self> {
        Ok(Self {
            basic: BasicFilter::new(&config),
            language: LanguageFilter::new(&config),
            pattern: PatternFilter::new(&config)?,
            content: ContentFilter::new(),
        })
    }

    /// Create a default composite filter
    pub fn default() -> crate::core::error::Result<Self> {
        Self::new(FilterConfig::default())
    }

    /// Get filter configuration (returns default for inspection)
    pub fn config(&self) -> FilterConfig {
        // Return default config for API compatibility
        FilterConfig::default()
    }

    /// Check if text contains placeholders
    pub fn contains_placeholder(&self, text: &str) -> bool {
        self.pattern.contains_placeholder(text)
    }

    /// Check if text contains code patterns
    pub fn contains_code_pattern(&self, text: &str) -> bool {
        self.pattern.contains_code_pattern(text)
    }
}

impl Filter for CompositeFilter {
    fn should_translate(&self, text: &str) -> bool {
        // Layer 1: Basic checks (fastest)
        if !self.basic.should_translate(text) {
            return false;
        }

        // Layer 2: Language detection
        if !self.language.should_translate(text) {
            return false;
        }

        // Layer 3: Pattern matching
        if !self.pattern.should_translate(text) {
            return false;
        }

        // Layer 4: Content analysis (slowest)
        if !self.content.should_translate(text) {
            return false;
        }

        debug!(text = %text, "Text passed all filter layers");
        true
    }

    fn name(&self) -> &str {
        "CompositeFilter"
    }
}

impl std::fmt::Debug for CompositeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeFilter")
            .field("layers", &vec!["Basic", "Language", "Pattern", "Content"])
            .finish()
    }
}

/// Create a default filter
pub fn default_filter() -> crate::core::error::Result<CompositeFilter> {
    CompositeFilter::default()
}

/// Create a filter from project config
pub fn from_project_config(
    config: &crate::config::project::FilterConfig,
    translate_config: &crate::config::project::TranslateConfig,
) -> crate::core::error::Result<CompositeFilter> {
    let filter_config = FilterConfig {
        source_langs: translate_config.source_langs.clone(),
        exclude_keywords: config.exclude_keywords.clone(),
        exclude_patterns: config.exclude_patterns.clone(),
        include_patterns: config.include_patterns.clone(),
        max_length: if config.max_length == 0 {
            100000
        } else {
            config.max_length
        },
        allow_placeholders: config.allow_placeholders,
        detect_code_patterns: config.detect_code_patterns,
    };
    CompositeFilter::new(filter_config)
}

/// Create a filter from project config with translator max length
pub fn from_project_config_with_translator(
    project_config: &crate::config::project::FilterConfig,
    translate_config: &crate::config::project::TranslateConfig,
    translator_max_length: Option<usize>,
) -> crate::core::error::Result<CompositeFilter> {
    let max_length = match (project_config.max_length, translator_max_length) {
        (0, None) => 100000,
        (0, Some(translator_max)) => translator_max,
        (project_max, None) => project_max,
        (project_max, Some(translator_max)) => project_max.min(translator_max),
    };

    let filter_config = FilterConfig {
        source_langs: translate_config.source_langs.clone(),
        exclude_keywords: project_config.exclude_keywords.clone(),
        exclude_patterns: project_config.exclude_patterns.clone(),
        include_patterns: project_config.include_patterns.clone(),
        max_length,
        allow_placeholders: project_config.allow_placeholders,
        detect_code_patterns: project_config.detect_code_patterns,
    };
    CompositeFilter::new(filter_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_filter() {
        let filter = CompositeFilter::default().unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("TODO: fix this"));
    }

    #[test]
    fn test_layer_ordering() {
        // Empty text should be rejected by basic layer
        let filter = CompositeFilter::default().unwrap();
        assert!(!filter.should_translate(""));

        // Keyword should be rejected by pattern layer
        assert!(!filter.should_translate("TODO"));
    }
}
