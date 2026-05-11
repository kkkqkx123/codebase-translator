//! Composite filter module
//!
//! This module provides a composite filter that orchestrates all filter checks.
//! Checks are applied in order of complexity for optimal performance.

use crate::config::project::FilterConfig;
use crate::parser::filtering::checks::language::LanguageOnlyFilter;
use crate::parser::filtering::checks::{
    ContentFilter, LanguageFilter, LengthFilter, PatternFilter,
};
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Composite filter that orchestrates all filter checks
///
/// Checks are applied in order:
/// 1. LengthFilter - O(1) checks (empty, max length)
/// 2. LanguageFilter - O(k) language detection
/// 3. PatternFilter - O(n) regex matching
/// 4. ContentFilter - O(len) content analysis
///
/// When `extract_languages` is non-empty, language-only filtering mode is enabled.
pub struct CompositeFilter {
    length: LengthFilter,
    language: LanguageFilter,
    pattern: PatternFilter,
    content: ContentFilter,
    language_only: Option<LanguageOnlyFilter>,
}

impl CompositeFilter {
    /// Create a new composite filter with language settings
    ///
    /// # Arguments
    /// * `config` - Filter configuration
    /// * `source_langs` - Source languages to translate from (e.g., ["zh", "AUTO"])
    /// * `target_lang` - Target language for translation (e.g., "EN")
    ///
    /// If `extract_languages` is non-empty, enables language-only filtering mode:
    /// - Only extracts text containing characters from specified languages
    /// - Skips keyword and pattern filtering
    /// - Still applies format protection (URL/placeholder filtering)
    pub fn with_language_settings(
        config: &FilterConfig,
        source_langs: Vec<String>,
        target_lang: String,
    ) -> crate::core::error::Result<Self> {
        let language_only = if !config.extract_languages.is_empty() {
            Some(LanguageOnlyFilter::new(config.extract_languages.clone()))
        } else {
            None
        };

        Ok(Self {
            length: LengthFilter::new(config),
            language: LanguageFilter::new(source_langs, target_lang),
            pattern: PatternFilter::new(config)?,
            content: ContentFilter::new(),
            language_only,
        })
    }

    /// Create a new composite filter (backward compatible)
    ///
    /// Uses default language settings (empty source_langs, "EN" target_lang).
    pub fn new(config: FilterConfig) -> crate::core::error::Result<Self> {
        Self::with_language_settings(&config, Vec::new(), "EN".to_string())
    }

    /// Create a default composite filter
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> crate::core::error::Result<Self> {
        Self::new(FilterConfig::default())
    }

    /// Get filter configuration (returns default for inspection)
    pub fn config(&self) -> FilterConfig {
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

    /// Check if text should be translated with options
    ///
    /// # Arguments
    /// * `text` - The text to check
    /// * `check_code_patterns` - Whether to check for code patterns (false for comments)
    pub fn should_translate_with_options(&self, text: &str, check_code_patterns: bool) -> bool {
        if let Some(ref lang_filter) = self.language_only {
            return lang_filter.should_translate(text);
        }

        if !self.length.should_translate(text) {
            return false;
        }

        if !self.language.should_translate(text) {
            return false;
        }

        if !self
            .pattern
            .should_translate_with_options(text, check_code_patterns)
        {
            return false;
        }

        if !self.content.should_translate(text) {
            return false;
        }

        debug!(text = %text, "Text passed all filter checks");
        true
    }
}

impl Filter for CompositeFilter {
    fn should_translate(&self, text: &str) -> bool {
        if let Some(ref lang_filter) = self.language_only {
            return lang_filter.should_translate(text);
        }

        if !self.length.should_translate(text) {
            return false;
        }

        if !self.language.should_translate(text) {
            return false;
        }

        if !self.pattern.should_translate(text) {
            return false;
        }

        if !self.content.should_translate(text) {
            return false;
        }

        debug!(text = %text, "Text passed all filter checks");
        true
    }

    fn name(&self) -> &str {
        if self.language_only.is_some() {
            "CompositeFilter (LanguageOnly mode)"
        } else {
            "CompositeFilter"
        }
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
    let max_length = if config.max_length == 0 {
        100000
    } else {
        config.max_length
    };

    let filter_config = FilterConfig {
        min_length: config.min_length,
        exclude_keywords: config.exclude_keywords.clone(),
        exclude_patterns: config.exclude_patterns.clone(),
        include_patterns: config.include_patterns.clone(),
        max_length,
        allow_placeholders: config.allow_placeholders,
        detect_code_patterns: config.detect_code_patterns,
        extract_languages: config.extract_languages.clone(),
        placeholder_patterns: config.placeholder_patterns.clone(),
        code_patterns: config.code_patterns.clone(),
    };

    CompositeFilter::with_language_settings(
        &filter_config,
        translate_config.source_langs.clone(),
        translate_config.target_lang.clone(),
    )
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
        min_length: project_config.min_length,
        exclude_keywords: project_config.exclude_keywords.clone(),
        exclude_patterns: project_config.exclude_patterns.clone(),
        include_patterns: project_config.include_patterns.clone(),
        max_length,
        allow_placeholders: project_config.allow_placeholders,
        detect_code_patterns: project_config.detect_code_patterns,
        extract_languages: project_config.extract_languages.clone(),
        placeholder_patterns: project_config.placeholder_patterns.clone(),
        code_patterns: project_config.code_patterns.clone(),
    };

    CompositeFilter::with_language_settings(
        &filter_config,
        translate_config.source_langs.clone(),
        translate_config.target_lang.clone(),
    )
}

/// Create a test filter that allows English content to be extracted
pub fn test_filter() -> crate::core::error::Result<CompositeFilter> {
    let config = FilterConfig::default();
    CompositeFilter::with_language_settings(&config, vec!["EN".to_string()], "ZH".to_string())
}

/// Create a verify filter that allows all content to be extracted
/// Used by the verify command to show all potential matches without filtering
pub fn verify_filter() -> crate::core::error::Result<CompositeFilter> {
    // Use empty source_langs and target_lang to bypass language filtering
    // This allows all content to pass through for verification purposes
    let config = FilterConfig {
        min_length: 1, // Only filter out empty strings
        exclude_keywords: vec![],
        exclude_patterns: vec![],
        include_patterns: vec![],
        max_length: 100000,
        allow_placeholders: true,
        detect_code_patterns: false,
        extract_languages: vec![], // Empty means no language-only filtering
        placeholder_patterns: vec![],
        code_patterns: vec![],
    };
    // Use empty source_langs to bypass all language filtering
    // This ensures verify command shows all potential extraction matches
    CompositeFilter::with_language_settings(&config, vec![], "XX".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_filter() {
        let config = FilterConfig::default();
        let filter = CompositeFilter::with_language_settings(
            &config,
            vec!["EN".to_string()],
            "ZH".to_string(),
        )
        .unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("TODO: fix this"));
    }

    #[test]
    fn test_check_ordering() {
        let filter = CompositeFilter::default().unwrap();
        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("TODO"));
    }

    #[test]
    fn test_language_only_mode() {
        let config = FilterConfig {
            extract_languages: vec!["ZH".to_string()],
            ..Default::default()
        };
        let filter = CompositeFilter::new(config).unwrap();

        assert!(filter.should_translate("TODO: 修复中文bug"));
        assert!(!filter.should_translate("https://example.com/你好"));
        assert!(!filter.should_translate("Hello %s 你好"));
        assert!(filter.should_translate("你好世界"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_standard_filter_mode() {
        let config = FilterConfig {
            exclude_keywords: vec!["TODO".to_string()],
            ..Default::default()
        };
        let filter = CompositeFilter::with_language_settings(
            &config,
            vec!["EN".to_string()],
            "ZH".to_string(),
        )
        .unwrap();

        assert!(!filter.should_translate("TODO: fix this"));
        assert!(filter.should_translate("Hello World"));
    }

    #[test]
    fn test_name_with_language_only() {
        let config = FilterConfig {
            extract_languages: vec!["ZH".to_string()],
            ..Default::default()
        };
        let filter = CompositeFilter::new(config).unwrap();

        assert_eq!(filter.name(), "CompositeFilter (LanguageOnly mode)");
    }

    #[test]
    fn test_multiple_languages() {
        let config = FilterConfig {
            extract_languages: vec!["ZH".to_string(), "JA".to_string()],
            ..Default::default()
        };
        let filter = CompositeFilter::new(config).unwrap();

        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("Hello 你好"));
        assert!(filter.should_translate("Hello こんにちは"));
        assert!(!filter.should_translate("Hello World"));
    }
}
