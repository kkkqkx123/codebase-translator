//! Content check
//!
//! Check 4: Content analysis (O(len) - most expensive)
//! - Symbol-only text detection
//! - Number-only text detection
//! - Number+symbol combination detection
//! - Character type analysis

use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Content filter for deep content analysis
pub struct ContentFilter;

impl ContentFilter {
    /// Create a new content filter
    pub fn new() -> Self {
        Self
    }

    /// Check if text contains only symbols/whitespace
    fn is_only_symbols(text: &str) -> bool {
        text.chars()
            .all(|c| c.is_whitespace() || Self::is_punctuation(c))
    }

    /// Check if text contains only digits and/or symbols (no translatable content)
    fn is_only_digits_and_symbols(text: &str) -> bool {
        text.chars()
            .all(|c| c.is_whitespace() || c.is_ascii_digit() || Self::is_punctuation(c))
    }

    /// Check if text contains only digits (pure numeric)
    fn is_only_digits(text: &str) -> bool {
        text.chars().all(|c| c.is_whitespace() || c.is_ascii_digit())
    }

    /// Check if text has any translatable content (letters or CJK characters)
    fn has_translatable_content(text: &str) -> bool {
        text.chars().any(Self::is_translatable_char)
    }

    /// Check if character is translatable (part of natural language)
    fn is_translatable_char(c: char) -> bool {
        if Self::is_unicode_special_char(c) {
            return false;
        }
        Self::is_letter_char(c)
    }

    /// Check if character is a letter (part of natural language writing systems)
    fn is_letter_char(c: char) -> bool {
        let cp = c as u32;

        match cp {
            // Basic Latin uppercase (A-Z)
            0x0041..=0x005A => true,
            // Basic Latin lowercase (a-z)
            0x0061..=0x007A => true,
            // Latin Extended (À-ÿ, Ā-ſ, etc.)
            0x00C0..=0x024F => true,
            // Greek and Coptic
            0x0370..=0x03FF => true,
            // Greek Extended
            0x1F00..=0x1FFF => true,
            // Cyrillic
            0x0400..=0x04FF => true,
            // Cyrillic Extended
            0x0500..=0x052F => true,
            // Hebrew
            0x0590..=0x05FF => true,
            // Arabic
            0x0600..=0x06FF => true,
            // Arabic Extended
            0x0750..=0x077F => true,
            // Devanagari (Hindi, etc.)
            0x0900..=0x097F => true,
            // Thai
            0x0E00..=0x0E7F => true,
            // Hiragana (Japanese)
            0x3040..=0x309F => true,
            // Katakana (Japanese)
            0x30A0..=0x30FF => true,
            // Bopomofo
            0x3100..=0x312F => true,
            // Hangul Compatibility Jamo
            0x3130..=0x318F => true,
            // CJK Unified Ideographs
            0x4E00..=0x9FFF => true,
            // CJK Extension A
            0x3400..=0x4DBF => true,
            // Hangul Syllables (Korean)
            0xAC00..=0xD7AF => true,
            // CJK Extension B and beyond (surrogate pairs handled by Rust)
            0x20000..=0x2A6DF => true,
            // CJK Extension C-F
            0x2A700..=0x2CEAF => true,
            // CJK Compatibility Ideographs
            0xF900..=0xFAFF => true,
            _ => false,
        }
    }

    /// Check if character is a Unicode special character (not translatable)
    fn is_unicode_special_char(c: char) -> bool {
        let cp = c as u32;

        match cp {
            // ASCII punctuation and symbols
            0x0021..=0x002F => true, // ! " # $ % & ' ( ) * + , - . /
            0x003A..=0x0040 => true, // : ; < = > ? @
            0x005B..=0x0060 => true, // [ \ ] ^ _ `
            0x007B..=0x007E => true, // { | } ~
            // Latin-1 Supplement (symbols and punctuation)
            0x00A0..=0x00BF => true,
            // Spacing Modifier Letters
            0x02B0..=0x02FF => true,
            // Combining Diacritical Marks
            0x0300..=0x036F => true,
            // Currency Symbols
            0x20A0..=0x20CF => true,
            // Letterlike Symbols
            0x2100..=0x214F => true,
            // Number Forms
            0x2150..=0x218F => true,
            // Arrows
            0x2190..=0x21FF => true,
            // Mathematical Operators
            0x2200..=0x22FF => true,
            // Miscellaneous Technical
            0x2300..=0x23FF => true,
            // Control Pictures
            0x2400..=0x243F => true,
            // Box Drawing
            0x2500..=0x257F => true,
            // Block Elements
            0x2580..=0x259F => true,
            // Geometric Shapes
            0x25A0..=0x25FF => true,
            // Miscellaneous Symbols
            0x2600..=0x26FF => true,
            // Dingbats
            0x2700..=0x27BF => true,
            // Miscellaneous Mathematical Symbols
            0x27C0..=0x27EF => true,
            // Supplemental Arrows
            0x27F0..=0x27FF => true,
            // Supplemental Mathematical Operators
            0x2A00..=0x2AFF => true,
            // Miscellaneous Symbols and Arrows
            0x2B00..=0x2BFF => true,
            // Supplemental Punctuation
            0x2E00..=0x2E7F => true,
            // CJK Symbols and Punctuation
            0x3000..=0x303F => true,
            // Halfwidth and Fullwidth Forms
            0xFF00..=0xFFEF => true,
            // Emoticons
            0x1F600..=0x1F64F => true,
            // Miscellaneous Symbols and Pictographs
            0x1F300..=0x1F5FF => true,
            // Transport and Map Symbols
            0x1F680..=0x1F6FF => true,
            // Supplemental Symbols and Pictographs
            0x1F900..=0x1F9FF => true,
            // Chess Symbols (includes Symbols and Pictographs Extended-A)
            0x1FA00..=0x1FAFF => true,
            _ => false,
        }
    }

    /// Check if character is punctuation
    fn is_punctuation(c: char) -> bool {
        matches!(
            c,
            '!' | '"'
                | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | '-'
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '?'
                | '@'
                | '['
                | '\\'
                | ']'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
        )
    }
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for ContentFilter {
    fn should_translate(&self, text: &str) -> bool {
        if Self::is_only_symbols(text) {
            debug!(reason = "only_symbols", "Text filtered by content check");
            return false;
        }

        if Self::is_only_digits(text) {
            debug!(reason = "only_digits", "Text filtered by content check");
            return false;
        }

        if Self::is_only_digits_and_symbols(text) {
            debug!(
                reason = "only_digits_and_symbols",
                "Text filtered by content check"
            );
            return false;
        }

        if !Self::has_translatable_content(text) {
            debug!(
                reason = "no_translatable_content",
                "Text filtered by content check"
            );
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "ContentFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_only() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("!@#$%"));
        assert!(!filter.should_translate("   "));
        assert!(!filter.should_translate("..."));
        assert!(filter.should_translate("Hello!"));
        assert!(filter.should_translate("你好"));
    }

    #[test]
    fn test_empty_string() {
        let filter = ContentFilter::new();
        assert!(!filter.should_translate(""));
    }

    #[test]
    fn test_digits_only() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("10"));
        assert!(!filter.should_translate("123"));
        assert!(!filter.should_translate("0"));
        assert!(!filter.should_translate(" 123 "));
        assert!(!filter.should_translate("1 2 3"));
    }

    #[test]
    fn test_digits_and_symbols() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("10!"));
        assert!(!filter.should_translate("123@#$"));
        assert!(!filter.should_translate("(123)"));
        assert!(!filter.should_translate("1.0"));
        assert!(!filter.should_translate("10,20,30"));
        assert!(!filter.should_translate("100%"));
        assert!(!filter.should_translate("$100"));
        assert!(!filter.should_translate("#123"));
    }

    #[test]
    fn test_mixed_content_symbols_and_text() {
        let filter = ContentFilter::new();

        assert!(filter.should_translate("Hello!"));
        assert!(filter.should_translate("Hello, World!"));
        assert!(filter.should_translate("What's up?"));
        assert!(filter.should_translate("Test..."));
        assert!(filter.should_translate("Code: function()"));
        assert!(filter.should_translate("Price: $100"));
    }

    #[test]
    fn test_whitespace_variations() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("     "));
        assert!(!filter.should_translate("\t\t\t"));
        assert!(!filter.should_translate("\n\n"));
        assert!(!filter.should_translate("  \t\n  "));

        assert!(filter.should_translate("  hello  "));
        assert!(filter.should_translate("\ttext\n"));
    }

    #[test]
    fn test_punctuation_edge_cases() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("!"));
        assert!(!filter.should_translate("."));
        assert!(!filter.should_translate(","));
        assert!(!filter.should_translate("@"));

        assert!(!filter.should_translate("!!!"));
        assert!(!filter.should_translate("..."));
        assert!(!filter.should_translate("?!?"));

        assert!(!filter.should_translate("! ! !"));
    }

    #[test]
    fn test_unicode_punctuation() {
        let filter = ContentFilter::new();

        assert!(filter.should_translate("你好，世界！"));
        assert!(filter.should_translate("测试。"));
        assert!(filter.should_translate("问题？"));

        assert!(!filter.should_translate("，。！"));
        assert!(!filter.should_translate("……"));
        assert!(!filter.should_translate("「」"));
    }

    #[test]
    fn test_numbers_and_symbols() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("123!@#"));
        assert!(!filter.should_translate("$%^&*"));

        assert!(filter.should_translate("Version 1.0"));
        assert!(filter.should_translate("Item #123"));
    }

    #[test]
    fn test_code_like_content() {
        let filter = ContentFilter::new();

        assert!(filter.should_translate("fn main()"));
        assert!(filter.should_translate("var x = 1;"));
        assert!(filter.should_translate("#include <stdio.h>"));

        assert!(!filter.should_translate("(){}[]"));
        assert!(!filter.should_translate("->::"));
    }

    #[test]
    fn test_translatable_content_detection() {
        let filter = ContentFilter::new();

        assert!(filter.should_translate("Hello"));
        assert!(filter.should_translate("你好"));
        assert!(filter.should_translate("こんにちは"));
        assert!(filter.should_translate("안녕하세요"));
        assert!(filter.should_translate("Привет"));
        assert!(filter.should_translate("مرحبا"));

        assert!(!filter.should_translate("123"));
        assert!(!filter.should_translate("!@#"));
        assert!(!filter.should_translate("123!@#"));
    }

    #[test]
    fn test_is_punctuation_all_cases() {
        let punctuation_chars = [
            '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';',
            '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
        ];

        for c in punctuation_chars {
            assert!(
                ContentFilter::is_punctuation(c),
                "Character '{}' should be recognized as punctuation",
                c
            );
        }

        assert!(!ContentFilter::is_punctuation('a'));
        assert!(!ContentFilter::is_punctuation('1'));
        assert!(!ContentFilter::is_punctuation('中'));
        assert!(!ContentFilter::is_punctuation(' '));
        assert!(!ContentFilter::is_punctuation('\n'));
        assert!(!ContentFilter::is_punctuation('，'));
        assert!(!ContentFilter::is_punctuation('。'));
    }

    #[test]
    fn test_filter_name() {
        let filter = ContentFilter::new();
        assert_eq!(filter.name(), "ContentFilter");
    }

    #[test]
    fn test_default_constructor() {
        let filter1 = ContentFilter::new();
        let filter2 = ContentFilter;

        assert_eq!(
            filter1.should_translate("test"),
            filter2.should_translate("test")
        );
        assert_eq!(
            filter1.should_translate("!@#"),
            filter2.should_translate("!@#")
        );
    }

    #[test]
    fn test_expect_to_contain_pattern() {
        let filter = ContentFilter::new();

        assert!(!filter.should_translate("10"));
        assert!(!filter.should_translate("\"10\""));
        assert!(filter.should_translate("expect(result).toContain(\"10\")"));
    }
}
