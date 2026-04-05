//! Language-only filter for language-specific extraction
//!
//! This filter checks for language characteristics while preserving
//! format protection features (placeholder detection, URL filtering).
//!
//! Enabled when `extract_languages` is non-empty in the filter configuration.

use super::strategy::QuickDetector;
use crate::parser::filtering::traits::Filter;
use regex::Regex;
use tracing::debug;

/// Configuration for format protection in language-only mode
#[derive(Debug, Clone)]
pub struct FormatProtectionConfig {
    /// Whether to filter out URLs and emails
    pub filter_urls: bool,
    /// Whether to filter out text containing placeholders
    pub filter_placeholders: bool,
    /// Whether to allow placeholders (if false, text with placeholders is filtered)
    pub allow_placeholders: bool,
}

impl Default for FormatProtectionConfig {
    fn default() -> Self {
        Self {
            filter_urls: true,
            filter_placeholders: true,
            allow_placeholders: false,
        }
    }
}

/// Language-only filter that checks for language characteristics
/// while preserving format protection features.
///
/// When `extract_languages` is non-empty, this filter:
/// 1. Checks if text contains characters from specified languages
/// 2. Applies format protection (URL/placeholder filtering)
pub struct LanguageOnlyFilter {
    /// Languages to extract
    languages: Vec<String>,
    /// Quick language detector
    detector: QuickDetector,
    /// Format protection configuration
    protection: FormatProtectionConfig,
    /// URL pattern regex
    url_pattern: Regex,
    /// Placeholder patterns
    placeholder_patterns: Vec<Regex>,
}

impl LanguageOnlyFilter {
    /// Create a new language-only filter with default protection
    ///
    /// # Arguments
    /// * `languages` - List of language codes to extract (e.g., ["ZH", "JA", "KO"])
    pub fn new(languages: Vec<String>) -> Self {
        Self::with_protection(languages, FormatProtectionConfig::default())
    }

    /// Create a new language-only filter with custom protection config
    ///
    /// # Arguments
    /// * `languages` - List of language codes to extract
    /// * `protection` - Format protection configuration
    pub fn with_protection(languages: Vec<String>, protection: FormatProtectionConfig) -> Self {
        let url_pattern = Regex::new(r"https?://[^\s]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
            .expect("Invalid URL pattern regex");

        let placeholder_patterns = vec![
            Regex::new(r"%[sdvf]").expect("Invalid placeholder regex"),
            Regex::new(r"\$\d{1,2}\b").expect("Invalid placeholder regex"),
            Regex::new(r"\$\{[^}]*\}").expect("Invalid placeholder regex"),
            Regex::new(r"\{[^}]*\}").expect("Invalid placeholder regex"),
        ];

        Self {
            languages,
            detector: QuickDetector::new(),
            protection,
            url_pattern,
            placeholder_patterns,
        }
    }

    /// Check if text contains any of the specified language characters
    fn contains_target_language(&self, text: &str) -> bool {
        for lang in &self.languages {
            let lang_upper = lang.to_uppercase();
            match lang_upper.as_str() {
                "ZH" | "ZH-CN" | "ZH-TW" | "HANS" | "HANT" => {
                    if self.detector.has_chinese(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_chinese",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "JA" => {
                    if self.detector.has_japanese(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_japanese",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "KO" => {
                    if self.detector.has_korean(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_korean",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "EN" | "EN-US" | "EN-GB" => {
                    if self.detector.is_latin(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_latin",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "AR" => {
                    if self.detector.has_arabic(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_arabic",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                "RU" | "UK" | "BG" => {
                    if self.detector.has_cyrillic(text) {
                        debug!(
                            language = %lang,
                            reason = "contains_cyrillic",
                            "Text matched language-only filter"
                        );
                        return true;
                    }
                }
                _ => {
                    debug!(
                        language = %lang,
                        reason = "unknown_language_allowed",
                        "Text matched language-only filter (unknown language)"
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Check if text should be filtered by format protection
    fn should_filter_by_protection(&self, text: &str) -> bool {
        // URL filtering
        if self.protection.filter_urls && self.url_pattern.is_match(text) {
            debug!(
                reason = "contains_url",
                "Text filtered by format protection"
            );
            return true;
        }

        // Placeholder filtering
        if self.protection.filter_placeholders && !self.protection.allow_placeholders {
            for pattern in &self.placeholder_patterns {
                if pattern.is_match(text) {
                    debug!(
                        reason = "contains_placeholder",
                        "Text filtered by format protection"
                    );
                    return true;
                }
            }
        }

        false
    }
}

impl Filter for LanguageOnlyFilter {
    fn should_translate(&self, text: &str) -> bool {
        // Check if text contains any of the specified language characters
        if self.languages.is_empty() {
            debug!(
                reason = "no_languages_specified",
                "Text filtered by language-only filter (no languages specified)"
            );
            return false;
        }

        // First check: language detection
        let matches = self.contains_target_language(text);
        if !matches {
            debug!(
                languages = ?self.languages,
                reason = "no_target_language",
                "Text filtered by language-only filter"
            );
            return false;
        }

        // Second check: format protection (optional)
        if self.should_filter_by_protection(text) {
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "LanguageOnlyFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("Hello 你好"));
        assert!(filter.should_translate("你好Hello"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_japanese_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["JA".to_string()]);

        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("カタカナ"));
        assert!(filter.should_translate("Hello こんにちは"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_korean_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["KO".to_string()]);

        assert!(filter.should_translate("안녕하세요"));
        assert!(filter.should_translate("Hello 안녕하세요"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_multiple_languages() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string(), "JA".to_string()]);

        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("Hello 你好"));
        assert!(filter.should_translate("Hello こんにちは"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_english_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["EN".to_string()]);

        assert!(filter.should_translate("Hello World"));
        assert!(filter.should_translate("你好 Hello"));
        assert!(!filter.should_translate("你好世界"));
    }

    #[test]
    fn test_empty_languages() {
        let filter = LanguageOnlyFilter::new(vec![]);

        assert!(!filter.should_translate("Hello World"));
        assert!(!filter.should_translate("你好世界"));
    }

    #[test]
    fn test_mixed_content_with_keywords() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Should extract even with TODO, Copyright, etc. (keywords are not filtered in force mode)
        assert!(filter.should_translate("TODO: 修复这个bug"));
        assert!(filter.should_translate("Copyright © 2024 - 版权所有"));
        assert!(filter.should_translate("Error: 参数错误"));
    }

    #[test]
    fn test_url_filtering_in_force_mode() {
        // Default: URLs are filtered
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // URLs should be filtered by default
        assert!(!filter.should_translate("https://example.com/你好"));
        assert!(!filter.should_translate("Visit https://example.com/文档"));

        // Without URL, should pass
        assert!(filter.should_translate("访问文档页面"));
    }

    #[test]
    fn test_url_filtering_disabled() {
        // With URL filtering disabled
        let protection = FormatProtectionConfig {
            filter_urls: false,
            filter_placeholders: true,
            allow_placeholders: false,
        };
        let filter = LanguageOnlyFilter::with_protection(vec!["ZH".to_string()], protection);

        // URLs should pass when filtering is disabled
        assert!(filter.should_translate("https://example.com/你好"));
        assert!(filter.should_translate("Visit https://example.com/文档"));
    }

    #[test]
    fn test_placeholder_filtering_in_force_mode() {
        // Default: placeholders are filtered
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Placeholders should be filtered by default
        assert!(!filter.should_translate("Hello %s, 你好"));
        assert!(!filter.should_translate("Value: {name}, 值"));

        // Without placeholders, should pass
        assert!(filter.should_translate("你好，世界"));
    }

    #[test]
    fn test_placeholder_filtering_disabled() {
        // With placeholder filtering disabled
        let protection = FormatProtectionConfig {
            filter_urls: true,
            filter_placeholders: false,
            allow_placeholders: false,
        };
        let filter = LanguageOnlyFilter::with_protection(vec!["ZH".to_string()], protection);

        // Placeholders should pass when filtering is disabled
        assert!(filter.should_translate("Hello %s, 你好"));
        assert!(filter.should_translate("Value: {name}, 值"));
    }

    #[test]
    fn test_allow_placeholders() {
        // With allow_placeholders enabled
        let protection = FormatProtectionConfig {
            filter_urls: true,
            filter_placeholders: true,
            allow_placeholders: true,
        };
        let filter = LanguageOnlyFilter::with_protection(vec!["ZH".to_string()], protection);

        // Placeholders should pass when allowed
        assert!(filter.should_translate("Hello %s, 你好"));
        assert!(filter.should_translate("Value: {name}, 值"));
    }

    #[test]
    fn test_mixed_content_with_code_patterns() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Code patterns are NOT filtered in force mode (only URLs and placeholders)
        assert!(filter.should_translate("obj.method() 你好"));
        assert!(filter.should_translate("func(arg) 参数错误"));
    }

    #[test]
    fn test_chinese_variants() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Both simplified and traditional should be recognized
        assert!(filter.should_translate("你好世界")); // Simplified
        assert!(filter.should_translate("你好世界")); // Traditional
    }

    #[test]
    fn test_arabic_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["AR".to_string()]);

        assert!(filter.should_translate("مرحبا بالعالم"));
        assert!(filter.should_translate("Hello مرحبا"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_cyrillic_extraction() {
        let filter = LanguageOnlyFilter::new(vec!["RU".to_string()]);

        assert!(filter.should_translate("Привет мир"));
        assert!(filter.should_translate("Hello Привет"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_unknown_language_allowed() {
        let filter = LanguageOnlyFilter::new(vec!["UNKNOWN".to_string()]);

        // Unknown language should allow through
        assert!(filter.should_translate("Hello World"));
        assert!(filter.should_translate("你好世界"));
    }

    #[test]
    fn test_empty_text() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("   "));
    }

    #[test]
    fn test_format_protection_config_default() {
        let config = FormatProtectionConfig::default();

        assert!(config.filter_urls);
        assert!(config.filter_placeholders);
        assert!(!config.allow_placeholders);
    }
}
