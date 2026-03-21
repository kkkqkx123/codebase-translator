//! Basic filter layer
//!
//! Layer 1: O(1) constant-time checks
//! - Empty text check
//! - Maximum length validation

use crate::parser::filtering::config::FilterConfig;
use crate::parser::filtering::traits::Filter;
use tracing::debug;

/// Basic filter for O(1) constant-time checks
pub struct BasicFilter {
    max_length: usize,
}

impl BasicFilter {
    /// Create a new basic filter
    pub fn new(config: &FilterConfig) -> Self {
        Self {
            max_length: config.max_length,
        }
    }
}

impl Filter for BasicFilter {
    fn should_translate(&self, text: &str) -> bool {
        // Empty check
        if text.is_empty() {
            debug!(reason = "empty", "Text filtered by basic layer");
            return false;
        }

        // Maximum length check
        let len = text.len();
        if self.max_length > 0 && len > self.max_length {
            debug!(
                length = len,
                max_length = self.max_length,
                reason = "too_long",
                "Text filtered by basic layer"
            );
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "BasicFilter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let config = FilterConfig::default();
        let filter = BasicFilter::new(&config);
        assert!(!filter.should_translate(""));
    }

    #[test]
    fn test_max_length_filtering() {
        let config = FilterConfig {
            max_length: 10,
            ..Default::default()
        };
        let filter = BasicFilter::new(&config);

        assert!(filter.should_translate("abc")); // short is ok
        assert!(filter.should_translate("abcdefghij")); // exact max
        assert!(!filter.should_translate("abcdefghijk")); // too long
    }

    #[test]
    fn test_no_max_length_limit() {
        let config = FilterConfig {
            max_length: 0,
            ..Default::default()
        };
        let filter = BasicFilter::new(&config);

        assert!(filter.should_translate("a"));
        assert!(filter.should_translate("this is a very long text that exceeds normal limits"));
    }
}
