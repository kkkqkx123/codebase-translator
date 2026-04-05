//! Language detection and filtering module
//!
//! This module provides a tiered language detection strategy:
//!
//! 1. **Quick Detection** (O(32)) - First 32 characters only
//!    - Used for filtering decisions
//!    - Fast path for common cases
//!
//! 2. **Sampled Detection** (O(n/sample_rate)) - For long texts
//!    - Samples text at intervals to avoid scanning entire content
//!    - Configurable sample rate
//!
//! 3. **Full Detection** (O(n)) - Complete text analysis
//!    - Used when precise language identification is needed
//!    - Distinguishes between languages in same script (e.g., Japanese vs Chinese)

mod detector;
mod language_only;
mod script;
mod strategy;

pub use detector::{LanguageDetector, LanguageInfo};
pub use language_only::LanguageOnlyFilter;
pub use script::Script;
pub use strategy::{DetectionStrategy, QuickDetector, SampledDetector};

use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Language filter for source language validation
pub struct LanguageFilter {
    source_langs: Vec<String>,
    target_lang: String,
    quick_detector: QuickDetector,
}

impl LanguageFilter {
    /// Create a new language filter
    ///
    /// # Arguments
    /// * `source_langs` - Source languages to translate from (e.g., ["zh", "AUTO"])
    /// * `target_lang` - Target language for translation (e.g., "EN")
    pub fn new(source_langs: Vec<String>, target_lang: String) -> Self {
        Self {
            source_langs,
            target_lang,
            quick_detector: QuickDetector::new(),
        }
    }

    /// Check if text is already in target language (to avoid re-translating)
    fn is_target_language(&self, text: &str) -> bool {
        let target = self.target_lang.to_uppercase();

        match target.as_str() {
            "EN" | "EN-US" | "EN-GB" => {
                if !self.quick_detector.is_latin(text) {
                    return false;
                }
                if self.quick_detector.has_cjk(text) {
                    return false;
                }
                true
            }
            "ZH" | "ZH-CN" | "ZH-TW" => self.quick_detector.has_cjk(text),
            "JA" => self.quick_detector.has_japanese(text),
            "KO" => self.quick_detector.has_korean(text),
            _ => false,
        }
    }

    /// Check if text contains source language characters
    fn contains_source_language(&self, text: &str) -> bool {
        if self.source_langs.is_empty() {
            return true;
        }

        if self
            .source_langs
            .iter()
            .any(|lang| lang.to_uppercase() == "AUTO")
        {
            if self.is_target_language(text) {
                return false;
            }
            return true;
        }

        for lang in &self.source_langs {
            let lang_upper = lang.to_uppercase();
            match lang_upper.as_str() {
                "ZH" | "ZH-CN" | "ZH-TW" | "HANS" | "HANT" => {
                    if self.quick_detector.has_chinese(text) {
                        return true;
                    }
                }
                "JA" => {
                    if self.quick_detector.has_japanese(text) {
                        return true;
                    }
                }
                "KO" => {
                    if self.quick_detector.has_korean(text) {
                        return true;
                    }
                }
                "EN" | "EN-US" | "EN-GB" => {
                    if self.quick_detector.is_latin(text) {
                        return true;
                    }
                }
                "AR" => {
                    if self.quick_detector.has_arabic(text) {
                        return true;
                    }
                }
                "RU" | "UK" | "BG" => {
                    if self.quick_detector.has_cyrillic(text) {
                        return true;
                    }
                }
                _ => return true,
            }
        }

        false
    }
}

impl Filter for LanguageFilter {
    fn should_translate(&self, text: &str) -> bool {
        let is_target = self.is_target_language(text);
        let has_source = self.contains_source_language(text);

        if self.source_langs.is_empty() {
            if is_target {
                debug!(
                    target_lang = %self.target_lang,
                    reason = "already_in_target_language",
                    "Text filtered by language check (AUTO mode)"
                );
                return false;
            }
            return true;
        }

        if !has_source {
            debug!(
                source_langs = ?self.source_langs,
                reason = "no_source_language",
                "Text filtered by language check"
            );
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "LanguageFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_filter(source_langs: Vec<&str>, target_lang: &str) -> LanguageFilter {
        LanguageFilter::new(
            source_langs.iter().map(|s| s.to_string()).collect(),
            target_lang.to_string(),
        )
    }

    #[test]
    fn test_no_language_restriction() {
        let filter = create_filter(vec![], "EN");
        assert!(!filter.should_translate("Hello"));
        assert!(filter.should_translate("你好"));
    }

    #[test]
    fn test_chinese_filter() {
        let filter = create_filter(vec!["zh"], "EN");

        assert!(filter.should_translate("你好世界"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_japanese_filter() {
        let filter = create_filter(vec!["ja"], "EN");

        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("カタカナ"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_korean_filter() {
        let filter = create_filter(vec!["ko"], "EN");

        assert!(filter.should_translate("안녕하세요"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_english_filter() {
        let filter = create_filter(vec!["en"], "ZH");

        assert!(filter.should_translate("Hello World"));
        assert!(!filter.should_translate("你好世界"));
    }

    #[test]
    fn test_auto_mode_skip_target() {
        let filter = create_filter(vec!["AUTO"], "EN");

        assert!(!filter.should_translate("Hello World"));
        assert!(!filter.should_translate("This is a test"));
        assert!(filter.should_translate("你好世界"));
    }

    #[test]
    fn test_auto_mode_skip_chinese() {
        let filter = create_filter(vec!["AUTO"], "ZH");

        assert!(!filter.should_translate("你好世界"));
        assert!(filter.should_translate("Hello World"));
    }

    #[test]
    fn test_mixed_content() {
        let filter = create_filter(vec!["zh"], "EN");

        assert!(filter.should_translate("Hello 你好"));
    }

    #[test]
    fn test_arabic_filter() {
        let filter = create_filter(vec!["ar"], "EN");

        assert!(filter.should_translate("مرحبا بالعالم"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_cyrillic_filter() {
        let filter = create_filter(vec!["ru"], "EN");

        assert!(filter.should_translate("Привет мир"));
        assert!(!filter.should_translate("Hello World"));
    }
}
