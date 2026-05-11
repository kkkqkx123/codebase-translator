//! Pattern check
//!
//! Check 3: Regex pattern matching (O(n) where n is number of patterns)
//! - Keyword exclusion
//! - Pattern exclusion/inclusion
//! - Placeholder detection
//! - Code pattern detection

use crate::config::project::FilterConfig;
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
    url_pattern_regex: Regex,
    allow_placeholders: bool,
    detect_code_patterns: bool,
}

impl PatternFilter {
    /// Create a new pattern filter
    pub fn new(config: &FilterConfig) -> crate::core::error::Result<Self> {
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

        let placeholder_regex = if config.placeholder_patterns.is_empty() {
            Self::default_placeholder_patterns()
        } else {
            Self::compile_patterns(&config.placeholder_patterns)?
        };

        let code_pattern_regex = if config.code_patterns.is_empty() {
            Self::default_code_patterns()
        } else {
            Self::compile_patterns(&config.code_patterns)?
        };

        let url_pattern_regex =
            Regex::new(r"https?://[^\s]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
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

    fn default_placeholder_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"%[sdvf]").expect("Invalid placeholder regex"),
            Regex::new(r"\$\d{1,2}\b").expect("Invalid placeholder regex"),
            Regex::new(r"\$\{[^}]*\}").expect("Invalid placeholder regex"),
            Regex::new(r"\{[a-zA-Z_][a-zA-Z0-9_]*\}").expect("Invalid placeholder regex"),
        ]
    }

    fn default_code_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"\w+\([^)]*\)").expect("Invalid code pattern regex"),
            Regex::new(r"\w+\[[^\]]*\]").expect("Invalid code pattern regex"),
            Regex::new(r"`[^`]+`").expect("Invalid code pattern regex"),
        ]
    }

    fn compile_patterns(patterns: &[String]) -> crate::core::error::Result<Vec<Regex>> {
        patterns
            .iter()
            .map(|p| {
                Regex::new(p).map_err(|e| {
                    crate::core::error::TranslateError::Config(format!(
                        "Invalid pattern regex '{}': {}",
                        p, e
                    ))
                })
            })
            .collect()
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
        self.should_translate_with_options(text, true)
    }

    fn name(&self) -> &str {
        "PatternFilter"
    }
}

impl PatternFilter {
    /// Check if text should be translated with options
    ///
    /// # Arguments
    /// * `text` - The text to check
    /// * `check_code_patterns` - Whether to check for code patterns (false for comments)
    pub fn should_translate_with_options(&self, text: &str, check_code_patterns: bool) -> bool {
        if self.url_pattern_regex.is_match(text) {
            debug!(reason = "contains_url", "Text filtered by pattern check");
            return false;
        }

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

        for pattern in &self.exclude_keywords_regex {
            if pattern.is_match(text) {
                debug!(
                    reason = "excluded_keyword",
                    "Text filtered by pattern check"
                );
                return false;
            }
        }

        for pattern in &self.exclude_patterns_regex {
            if pattern.is_match(text) {
                debug!(
                    reason = "excluded_pattern",
                    "Text filtered by pattern check"
                );
                return false;
            }
        }

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

        if self.detect_code_patterns && check_code_patterns {
            for pattern in &self.code_pattern_regex {
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
        let config = FilterConfig {
            detect_code_patterns: true,
            exclude_patterns: vec![],
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("Visit https://example.com for more info"));
        assert!(!filter.should_translate("Contact admin@company.com"));
        assert!(!filter.should_translate("func(arg1, arg2)"));
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
        let config = FilterConfig {
            include_patterns: vec![r"translate_me".to_string()],
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.should_translate("Please translate_me today"));
        assert!(filter.should_translate("translate_me is important"));
        assert!(!filter.should_translate("Hello world"));
        assert!(!filter.should_translate("Do not translate this"));
    }

    #[test]
    fn test_include_patterns_multiple() {
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
        let config = FilterConfig {
            detect_code_patterns: true,
            allow_placeholders: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("func(arg1, arg2)"));
        assert!(!filter.should_translate("array[index]"));
        assert!(filter.should_translate("Hello world"));
        assert!(filter.should_translate("This is a normal sentence."));
    }

    #[test]
    fn test_detect_code_patterns_disabled() {
        let config = FilterConfig {
            detect_code_patterns: false,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

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

        assert!(!filter.should_translate("[link text](https://example.com)"));
        assert!(!filter.should_translate("`inline code`"));
    }

    #[test]
    fn test_empty_patterns() {
        let config = FilterConfig {
            exclude_keywords: vec![],
            exclude_patterns: vec![],
            include_patterns: vec![],
            detect_code_patterns: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(filter.should_translate("TODO something"));
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

        assert!(filter.contains_code_pattern("func()"));
        assert!(filter.contains_code_pattern("array[index]"));
        assert!(filter.contains_code_pattern("`code`"));
        assert!(!filter.contains_code_pattern("Plain text"));
    }

    #[test]
    fn test_invalid_keyword_regex() {
        let config = FilterConfig {
            exclude_keywords: vec![
                "[invalid".to_string(),
                "(test".to_string(),
                "+plus".to_string(),
            ],
            ..Default::default()
        };
        let result = PatternFilter::new(&config);
        assert!(result.is_ok());

        let filter = result.unwrap();
        assert_eq!(filter.name(), "PatternFilter");
    }

    #[test]
    fn test_invalid_exclude_pattern_regex() {
        let config = FilterConfig {
            exclude_patterns: vec!["(unclosed".to_string()],
            ..Default::default()
        };
        let result = PatternFilter::new(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_include_pattern_regex() {
        let config = FilterConfig {
            include_patterns: vec!["*invalid".to_string()],
            ..Default::default()
        };
        let result = PatternFilter::new(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_improved_patterns_no_false_positives() {
        let config = FilterConfig {
            detect_code_patterns: true,
            allow_placeholders: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.should_translate("Mr. Smith"));
        assert!(filter.should_translate("Version 2.0"));
        assert!(filter.should_translate("e.g. this is an example"));
        assert!(filter.should_translate("Price: 100-200 yuan"));
        assert!(filter.should_translate("See section 3.2.1"));

        assert!(!filter.should_translate("func()"));
        assert!(!filter.should_translate("array[index]"));
        assert!(!filter.should_translate("`code`"));
    }

    #[test]
    fn test_custom_placeholder_patterns_override() {
        let config = FilterConfig {
            placeholder_patterns: vec![r"\{[0-9]+\}".to_string(), r"%[a-zA-Z]+".to_string()],
            allow_placeholders: false,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("Value: {0}"));
        assert!(!filter.should_translate("Hello %s"));
        assert!(filter.should_translate("Value: ${name}"));
        assert!(filter.should_translate("Template: {name}"));
    }

    #[test]
    fn test_custom_code_patterns_override() {
        let config = FilterConfig {
            code_patterns: vec![r"\w+\(\)".to_string()],
            detect_code_patterns: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("func()"));
        assert!(filter.should_translate("array[index]"));
        assert!(filter.should_translate("`code`"));
    }

    #[test]
    fn test_empty_custom_patterns_uses_defaults() {
        let config = FilterConfig {
            placeholder_patterns: vec![],
            code_patterns: vec![],
            allow_placeholders: false,
            detect_code_patterns: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(!filter.should_translate("Hello %s"));
        assert!(!filter.should_translate("Value: {name}"));
        assert!(!filter.should_translate("func()"));
        assert!(!filter.should_translate("array[index]"));
    }

    #[test]
    fn test_should_translate_with_options_code_patterns_disabled() {
        let config = FilterConfig {
            detect_code_patterns: true,
            allow_placeholders: true,
            ..Default::default()
        };
        let filter = PatternFilter::new(&config).unwrap();

        assert!(filter.should_translate_with_options("支持数组索引访问（如 items[0].name）", false));
        assert!(filter.should_translate_with_options("调用 func(arg1, arg2) 函数", false));
        assert!(!filter.should_translate_with_options("支持数组索引访问（如 items[0].name）", true));
        assert!(!filter.should_translate_with_options("调用 func(arg1, arg2) 函数", true));
    }
}
