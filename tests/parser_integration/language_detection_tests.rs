//! Language Detection Integration Tests
//!
//! Tests for language and script detection capabilities.

use codebase_translate::parser::{LanguageDetector, LanguageInfo, Script};

mod script_detection_tests {
    use super::*;

    #[test]
    fn test_detect_latin_script() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hello, world!");
        assert_eq!(info.script, Script::Latin, "Should detect Latin script");
        assert!(info.has_chars, "Should have characters");
    }

    #[test]
    fn test_detect_cjk_script() {
        let detector = LanguageDetector::new();

        let info = detector.detect("你好世界");
        assert_eq!(info.script, Script::Cjk, "Should detect CJK script");
        assert!(info.has_chars);
    }

    #[test]
    fn test_detect_cjk_japanese() {
        let detector = LanguageDetector::new();

        let info = detector.detect("こんにちは");
        assert_eq!(info.script, Script::Cjk, "Should detect CJK script for Japanese");
    }

    #[test]
    fn test_detect_cjk_korean() {
        let detector = LanguageDetector::new();

        let info = detector.detect("안녕하세요");
        assert_eq!(info.script, Script::Cjk, "Should detect CJK script for Korean");
    }

    #[test]
    fn test_detect_arabic_script() {
        let detector = LanguageDetector::new();

        let info = detector.detect("مرحبا بالعالم");
        assert_eq!(info.script, Script::Arabic, "Should detect Arabic script");
    }

    #[test]
    fn test_detect_hebrew_script() {
        let detector = LanguageDetector::new();

        let info = detector.detect("שלום עולם");
        assert_eq!(info.script, Script::Hebrew, "Should detect Hebrew script");
    }

    #[test]
    fn test_detect_greek_script() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Γειά σου Κόσμε");
        assert_eq!(info.script, Script::Greek, "Should detect Greek script");
    }

    #[test]
    fn test_detect_cyrillic_script() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Привет мир");
        assert_eq!(info.script, Script::Cyrillic, "Should detect Cyrillic script");
    }

    #[test]
    fn test_detect_only_symbols() {
        let detector = LanguageDetector::new();

        let info = detector.detect("!@#$%^&*()");
        assert_eq!(info.script, Script::Unknown, "Should detect unknown script for symbols");
        assert!(!info.has_chars, "Should not have characters");
    }

    #[test]
    fn test_detect_empty_string() {
        let detector = LanguageDetector::new();

        let info = detector.detect("");
        assert_eq!(info.script, Script::Unknown, "Should detect unknown script for empty string");
        assert!(!info.has_chars);
    }

    #[test]
    fn test_detect_whitespace_only() {
        let detector = LanguageDetector::new();

        let info = detector.detect("   \n\t  ");
        assert_eq!(info.script, Script::Unknown, "Should detect unknown script for whitespace");
        assert!(!info.has_chars);
    }

    #[test]
    fn test_detect_numbers_only() {
        let detector = LanguageDetector::new();

        let info = detector.detect("12345");
        assert_eq!(info.script, Script::Unknown, "Should detect unknown script for numbers");
        assert!(!info.has_chars);
    }
}

mod language_identification_tests {
    use super::*;

    #[test]
    fn test_identify_english() {
        let detector = LanguageDetector::new();

        let info = detector.detect("This is an English sentence.");
        
        assert!(
            info.contains_lang("English") || info.langs.iter().any(|l| l.contains("English")),
            "Should identify English language"
        );
    }

    #[test]
    fn test_identify_chinese() {
        let detector = LanguageDetector::new();

        let info = detector.detect("这是一个中文句子。");
        
        assert!(
            info.contains_lang("Chinese") || info.langs.iter().any(|l| l.contains("Chinese")),
            "Should identify Chinese language"
        );
    }

    #[test]
    fn test_identify_japanese() {
        let detector = LanguageDetector::new();

        let info = detector.detect("これは日本語の文章です。");
        
        assert!(
            info.contains_lang("Japanese") || info.langs.iter().any(|l| l.contains("Japanese")),
            "Should identify Japanese language"
        );
    }

    #[test]
    fn test_identify_korean() {
        let detector = LanguageDetector::new();

        let info = detector.detect("이것은 한국어 문장입니다.");
        
        assert!(
            info.contains_lang("Korean") || info.langs.iter().any(|l| l.contains("Korean")),
            "Should identify Korean language"
        );
    }

    #[test]
    fn test_identify_russian() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Это русское предложение.");
        
        assert!(
            info.contains_lang("Russian") || info.langs.iter().any(|l| l.contains("Russian")),
            "Should identify Russian language"
        );
    }

    #[test]
    fn test_identify_german() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Das ist ein deutscher Satz.");
        
        assert!(
            info.contains_lang("German") || info.langs.iter().any(|l| l.contains("German")),
            "Should identify German language"
        );
    }

    #[test]
    fn test_identify_french() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Ceci est une phrase française.");
        
        assert!(
            info.contains_lang("French") || info.langs.iter().any(|l| l.contains("French")),
            "Should identify French language"
        );
    }

    #[test]
    fn test_identify_spanish() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Esta es una oración en español.");
        
        assert!(
            info.contains_lang("Spanish") || info.langs.iter().any(|l| l.contains("Spanish")),
            "Should identify Spanish language"
        );
    }
}

mod allowed_languages_tests {
    use super::*;

    #[test]
    fn test_allowed_languages_filter() {
        let allowed = vec!["English".to_string(), "Chinese".to_string()];
        let detector = LanguageDetector::with_allowed_langs(allowed);

        let english_info = detector.detect("Hello world");
        assert!(
            english_info.contains_lang("English"),
            "Should allow English"
        );

        let chinese_info = detector.detect("你好世界");
        assert!(
            chinese_info.contains_lang("Chinese"),
            "Should allow Chinese"
        );

        let french_info = detector.detect("Bonjour le monde");
        assert!(
            !french_info.contains_lang("French"),
            "Should not allow French when not in allowed list"
        );
    }

    #[test]
    fn test_allowed_scripts_filter() {
        let allowed = vec![Script::Latin, Script::Cjk];
        let detector = LanguageDetector::with_allowed_scripts(allowed);

        let latin_info = detector.detect("Hello");
        assert_eq!(latin_info.script, Script::Latin);

        let cjk_info = detector.detect("你好");
        assert_eq!(cjk_info.script, Script::Cjk);

        let arabic_info = detector.detect("مرحبا");
        assert_eq!(arabic_info.script, Script::Unknown, "Should filter Arabic script");
    }
}

mod mixed_content_tests {
    use super::*;

    #[test]
    fn test_mixed_latin_cjk() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hello 世界");
        
        assert!(
            info.script == Script::Latin || info.script == Script::Cjk,
            "Should detect one of the scripts in mixed content"
        );
    }

    #[test]
    fn test_code_comment_with_chinese() {
        let detector = LanguageDetector::new();

        let info = detector.detect("// 这是一个中文注释");
        
        assert_eq!(info.script, Script::Cjk, "Should detect CJK in code comment");
    }

    #[test]
    fn test_code_comment_with_english() {
        let detector = LanguageDetector::new();

        let info = detector.detect("// This is an English comment");
        
        assert_eq!(info.script, Script::Latin, "Should detect Latin in code comment");
    }

    #[test]
    fn test_multilingual_text() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hello 你好 Bonjour");
        
        assert!(
            info.has_chars,
            "Should detect characters in multilingual text"
        );
    }
}

mod language_info_tests {
    use super::*;

    #[test]
    fn test_language_info_primary() {
        let info = LanguageInfo {
            script: Script::Latin,
            langs: vec!["English".to_string(), "German".to_string()],
            has_chars: true,
        };

        assert_eq!(info.primary(), Some("English"), "Should return primary language");
    }

    #[test]
    fn test_language_info_primary_empty() {
        let info = LanguageInfo {
            script: Script::Unknown,
            langs: vec![],
            has_chars: false,
        };

        assert_eq!(info.primary(), None, "Should return None for empty languages");
    }

    #[test]
    fn test_language_info_contains_lang() {
        let info = LanguageInfo {
            script: Script::Latin,
            langs: vec!["English".to_string(), "French".to_string()],
            has_chars: true,
        };

        assert!(info.contains_lang("English"), "Should contain English");
        assert!(info.contains_lang("english"), "Should be case-insensitive");
        assert!(!info.contains_lang("German"), "Should not contain German");
    }

    #[test]
    fn test_script_as_str() {
        assert_eq!(Script::Latin.as_str(), "Latin");
        assert_eq!(Script::Cjk.as_str(), "CJK");
        assert_eq!(Script::Arabic.as_str(), "Arabic");
        assert_eq!(Script::Hebrew.as_str(), "Hebrew");
        assert_eq!(Script::Greek.as_str(), "Greek");
        assert_eq!(Script::Cyrillic.as_str(), "Cyrillic");
        assert_eq!(Script::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_script_display() {
        assert_eq!(format!("{}", Script::Latin), "Latin");
        assert_eq!(format!("{}", Script::Cjk), "CJK");
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_very_short_text() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hi");
        assert!(info.has_chars, "Should detect characters in short text");
    }

    #[test]
    fn test_single_character() {
        let detector = LanguageDetector::new();

        let info = detector.detect("A");
        assert!(info.has_chars);
        assert_eq!(info.script, Script::Latin);
    }

    #[test]
    fn test_single_cjk_character() {
        let detector = LanguageDetector::new();

        let info = detector.detect("中");
        assert!(info.has_chars);
        assert_eq!(info.script, Script::Cjk);
    }

    #[test]
    fn test_very_long_text() {
        let detector = LanguageDetector::new();

        let long_text = "This is a very long text. ".repeat(1000);
        let info = detector.detect(&long_text);
        
        assert!(info.has_chars);
        assert_eq!(info.script, Script::Latin);
    }

    #[test]
    fn test_special_characters() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hello! @#$%^&*() World?");
        assert_eq!(info.script, Script::Latin, "Should ignore special characters");
    }

    #[test]
    fn test_emojis() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hello 👋 World 🌍");
        assert_eq!(info.script, Script::Latin, "Should handle emojis");
    }

    #[test]
    fn test_mixed_scripts_priority() {
        let detector = LanguageDetector::new();

        let info = detector.detect("Hello Привет 你好");
        
        assert!(
            info.script == Script::Latin || 
            info.script == Script::Cyrillic || 
            info.script == Script::Cjk,
            "Should detect one of the scripts"
        );
    }
}

