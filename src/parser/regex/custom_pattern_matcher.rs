//! Custom pattern matcher for simple regex-based extraction
//!
//! Provides a simple regex-based pattern matcher for custom extraction patterns.
//! Unlike state machines, this supports single-step regex matching with capture groups.

use regex::Regex;
use tracing::{debug, info, trace};

use crate::config::project::CustomRegexPattern;
use crate::core::error::{Result, TranslateError};
use crate::core::models::Position;

/// A match result from custom pattern
#[derive(Debug, Clone)]
pub struct CustomPatternMatch {
    /// Complete matched content (including format markers)
    pub raw_content: String,
    /// Extracted text content (for translation)
    pub extracted_text: String,
    /// Start position in source
    pub start_pos: Position,
    /// End position in source
    pub end_pos: Position,
    /// Pattern name for identification
    pub pattern_name: String,
}

/// Custom pattern matcher for simple regex-based extraction
pub struct CustomPatternMatcher {
    /// Pattern name for identification
    pub name: String,
    /// Compiled regex pattern
    regex: Regex,
    /// Capture group index to extract
    group: usize,
    /// File extensions this pattern applies to
    file_extensions: Vec<String>,
}

impl CustomPatternMatcher {
    /// Create a new custom pattern matcher from configuration
    pub fn from_config(pattern: &CustomRegexPattern) -> Result<Self> {
        debug!(
            name = %pattern.name,
            regex = %pattern.regex,
            group = pattern.group,
            extensions_count = pattern.file_extensions.len(),
            "Creating custom pattern matcher"
        );

        let regex = Regex::new(&pattern.regex).map_err(|e| {
            TranslateError::Config(format!(
                "Invalid regex in pattern '{}': {}",
                pattern.name, e
            ))
        })?;

        info!(
            name = %pattern.name,
            "Custom pattern matcher created successfully"
        );

        Ok(Self {
            name: pattern.name.clone(),
            regex,
            group: pattern.group,
            file_extensions: pattern.file_extensions.clone(),
        })
    }

    /// Check if this pattern applies to a given file extension
    pub fn applies_to_extension(&self, extension: &str) -> bool {
        if self.file_extensions.is_empty() {
            return true;
        }

        let ext_lower = extension.to_lowercase();
        self.file_extensions
            .iter()
            .any(|e| e.to_lowercase() == ext_lower || e == "*")
    }

    /// Find all matches in the content
    pub fn find_matches(&self, content: &str) -> Result<Vec<CustomPatternMatch>> {
        trace!(
            name = %self.name,
            content_length = content.len(),
            "Finding matches"
        );

        let mut matches = Vec::new();

        for mat in self.regex.find_iter(content) {
            let raw_content = mat.as_str().to_string();
            let extracted_text = if let Some(captured) = self.regex.captures(&raw_content) {
                if let Some(group) = captured.get(self.group) {
                    group.as_str().to_string()
                } else {
                    raw_content.clone()
                }
            } else {
                raw_content.clone()
            };

            let start_pos = Position::new(0, 0, mat.start());

            let end_pos = Position::new(0, 0, mat.end());

            matches.push(CustomPatternMatch {
                raw_content,
                extracted_text,
                start_pos,
                end_pos,
                pattern_name: self.name.clone(),
            });
        }

        trace!(
            name = %self.name,
            matches_count = matches.len(),
            "Matches found"
        );

        Ok(matches)
    }

    /// Get the file extensions this pattern applies to
    pub fn file_extensions(&self) -> &[String] {
        &self.file_extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_pattern_matcher_creation() {
        let pattern = CustomRegexPattern {
            name: "test_pattern".to_string(),
            file_extensions: vec!["js".to_string(), "ts".to_string()],
            category: crate::config::project::StringLiteralCategory::ErrorHandling,
            regex: r#"Error:\s*([^"]+)"#.to_string(),
            group: 1,
        };

        let matcher = CustomPatternMatcher::from_config(&pattern).unwrap();
        assert_eq!(matcher.name, "test_pattern");
        assert!(matcher.applies_to_extension("js"));
        assert!(matcher.applies_to_extension("ts"));
        assert!(!matcher.applies_to_extension("py"));
    }

    #[test]
    fn test_custom_pattern_matcher_all_extensions() {
        let pattern = CustomRegexPattern {
            name: "universal_pattern".to_string(),
            file_extensions: vec![],
            category: crate::config::project::StringLiteralCategory::Other,
            regex: r#"TODO:\s*(.+)"#.to_string(),
            group: 1,
        };

        let matcher = CustomPatternMatcher::from_config(&pattern).unwrap();
        assert!(matcher.applies_to_extension("js"));
        assert!(matcher.applies_to_extension("py"));
        assert!(matcher.applies_to_extension("rs"));
    }

    #[test]
    fn test_custom_pattern_matching() {
        let pattern = CustomRegexPattern {
            name: "error_pattern".to_string(),
            file_extensions: vec![],
            category: crate::config::project::StringLiteralCategory::ErrorHandling,
            regex: r#"Error:\s*"([^"]+)""#.to_string(),
            group: 1,
        };

        let matcher = CustomPatternMatcher::from_config(&pattern).unwrap();
        let content = r#"
            Error: "Invalid input"
            Error: "File not found"
            Error: "Access denied"
        "#;

        let matches = matcher.find_matches(content).unwrap();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].extracted_text, "Invalid input");
        assert_eq!(matches[1].extracted_text, "File not found");
        assert_eq!(matches[2].extracted_text, "Access denied");
    }

    #[test]
    fn test_custom_pattern_with_group_zero() {
        let pattern = CustomRegexPattern {
            name: "full_match_pattern".to_string(),
            file_extensions: vec![],
            category: crate::config::project::StringLiteralCategory::Other,
            regex: r#"TODO:\s*.+"#.to_string(),
            group: 0,
        };

        let matcher = CustomPatternMatcher::from_config(&pattern).unwrap();
        let content = "TODO: Fix this bug\nTODO: Add tests";

        let matches = matcher.find_matches(content).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].extracted_text, "TODO: Fix this bug");
        assert_eq!(matches[1].extracted_text, "TODO: Add tests");
    }
}
