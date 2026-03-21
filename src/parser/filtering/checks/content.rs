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
        text.chars().all(|c| c.is_whitespace() || Self::is_punctuation(c))
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
}
