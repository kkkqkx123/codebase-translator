//! Pattern check
//!
//! Check 3: Regex pattern matching (O(n) where n is number of patterns)
//! - Keyword exclusion
//! - Pattern exclusion/inclusion
//! - Placeholder detection
//! - Code pattern detection

use crate::parser::filtering::config::FilterConfig;
use crate::parser::filtering::traits::Filter;
use regex::Regex;
use tracing::debug;

/// Pattern filter for regex-based matching
pub struct PatternFilter {
    exclude_keywords_regex: Vec<Regex>,
    exclude_patterns_regex: Vec<Regex>,
    include_patterns_regex: Vec<Regex>,
    placeholder_regex: Vec<Regex>,
    code_pattern_regex: Vec<Regex>,
    url_pattern_regex: Regex, // URL 模式优先检测
    allow_placeholders: bool,
    detect_code_patterns: bool,
}

impl PatternFilter {
    /// Create a new pattern filter
    pub fn new(config: &FilterConfig) -> crate::core::error::Result<Self> {
        // Compile exclude keywords as word-boundary regexes
        let exclude_keywords_regex = config
            .exclude_keywords
            .iter()
            .map(|kw| Regex::new(&format!(r"\b{}\b", regex::escape(kw))))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::core::error::TranslateError::Config(format!(
                    "Invalid exclude keyword regex: {}",
                    e
                ))
            })?;

        // Compile exclude patterns
        let exclude_patterns_regex = config
            .exclude_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::core::error::TranslateError::Config(format!(
                    "Invalid exclude pattern regex: {}",
                    e
                ))
            })?;

        // Compile include patterns
        let include_patterns_regex = config
            .include_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::core::error::TranslateError::Config(format!(
                    "Invalid include pattern regex: {}",
                    e
                ))
            })?;

        // Placeholder patterns
        let placeholder_regex = vec![
            Regex::new(r"%[sdvf]").expect("Invalid placeholder regex"),
            Regex::new(r"\$\d{1,2}\b").expect("Invalid placeholder regex"),
            Regex::new(r"\$\{[^}]*\}").expect("Invalid placeholder regex"),
            Regex::new(r"\{[^}]*\}").expect("Invalid placeholder regex"),
        ];

        // Code pattern detection
        let code_pattern_regex = vec![
            Regex::new(r"\w+\.\w+").expect("Invalid code pattern regex"), // Member access
            Regex::new(r"\w+\([^)]*\)").expect("Invalid code pattern regex"), // Function call
            Regex::new(r"\{[^}]*\}").expect("Invalid code pattern regex"), // Braces
            Regex::new(r"\[[^\]]*\]").expect("Invalid code pattern regex"), // Brackets
        ];

        // URL pattern - 优先于代码模式检测
        let url_pattern_regex = Regex::new(r"https?://[^\s]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
            .expect("Invalid URL pattern regex");

        Ok(Self {
            exclude_keywords_regex,
            exclude_patterns_regex,
            include_patterns_regex,
            placeholder_regex,
            code_pattern_regex,
            url_pattern_regex,
            allow_placeholders: config.allow_placeholders,
            detect_code_patterns: config.detect_code_patterns,
        })
    }

    /// Check if text contains placeholders
    pub fn contains_placeholder(&self, text: &str) -> bool {
        self.placeholder_regex.iter().any(|p| p.is_match(text))
    }

    /// Check if text contains code patterns
    pub fn contains_code_pattern(&self, text: &str) -> bool {
        self.code_pattern_regex.iter().any(|p| p.is_match(text))
    }
}

impl Filter for PatternFilter {
    fn should_translate(&self, text: &str) -> bool {
        // 1. URL pattern check - 代价低（单个正则）且常见，优先检测
        // 避免 URL 中的点号被误判为代码模式
        if self.url_pattern_regex.is_match(text) {
            debug!(reason = "contains_url", "Text filtered by pattern check");
            return false;
        }

        // 2. Placeholder check - 代价低（固定4个简单正则）且非常常见
        if !self.allow_placeholders {
            for pattern in &self.placeholder_regex {
                if pattern.is_match(text) {
                    debug!(
                        reason = "contains_placeholder",
                        "Text filtered by pattern check"
                    );
                    return false;
                }
            }
        }

        // 3. Exclude keywords check - 数量少，简单匹配
        for pattern in &self.exclude_keywords_regex {
            if pattern.is_match(text) {
                debug!(reason = "excluded_keyword", "Text filtered by pattern check");
                return false;
            }
        }

        // 4. Exclude patterns check - 用户自定义，数量不确定
        for pattern in &self.exclude_patterns_regex {
            if pattern.is_match(text) {
                debug!(reason = "excluded_pattern", "Text filtered by pattern check");
                return false;
            }
        }

        // 5. Include patterns check - 白名单逻辑，必须检查所有
        if !self.include_patterns_regex.is_empty() {
            let included = self.include_patterns_regex.iter().any(|p| p.is_match(text));
            if !included {
                debug!(
                    reason = "not_in_include_patterns",
                    "Text filtered by pattern check"
                );
                return false;
            }
        }

        // 6. Code pattern check - 代价较高（4个正则），且可能被占位符设置跳过
        if self.detect_code_patterns {
            for pattern in &self.code_pattern_regex {
                // Skip brace pattern check when placeholders are allowed
                // since braces like {name} are commonly used as placeholders
                if self.allow_placeholders && pattern.as_str() == r"\{[^}]*\}" {
                    continue;
                }
                if pattern.is_match(text) {
                    debug!(
                        reason = "contains_code_pattern",
                        "Text filtered by pattern check"
                    );
                    return false;
                }
            }
        }

        true
    }

    fn name(&self) -> &str {
        "PatternFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_filtering() {
        let config = FilterConfig::default();
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("TODO: fix this"));
        assert!(!filter.should_translate("Copyright 2024"));
        assert!(filter.should_translate("Hello world"));
    }

    #[test]
    fn test_url_filtering() {
        let config = FilterConfig::default();
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("Visit https://example.com"));
        assert!(!filter.should_translate("Email test@example.com"));
    }

    #[test]
    fn test_url_priority_over_code_patterns() {
        // 验证 URL 检测优先于代码模式检测
        // 即使 detect_code_patterns 启用，URL 也不会被误判为代码模式
        let config = FilterConfig {
            detect_code_patterns: true,
            // 清空 exclude_patterns，确保 URL 不是被默认的 exclude_patterns 过滤
            exclude_patterns: vec![],
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        // URL 应该被过滤（优先检测）
        assert!(!filter.should_translate("Visit https://example.com for more info"));
        assert!(!filter.should_translate("Check sub.domain.org now"));
        assert!(!filter.should_translate("Contact admin@company.com"));

        // 纯代码模式（不含 URL）应该被过滤
        assert!(!filter.should_translate("object.method()"));
        assert!(!filter.should_translate("func(arg1, arg2)"));

        // 普通文本应该通过
        assert!(filter.should_translate("Hello world"));
        assert!(filter.should_translate("This is a normal sentence."));
    }

    #[test]
    fn test_placeholder_filtering() {
        let config = FilterConfig {
            allow_placeholders: false,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("Hello %s"));
        assert!(!filter.should_translate("Value: {name}"));
    }

    #[test]
    fn test_allow_placeholders() {
        let config = FilterConfig {
            allow_placeholders: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.should_translate("Hello %s"));
        assert!(filter.should_translate("Value: {name}"));
    }

    #[test]
    fn test_include_patterns_whitelist() {
        // 当 include_patterns 设置时，只有匹配的内容才会被翻译
        let config = FilterConfig {
            include_patterns: vec![r"translate_me".to_string()],
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        // 匹配 include_patterns 的内容应该被翻译
        assert!(filter.should_translate("Please translate_me today"));
        assert!(filter.should_translate("translate_me is important"));

        // 不匹配的内容应该被过滤
        assert!(!filter.should_translate("Hello world"));
        assert!(!filter.should_translate("Do not translate this"));
    }

    #[test]
    fn test_include_patterns_multiple() {
        // 测试多个 include_patterns
        let config = FilterConfig {
            include_patterns: vec![r"^PREFIX_".to_string(), r"_SUFFIX$".to_string()],
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.should_translate("PREFIX_hello"));
        assert!(filter.should_translate("hello_SUFFIX"));
        assert!(!filter.should_translate("middle_text"));
    }

    #[test]
    fn test_exclude_patterns_custom() {
        // 测试自定义 exclude_patterns
        let config = FilterConfig {
            exclude_patterns: vec![r"secret:\s*\w+".to_string(), r"password\d*".to_string()],
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("secret: token123"));
        assert!(!filter.should_translate("password123 field"));
        assert!(!filter.should_translate("my password"));
        assert!(filter.should_translate("Hello world"));
        assert!(filter.should_translate("This is safe text"));
    }

    #[test]
    fn test_detect_code_patterns_enabled() {
        // 测试代码模式检测（启用状态）
        let config = FilterConfig {
            detect_code_patterns: true,
            allow_placeholders: true, // 允许占位符，避免与代码模式冲突
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        // 代码模式应该被过滤
        assert!(!filter.should_translate("object.method()"));
        assert!(!filter.should_translate("func(arg1, arg2)"));
        assert!(!filter.should_translate("array[index]"));

        // 普通文本应该通过
        assert!(filter.should_translate("Hello world"));
        assert!(filter.should_translate("This is a normal sentence."));
    }

    #[test]
    fn test_detect_code_patterns_disabled() {
        // 测试代码模式检测（禁用状态）
        let config = FilterConfig {
            detect_code_patterns: false,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        // 代码模式应该被允许
        assert!(filter.should_translate("object.method()"));
        assert!(filter.should_translate("func(arg1, arg2)"));
        assert!(filter.should_translate("array[index]"));
    }

    #[test]
    fn test_placeholder_variations() {
        let config = FilterConfig {
            allow_placeholders: false,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        // 各种占位符格式都应该被检测
        assert!(!filter.should_translate("Hello %s"));
        assert!(!filter.should_translate("Number: %d"));
        assert!(!filter.should_translate("Float: %f"));
        assert!(!filter.should_translate("Value: %v"));
        assert!(!filter.should_translate("Arg $1 and $2"));
        assert!(!filter.should_translate("Template: ${variable}"));
        assert!(!filter.should_translate("Format: {name}"));
    }

    #[test]
    fn test_markdown_patterns() {
        let config = FilterConfig::default();
        let filter = PatternFilter::new(&config).unwrap();

        // Markdown 链接和图片应该被过滤
        assert!(!filter.should_translate("[link text](https://example.com)"));
        assert!(!filter.should_translate("![alt text](image.png)"));
        assert!(!filter.should_translate("<div>HTML tag</div>"));
        assert!(!filter.should_translate("`inline code`"));
    }

    #[test]
    fn test_empty_patterns() {
        // 测试空配置（启用代码模式检测，URL 会被优先处理）
        let config = FilterConfig {
            exclude_keywords: vec![],
            exclude_patterns: vec![],
            include_patterns: vec![],
            detect_code_patterns: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        // 普通文本应该通过
        assert!(filter.should_translate("Hello world"));
        assert!(filter.should_translate("TODO something"));

        // URL 应该被过滤（优先于代码模式检测）
        assert!(!filter.should_translate("Visit https://example.com"));
    }

    #[test]
    fn test_contains_placeholder_method() {
        let config = FilterConfig::default();
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.contains_placeholder("Hello %s"));
        assert!(filter.contains_placeholder("Value: {name}"));
        assert!(filter.contains_placeholder("Args: $1 $2"));
        assert!(!filter.contains_placeholder("Plain text"));
    }

    #[test]
    fn test_contains_code_pattern_method() {
        let config = FilterConfig::default();
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.contains_code_pattern("obj.method"));
        assert!(filter.contains_code_pattern("func()"));
        assert!(filter.contains_code_pattern("{key: value}"));
        assert!(filter.contains_code_pattern("[item1, item2]"));
        assert!(!filter.contains_code_pattern("Plain text"));
    }

    #[test]
    fn test_invalid_keyword_regex() {
        // 测试关键字正则处理 - regex::escape 会转义特殊字符
        // 所以即使包含正则特殊字符的关键字也能正常编译
        let config = FilterConfig {
            exclude_keywords: vec!["[invalid".to_string(), "(test".to_string(), "+plus".to_string()],
            ..Default::default()
        };
        let result = PatternFilter::new(&config);
        // regex::escape 会转义这些字符，所以不会报错
        assert!(result.is_ok());

        // 验证 Filter 可以正常工作
        let filter = result.unwrap();
        assert_eq!(filter.name(), "PatternFilter");
    }

    #[test]
    fn test_invalid_exclude_pattern_regex() {
        // 测试无效的排除模式正则
        let config = FilterConfig {
            exclude_patterns: vec!["(unclosed".to_string()], // 未闭合的分组
            ..Default::default()
        };
        let result = PatternFilter::new(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_include_pattern_regex() {
        // 测试无效的包含模式正则
        let config = FilterConfig {
            include_patterns: vec!["*invalid".to_string()], // 量词没有前置表达式
            ..Default::default()
        };
        let result = PatternFilter::new(&config);
        assert!(result.is_err());
    }
}
