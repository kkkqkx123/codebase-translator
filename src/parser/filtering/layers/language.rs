//! Language filter layer
//!
//! Layer 2: Quick language detection (O(k) where k=32)
//! - Source language matching
//! - CJK character detection

use crate::parser::filtering::config::FilterConfig;
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Language filter for source language validation
pub struct LanguageFilter {
    source_langs: Vec<String>,
}

impl LanguageFilter {
    /// Create a new language filter
    pub fn new(config: &FilterConfig) -> Self {
        Self {
            source_langs: config.source_langs.clone(),
        }
    }

    /// Quick detection of CJK characters in text
    /// Only checks the first 32 characters for performance
    fn quick_detect_cjk(text: &str) -> bool {
        text.chars().take(32).any(|c| {
            matches!(c,
                '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
                '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
                '\u{20000}'..='\u{2A6DF}' | // CJK Extension B
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
                "Text filtered by language layer"
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
        let config = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);
        assert!(filter.should_translate("anything"));
    }
}
