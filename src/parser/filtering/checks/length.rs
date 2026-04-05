//! Length check
//!
//! Check 1: O(1) constant-time checks
//! - Empty text detection
//! - Minimum/Maximum length validation

use crate::config::project::FilterConfig;
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Length filter for O(1) constant-time checks
pub struct LengthFilter {
    min_length: usize,
    max_length: usize,
}

impl LengthFilter {
    /// Create a new length filter
    pub fn new(config: &FilterConfig) -> Self {
        Self {
            min_length: config.min_length,
            max_length: config.max_length,
        }
    }
}

impl Filter for LengthFilter {
    fn should_translate(&self, text: &str) -> bool {
        if text.trim().is_empty() {
            debug!(
                reason = "empty_or_whitespace",
                "Text filtered by length check"
            );
            return false;
        }

        let len = text.len();

        if self.min_length > 0 && len < self.min_length {
            debug!(
                length = len,
                min_length = self.min_length,
                reason = "too_short",
                "Text filtered by length check"
            );
            return false;
        }

        if self.max_length > 0 && len > self.max_length {
            debug!(
                length = len,
                max_length = self.max_length,
                reason = "too_long",
                "Text filtered by length check"
            );
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "LengthFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let config = FilterConfig::default();
        let filter = LengthFilter::new(&config);

        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("   "));
        assert!(!filter.should_translate("\t\t\t"));
        assert!(!filter.should_translate("\n\n"));
        assert!(!filter.should_translate("  \t\n  "));
    }

    #[test]
    fn test_max_length_filtering() {
        let config = FilterConfig {
            max_length: 10,
            ..Default::default()
        };
        let filter = LengthFilter::new(&config);

        assert!(filter.should_translate("abc"));
        assert!(filter.should_translate("abcdefghij"));
        assert!(!filter.should_translate("abcdefghijk"));
    }

    #[test]
    fn test_min_length_filtering() {
        let config = FilterConfig {
            min_length: 5,
            ..Default::default()
        };
        let filter = LengthFilter::new(&config);

        assert!(!filter.should_translate("abc"));
        assert!(!filter.should_translate("abcd"));
        assert!(filter.should_translate("abcde"));
        assert!(filter.should_translate("abcdefghij"));
    }

    #[test]
    fn test_no_max_length_limit() {
        let config = FilterConfig {
            max_length: 0,
            ..Default::default()
        };
        let filter = LengthFilter::new(&config);

        assert!(filter.should_translate("a"));
        assert!(filter.should_translate("this is a very long text that exceeds normal limits"));
    }
}
