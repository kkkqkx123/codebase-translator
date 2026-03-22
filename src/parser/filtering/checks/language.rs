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

    #[test]
    fn test_japanese_hiragana() {
        // 日文平假名检测
        let config = FilterConfig {
            source_langs: vec!["ja".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        assert!(filter.should_translate("こんにちは")); // 平假名
        assert!(filter.should_translate("ひらがな")); // 平假名
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_japanese_katakana() {
        // 日文片假名检测
        let config = FilterConfig {
            source_langs: vec!["ja".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        assert!(filter.should_translate("カタカナ")); // 片假名
        assert!(filter.should_translate("コンピュータ")); // 片假名
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_korean_detection() {
        // 韩文检测
        let config = FilterConfig {
            source_langs: vec!["ko".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        assert!(filter.should_translate("안녕하세요")); // 韩文
        assert!(filter.should_translate("한글")); // 韩文
        assert!(filter.should_translate("컴퓨터")); // 韩文
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_mixed_language_content() {
        // 多语言混合内容
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 中文 + 英文混合
        assert!(filter.should_translate("Hello 你好"));
        assert!(filter.should_translate("你好 world"));

        // 中文 + 日文混合
        assert!(filter.should_translate("你好こんにちは"));

        // 中文 + 韩文混合
        assert!(filter.should_translate("你好안녕하세요"));
    }

    #[test]
    fn test_empty_string() {
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 空字符串应该被允许（留给 LengthFilter 处理）
        assert!(filter.should_translate(""));
    }

    #[test]
    fn test_whitespace_only() {
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 纯空白应该被允许
        assert!(filter.should_translate("   "));
        assert!(filter.should_translate("\t\n"));
    }

    #[test]
    fn test_cjk_32_chars_limit() {
        // 测试只检查前32个字符的优化
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 前32个字符是英文，后面是中文
        let text = "abcdefghijklmnopqrstuvwxyz123456你好世界";
        assert!(!filter.should_translate(text)); // 前32字符没有中文

        // 前32个字符包含中文
        let text2 = "abcdefghijklmnop你好qrstuvwxyz123456";
        assert!(filter.should_translate(text2));
    }

    #[test]
    fn test_english_32_chars_limit() {
        // 测试英文检测也只检查前32个字符
        let config = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "EN".to_string(),
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 前32个字符是中文，后面是英文
        let text = "你好世界这是一个很长的中文字符串Hello World";
        assert!(filter.should_translate(text)); // 前32字符主要是中文

        // 前32个字符是英文
        let text2 = "This is an English text. 这是后面的中文";
        assert!(!filter.should_translate(text2));
    }

    #[test]
    fn test_chinese_variants() {
        // 测试中文变体
        let config_zh = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter_zh = LanguageFilter::new(&config_zh);

        let config_zhcn = FilterConfig {
            source_langs: vec!["zh-CN".to_string()],
            ..Default::default()
        };
        let filter_zhcn = LanguageFilter::new(&config_zhcn);

        let config_zhtw = FilterConfig {
            source_langs: vec!["zh-TW".to_string()],
            ..Default::default()
        };
        let filter_zhtw = LanguageFilter::new(&config_zhtw);

        // 所有中文变体都应该检测中文
        assert!(filter_zh.should_translate("你好世界"));
        assert!(filter_zhcn.should_translate("你好世界"));
        assert!(filter_zhtw.should_translate("你好世界"));
    }

    #[test]
    fn test_target_language_variants() {
        // 测试目标语言的各种变体
        let config_en_us = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "EN-US".to_string(),
            ..Default::default()
        };
        let filter_en_us = LanguageFilter::new(&config_en_us);

        assert!(!filter_en_us.should_translate("Hello World")); // 已经是目标语言
        assert!(filter_en_us.should_translate("你好世界"));

        let config_zh_cn = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "ZH-CN".to_string(),
            ..Default::default()
        };
        let filter_zh_cn = LanguageFilter::new(&config_zh_cn);

        assert!(!filter_zh_cn.should_translate("你好世界")); // 已经是目标语言
        assert!(filter_zh_cn.should_translate("Hello World"));
    }

    #[test]
    fn test_unknown_target_language() {
        // 未知目标语言应该允许翻译（保守策略）
        let config = FilterConfig {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "FR".to_string(), // 法语检测未实现
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 所有内容都应该被允许翻译
        assert!(filter.should_translate("Hello World"));
        assert!(filter.should_translate("你好世界"));
        assert!(filter.should_translate("こんにちは"));
    }

    #[test]
    fn test_filter_name() {
        let config = FilterConfig::default();
        let filter = LanguageFilter::new(&config);
        assert_eq!(filter.name(), "LanguageFilter");
    }

    #[test]
    fn test_cjk_radicals_and_symbols() {
        // CJK 部首和符号
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        assert!(filter.should_translate("⽂字")); // CJK 部首
        assert!(filter.should_translate("「引用」")); // CJK 符号
        assert!(filter.should_translate("【括号】")); // CJK 符号
    }

    #[test]
    fn test_special_characters_and_emojis() {
        let config = FilterConfig {
            source_langs: vec!["zh".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // Emoji 不应该影响检测
        assert!(!filter.should_translate("Hello World 🎉"));
        assert!(filter.should_translate("你好 🎉"));

        // 特殊字符
        assert!(!filter.should_translate("© 2024"));
        assert!(!filter.should_translate("Hello™"));
    }

    #[test]
    fn test_multiple_source_languages() {
        // 多个源语言
        let config = FilterConfig {
            source_langs: vec!["zh".to_string(), "ja".to_string()],
            ..Default::default()
        };
        let filter = LanguageFilter::new(&config);

        // 中文应该通过
        assert!(filter.should_translate("你好世界"));

        // 日文应该通过
        assert!(filter.should_translate("こんにちは"));

        // 纯英文不应该通过（因为需要 CJK）
        assert!(!filter.should_translate("Hello World"));
    }

    #[test]
    fn test_english_edge_cases() {
        // 英文检测边界情况
        assert!(!LanguageFilter::quick_detect_english("")); // 空字符串
        assert!(!LanguageFilter::quick_detect_english("   ")); // 纯空白
        assert!(!LanguageFilter::quick_detect_english("123 456")); // 纯数字
        assert!(!LanguageFilter::quick_detect_english("!@#$%")); // 纯符号

        // 接近70%边界的情况
        assert!(LanguageFilter::quick_detect_english("abc123")); // 3/6 = 50% - 实际是字母3个，非空白6个
        assert!(LanguageFilter::quick_detect_english("Hello World!!!")); // 字母10个，非空白13个 -> 76%
    }
}
