//! Language-only filter for forced language extraction
//!
//! This filter checks only for language characteristics and ignores all other
//! filtering rules. Used when `force_extract_by_language` is enabled.

use super::strategy::QuickDetector;
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Language-only filter that checks only for language characteristics
///
/// When enabled, this filter bypasses all other filtering rules and only
/// checks if the text contains characters from the specified languages.
pub struct LanguageOnlyFilter {
    /// Languages to extract
    languages: Vec<String>,
    /// Quick language detector
    detector: QuickDetector,
}

impl LanguageOnlyFilter {
    /// Create a new language-only filter
    ///
    /// # Arguments
    /// * `languages` - List of language codes to extract (e.g., ["ZH", "JA", "KO"])
    pub fn new(languages: Vec<String>) -> Self {
        Self {
            languages,
            detector: QuickDetector::new(),
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
                // For unknown languages, allow through
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

        let matches = self.contains_target_language(text);
        if !matches {
            debug!(
                languages = ?self.languages,
                reason = "no_target_language",
                "Text filtered by language-only filter"
            );
        }
        matches
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

        // Should extract even with TODO, Copyright, etc.
        assert!(filter.should_translate("TODO: 修复这个bug"));
        assert!(filter.should_translate("Copyright © 2024 - 版权所有"));
        assert!(filter.should_translate("Error: 参数错误"));
    }

    #[test]
    fn test_mixed_content_with_urls() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Should extract even with URLs
        assert!(filter.should_translate("https://example.com/你好"));
        assert!(filter.should_translate("Visit https://example.com/文档"));
    }

    #[test]
    fn test_mixed_content_with_placeholders() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Should extract even with placeholders
        assert!(filter.should_translate("Hello %s, 你好"));
        assert!(filter.should_translate("Value: {name}, 值"));
    }

    #[test]
    fn test_mixed_content_with_code_patterns() {
        let filter = LanguageOnlyFilter::new(vec!["ZH".to_string()]);

        // Should extract even with code patterns
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
}
