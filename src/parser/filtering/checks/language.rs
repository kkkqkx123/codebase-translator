//! Language check
//!
//! Check 2: Quick language detection (O(k) where k=32)
//! - Source language matching
//! - CJK character detection
//! - Target language detection (to avoid translating already-translated content)

use crate::parser::filtering::config::FilterConfig;
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Language filter for source language validation
pub struct LanguageFilter {
    source_langs: Vec<String>,
    target_lang: String,
}

impl LanguageFilter {
    /// Create a new language filter
    pub fn new(config: &FilterConfig) -> Self {
        Self {
            source_langs: config.source_langs.clone(),
            target_lang: config.target_lang.clone(),
        }
    }

    /// Quick detection of CJK characters in text
    /// Only checks the first 32 characters for performance
    fn quick_detect_cjk(text: &str) -> bool {
        text.chars().take(32).any(|c| {
            matches!(c,
                '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
                '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
                '\u{20000}'..='\u{2A6DF}' |  // CJK Extension B
                '\u{3040}'..='\u{309F}' |  // Hiragana
                '\u{30A0}'..='\u{30FF}' |  // Katakana
                '\u{AC00}'..='\u{D7AF}' |  // Hangul Syllables
                '\u{1100}'..='\u{11FF}' |  // Hangul Jamo
                '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility
                '\u{2E80}'..='\u{2EFF}' |  // CJK Radicals
                '\u{2F00}'..='\u{2FDF}' |  // Kangxi Radicals
                '\u{2FF0}'..='\u{2FFF}' |  // Ideographic Description
                '\u{3000}'..='\u{303F}' |  // CJK Symbols
                '\u{FF00}'..='\u{FFEF}'    // Halfwidth/Fullwidth
            )
        })
    }

    /// Quick detection of Latin/English characters
    /// Returns true if text appears to be primarily English/Latin
    fn quick_detect_english(text: &str) -> bool {
        let sample = text.chars().take(32).collect::<String>();
        if sample.is_empty() {
            return false;
        }

        // Count Latin characters
        let latin_count = sample.chars().filter(|c| c.is_ascii_alphabetic()).count();
        let total_chars = sample.chars().filter(|c| !c.is_whitespace()).count();

        if total_chars == 0 {
            return false;
        }

        // If more than 70% of characters are Latin, consider it English
        (latin_count as f64 / total_chars as f64) > 0.7
    }

    /// Check if text is already in target language (to avoid re-translating)
    fn is_target_language(&self, text: &str) -> bool {
        let target = self.target_lang.to_uppercase();

        match target.as_str() {
            "EN" | "EN-US" | "EN-GB" => Self::quick_detect_english(text),
            "ZH" | "ZH-CN" | "ZH-TW" => Self::quick_detect_cjk(text),
            // For other languages, allow translation (conservative approach)
            _ => false,
        }
    }

    /// Check if text contains target language characters
    fn contains_target_language(&self, text: &str) -> bool {
        if self.source_langs.is_empty() {
            return true;
        }

        // Check if AUTO mode is enabled
        if self
            .source_langs
            .iter()
            .any(|lang| lang.to_uppercase() == "AUTO")
        {
            // In AUTO mode, skip if text is already in target language
            if self.is_target_language(text) {
                return false;
            }
            return true;
        }

        // Check if any source language requires CJK characters
        let requires_cjk = self.source_langs.iter().any(|lang| {
            lang == "zh" || lang == "zh-CN" || lang == "zh-TW" || lang == "ja" || lang == "ko"
        });

        if requires_cjk {
            Self::quick_detect_cjk(text)
        } else {
            true
        }
    }
}

impl Filter for LanguageFilter {
    fn should_translate(&self, text: &str) -> bool {
        if self.source_langs.is_empty() {
            return true;
        }

        if !self.contains_target_language(text) {
            debug!(
                source_langs = ?self.source_langs,
                reason = "no_target_language",
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

    #[test]
    fn test_no_language_restriction() {
        let config = FilterConfig::default();
        let filter = LanguageFilter::new(&config);
        assert!(filter.should_translate("Hello"));
        assert!(filter.should_translate("你好"));
    }

    #[test]
    fn test_cjk_detection() {
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        assert!(filter.should_translate("你好世界"));
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_auto_mode() {
        // In AUTO mode with default target EN, Chinese text should be translated
        // but English text should be skipped (already in target language)
        let config = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // Chinese text should be translated
        assert!(filter.should_translate("你好世界"));

        // English text should NOT be translated (already in target language)
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_auto_mode_skip_english_when_target_is_en() {
        // When target is EN and source is AUTO, English text should be skipped
        let config = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "EN".to_string(),
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // English text should NOT be translated (already in target language)
        assert!(!filter.should_translate("Hello World"));
        assert!(!filter.should_translate("This is a simple test"));

        // Chinese text should be translated
        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("这是一个测试"));
    }

    #[test]
    fn test_auto_mode_skip_chinese_when_target_is_zh() {
        // When target is ZH and source is AUTO, Chinese text should be skipped
        let config = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "ZH".to_string(),
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // Chinese text should NOT be translated (already in target language)
        assert!(!filter.should_translate("你好世界"));

        // English text should be translated
        assert!(filter.should_translate("Hello World"));
    }

    #[test]
    fn test_english_detection() {
        // Test various English texts
        assert!(LanguageFilter::quick_detect_english("Hello World"));
        assert!(LanguageFilter::quick_detect_english("This is a test"));
        assert!(LanguageFilter::quick_detect_english("function main()"));

        // Test non-English texts
        assert!(!LanguageFilter::quick_detect_english("你好世界"));
        assert!(!LanguageFilter::quick_detect_english("こんにちは"));
    }
}
