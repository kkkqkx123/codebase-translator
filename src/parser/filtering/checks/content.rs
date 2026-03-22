//! Content check
//!
//! Check 4: Content analysis (O(len) - most expensive)
//! - Symbol-only text detection
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
        // Symbol-only check
        if Self::is_only_symbols(text) {
            debug!(reason = "only_symbols", "Text filtered by content check");
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

        // 空字符串的 chars().all() 返回 true，所以 is_only_symbols 返回 true
        // should_translate 返回 false（空字符串应由 LengthFilter 处理）
        assert!(!filter.should_translate(""));
    }

    #[test]
    fn test_mixed_content_symbols_and_text() {
        let filter = ContentFilter::new();

        // 符号 + 文本的混合内容应该被翻译
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

        // 各种空白字符组合
        assert!(!filter.should_translate("     ")); // 多个空格
        assert!(!filter.should_translate("\t\t\t")); // 制表符
        assert!(!filter.should_translate("\n\n")); // 换行符
        assert!(!filter.should_translate("  \t\n  ")); // 混合空白

        // 空白 + 文本应该被翻译
        assert!(filter.should_translate("  hello  "));
        assert!(filter.should_translate("\ttext\n"));
    }

    #[test]
    fn test_punctuation_edge_cases() {
        let filter = ContentFilter::new();

        // 单个标点符号
        assert!(!filter.should_translate("!"));
        assert!(!filter.should_translate("."));
        assert!(!filter.should_translate(","));
        assert!(!filter.should_translate("@"));

        // 多个标点符号
        assert!(!filter.should_translate("!!!"));
        assert!(!filter.should_translate("..."));
        assert!(!filter.should_translate("?!?"));

        // 标点符号 + 空格
        assert!(!filter.should_translate("! ! !"));
    }

    #[test]
    fn test_unicode_punctuation() {
        let filter = ContentFilter::new();

        // 中文标点符号 - 应该被视为普通字符（因为 is_punctuation 只检查 ASCII）
        assert!(filter.should_translate("你好，世界！"));
        assert!(filter.should_translate("测试。"));
        assert!(filter.should_translate("问题？"));

        // 纯中文标点应该被翻译（不是 ASCII 标点）
        assert!(filter.should_translate("，。！"));
        assert!(filter.should_translate("……"));
        assert!(filter.should_translate("「」"));
    }

    #[test]
    fn test_numbers_and_symbols() {
        let filter = ContentFilter::new();

        // 纯数字 + 符号（数字不是标点符号，所以应该被翻译）
        assert!(filter.should_translate("123!@#"));
        // 纯符号应该被过滤
        assert!(!filter.should_translate("$%^&*"));

        // 数字 + 字母应该被翻译
        assert!(filter.should_translate("Version 1.0"));
        assert!(filter.should_translate("Item #123"));
    }

    #[test]
    fn test_code_like_content() {
        let filter = ContentFilter::new();

        // 代码样内容但包含字母
        assert!(filter.should_translate("fn main()"));
        assert!(filter.should_translate("var x = 1;"));
        assert!(filter.should_translate("#include <stdio.h>"));

        // 纯符号代码片段
        assert!(!filter.should_translate("(){}[]"));
        assert!(!filter.should_translate("->::"));
    }

    #[test]
    fn test_is_punctuation_all_cases() {
        // 测试所有被定义的标点符号
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

        // 非标点符号
        assert!(!ContentFilter::is_punctuation('a'));
        assert!(!ContentFilter::is_punctuation('1'));
        assert!(!ContentFilter::is_punctuation('中'));
        assert!(!ContentFilter::is_punctuation(' '));
        assert!(!ContentFilter::is_punctuation('\n'));
        assert!(!ContentFilter::is_punctuation('，')); // 中文逗号
        assert!(!ContentFilter::is_punctuation('。')); // 中文句号
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

        // 两者行为应该一致
        assert_eq!(
            filter1.should_translate("test"),
            filter2.should_translate("test")
        );
        assert_eq!(
            filter1.should_translate("!@#"),
            filter2.should_translate("!@#")
        );
    }
}
