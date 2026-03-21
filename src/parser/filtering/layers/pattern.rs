//! Pattern filter layer
//!
//! Layer 3: Regex pattern matching (O(n) where n is number of patterns)
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

        Ok(Self {
            exclude_keywords_regex,
            exclude_patterns_regex,
            include_patterns_regex,
            placeholder_regex,
            code_pattern_regex,
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
        // Exclude keywords check
        for pattern in &self.exclude_keywords_regex {
            if pattern.is_match(text) {
                debug!(reason = "excluded_keyword", "Text filtered by pattern layer");
                return false;
            }
        }

        // Exclude patterns check
        for pattern in &self.exclude_patterns_regex {
            if pattern.is_match(text) {
                debug!(reason = "excluded_pattern", "Text filtered by pattern layer");
                return false;
            }
        }

        // Include patterns check
        if !self.include_patterns_regex.is_empty() {
            let included = self.include_patterns_regex.iter().any(|p| p.is_match(text));
            if !included {
                debug!(
                    reason = "not_in_include_patterns",
                    "Text filtered by pattern layer"
                );
                return false;
            }
        }

        // Placeholder check
        if !self.allow_placeholders {
            for pattern in &self.placeholder_regex {
                if pattern.is_match(text) {
                    debug!(
                        reason = "contains_placeholder",
                        "Text filtered by pattern layer"
                    );
                    return false;
                }
            }
        }

        // Code pattern check
        if self.detect_code_patterns {
            for pattern in &self.code_pattern_regex {
                if pattern.is_match(text) {
                    debug!(
                        reason = "contains_code_pattern",
                        "Text filtered by pattern layer"
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
}
