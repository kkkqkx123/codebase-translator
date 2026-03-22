//! Composite filter module
//!
//! This module provides a composite filter that orchestrates all filter checks.
//! Checks are applied in order of complexity for optimal performance.

use crate::parser::filtering::config::FilterConfig;
use crate::parser::filtering::checks::{LengthFilter, ContentFilter, LanguageFilter, PatternFilter};
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Composite filter that orchestrates all filter checks
///
/// Checks are applied in order:
/// 1. LengthFilter - O(1) checks (empty, max length)
/// 2. LanguageFilter - O(k) language detection
/// 3. PatternFilter - O(n) regex matching
/// 4. ContentFilter - O(len) content analysis
pub struct CompositeFilter {
    length: LengthFilter,
    language: LanguageFilter,
    pattern: PatternFilter,
    content: ContentFilter,
}

impl CompositeFilter {
    /// Create a new composite filter
    pub fn new(config: FilterConfig) -> crate::core::error::Result<Self> {
        Ok(Self {
            length: LengthFilter::new(&config),
            language: LanguageFilter::new(&config),
            pattern: PatternFilter::new(&config)?,
            content: ContentFilter::new(),
        })
    }

    /// Create a default composite filter
    #[allow(clippy::should_implement_trait)]
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
        // Check 1: Length checks (fastest)
        if !self.length.should_translate(text) {
            return false;
        }

        // Check 2: Language detection
        if !self.language.should_translate(text) {
            return false;
        }

        // Check 3: Pattern matching
        if !self.pattern.should_translate(text) {
            return false;
        }

        // Check 4: Content analysis (slowest)
        if !self.content.should_translate(text) {
            return false;
        }

        debug!(text = %text, "Text passed all filter checks");
        true
    }

    fn name(&self) -> &str {
        "CompositeFilter"
    }
}

impl std::fmt::Debug for CompositeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeFilter")
            .field("checks", &vec!["Length", "Language", "Pattern", "Content"])
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
        target_lang: translate_config.target_lang.clone(),
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
        target_lang: translate_config.target_lang.clone(),
        exclude_keywords: project_config.exclude_keywords.clone(),
        exclude_patterns: project_config.exclude_patterns.clone(),
        include_patterns: project_config.include_patterns.clone(),
        max_length,
        allow_placeholders: project_config.allow_placeholders,
        detect_code_patterns: project_config.detect_code_patterns,
    };
    CompositeFilter::new(filter_config)
}

/// Create a test filter that allows English content to be extracted
/// This is needed for tests that expect to extract English text
pub fn test_filter() -> crate::core::error::Result<CompositeFilter> {
    let config = FilterConfig {
        source_langs: vec!["EN".to_string()],
        ..FilterConfig::default()
    };
    CompositeFilter::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_filter() {
        // Create a filter with EN as source language to allow English content
        let config = FilterConfig {
            source_langs: vec!["EN".to_string()],
            ..FilterConfig::default()
        };
        let filter = CompositeFilter::new(config).unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("TODO: fix this"));
    }

    #[test]
    fn test_check_ordering() {
        // Empty text should be rejected by length check
        let filter = CompositeFilter::default().unwrap();
        assert!(!filter.should_translate(""));

        // Keyword should be rejected by pattern check
        assert!(!filter.should_translate("TODO"));
    }
}
