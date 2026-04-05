//! Placeholder protection for template strings
//!
//! Provides functionality to protect placeholders during translation
//! and restore them afterwards.

use crate::parser::scanner::region::TextRegion;

/// Placeholder protector for template strings
pub struct PlaceholderProtector {
    /// Marker prefix for placeholders
    marker_prefix: String,
    /// Marker suffix for placeholders
    marker_suffix: String,
}

impl Default for PlaceholderProtector {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceholderProtector {
    /// Create a new placeholder protector
    pub fn new() -> Self {
        Self {
            marker_prefix: "__PH_".to_string(),
            marker_suffix: "__".to_string(),
        }
    }

    /// Create with custom markers
    pub fn with_markers(prefix: impl Into<String>, suffix: impl Into<String>) -> Self {
        Self {
            marker_prefix: prefix.into(),
            marker_suffix: suffix.into(),
        }
    }

    /// Prepare text for translation by replacing placeholders with markers
    pub fn prepare_for_translation(&self, region: &TextRegion, content: &str) -> String {
        let text = match region.extract_content(content) {
            Some(t) => t,
            None => return String::new(),
        };

        if region.placeholders.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();
        let mut sorted = region.placeholders.clone();
        sorted.sort_by(|a, b| b.start.cmp(&a.start));

        for (idx, placeholder) in sorted.iter().enumerate() {
            let marker = self.make_marker(region.placeholders.len() - 1 - idx);
            if placeholder.end <= result.len() {
                result.replace_range(placeholder.start..placeholder.end, &marker);
            }
        }

        result
    }

    /// Restore placeholders in translated text
    pub fn restore_placeholders(&self, translated: &str, region: &TextRegion) -> String {
        if region.placeholders.is_empty() {
            return translated.to_string();
        }

        let mut result = translated.to_string();

        for (idx, placeholder) in region.placeholders.iter().enumerate() {
            let marker = self.make_marker(idx);
            result = result.replace(&marker, &placeholder.original);
        }

        result
    }

    /// Make a marker for the given index
    fn make_marker(&self, index: usize) -> String {
        format!("{}{}{}", self.marker_prefix, index, self.marker_suffix)
    }

    /// Check if text contains any placeholder markers
    pub fn contains_markers(&self, text: &str) -> bool {
        text.contains(&self.marker_prefix)
    }

    /// Extract all markers from text
    pub fn extract_markers(&self, text: &str) -> Vec<(usize, usize, usize)> {
        let mut markers = Vec::new();
        let marker_pattern = format!("{}(\\d+){}", self.marker_prefix, self.marker_suffix);

        if let Ok(re) = regex::Regex::new(&marker_pattern) {
            for cap in re.captures_iter(text) {
                if let (Some(full), Some(idx)) = (cap.get(0), cap.get(1)) {
                    if let Ok(index) = idx.as_str().parse::<usize>() {
                        markers.push((full.start(), full.end(), index));
                    }
                }
            }
        }

        markers
    }
}

/// Format protection utilities
pub struct FormatProtector;

impl FormatProtector {
    /// Protect printf-style format specifiers
    pub fn protect_printf_format(text: &str) -> (String, Vec<(usize, String)>) {
        let mut replacements = Vec::new();
        let mut result = text.to_string();
        let mut idx = 0;

        let re =
            regex::Regex::new(r"%[+-]?\d*(?:\.\d+)?[diouxXeEfFgGaAcspn%]").expect("Invalid regex");

        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(0) {
                let marker = format!("__FMT_{}__", idx);
                replacements.push((idx, m.as_str().to_string()));
                result = result.replacen(m.as_str(), &marker, 1);
                idx += 1;
            }
        }

        (result, replacements)
    }

    /// Restore printf-style format specifiers
    pub fn restore_printf_format(text: &str, replacements: &[(usize, String)]) -> String {
        let mut result = text.to_string();
        for (idx, original) in replacements.iter().rev() {
            let marker = format!("__FMT_{}__", idx);
            result = result.replace(&marker, original);
        }
        result
    }

    /// Protect Python format specifiers
    pub fn protect_python_format(text: &str) -> (String, Vec<(usize, String)>) {
        let mut replacements = Vec::new();
        let mut result = text.to_string();
        let mut idx = 0;

        // Match {name}, {0}, {name:format}
        let re = regex::Regex::new(r"\{[^}]*\}").expect("Invalid regex");

        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(0) {
                let marker = format!("__PYFMT_{}__", idx);
                replacements.push((idx, m.as_str().to_string()));
                result = result.replacen(m.as_str(), &marker, 1);
                idx += 1;
            }
        }

        (result, replacements)
    }

    /// Restore Python format specifiers
    pub fn restore_python_format(text: &str, replacements: &[(usize, String)]) -> String {
        let mut result = text.to_string();
        for (idx, original) in replacements.iter().rev() {
            let marker = format!("__PYFMT_{}__", idx);
            result = result.replace(&marker, original);
        }
        result
    }

    /// Protect Rust format specifiers
    pub fn protect_rust_format(text: &str) -> (String, Vec<(usize, String)>) {
        let mut replacements = Vec::new();
        let mut result = text.to_string();
        let mut idx = 0;

        // Match {name}, {0}, {name:format}, {:?}, {}
        let re = regex::Regex::new(r"\{(?:[^{}]|\{[^{}]*\})*\}").expect("Invalid regex");

        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(0) {
                let marker = format!("__RUSTFMT_{}__", idx);
                replacements.push((idx, m.as_str().to_string()));
                result = result.replacen(m.as_str(), &marker, 1);
                idx += 1;
            }
        }

        (result, replacements)
    }

    /// Restore Rust format specifiers
    pub fn restore_rust_format(text: &str, replacements: &[(usize, String)]) -> String {
        let mut result = text.to_string();
        for (idx, original) in replacements.iter().rev() {
            let marker = format!("__RUSTFMT_{}__", idx);
            result = result.replace(&marker, original);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::scanner::region::PlaceholderSpan;

    #[test]
    fn test_protect_placeholders() {
        let protector = PlaceholderProtector::new();
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            35,
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 34)
        .with_placeholders(vec![
            PlaceholderSpan::new(7, 14, "${name}".to_string()),
            PlaceholderSpan::new(26, 33, "${value}".to_string()),
        ]);

        let content = "`Hello ${name}, value is ${value}!`";
        let prepared = protector.prepare_for_translation(&region, content);

        assert!(prepared.contains("__PH_0__"));
        assert!(prepared.contains("__PH_1__"));
    }

    #[test]
    fn test_restore_placeholders() {
        let protector = PlaceholderProtector::new();
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            35,
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 34)
        .with_placeholders(vec![
            PlaceholderSpan::new(7, 14, "${name}".to_string()),
            PlaceholderSpan::new(26, 33, "${value}".to_string()),
        ]);

        let translated = "Hello __PH_0__, value is __PH_1__!";
        let restored = protector.restore_placeholders(translated, &region);

        assert!(restored.contains("${name}"));
        assert!(restored.contains("${value}"));
    }

    #[test]
    fn test_printf_format_protection() {
        let text = "错误: %s, 代码: %d";
        let (protected, replacements) = FormatProtector::protect_printf_format(text);

        assert!(protected.contains("__FMT_"));
        assert_eq!(replacements.len(), 2);

        let restored = FormatProtector::restore_printf_format(&protected, &replacements);
        assert_eq!(restored, text);
    }

    #[test]
    fn test_python_format_protection() {
        let text = "错误: {error}, 代码: {code}";
        let (protected, replacements) = FormatProtector::protect_python_format(text);

        assert!(protected.contains("__PYFMT_"));
        assert_eq!(replacements.len(), 2);

        let restored = FormatProtector::restore_python_format(&protected, &replacements);
        assert_eq!(restored, text);
    }

    #[test]
    fn test_rust_format_protection() {
        let text = "错误: {}, 代码: {:?}";
        let (protected, replacements) = FormatProtector::protect_rust_format(text);

        assert!(protected.contains("__RUSTFMT_"));
        assert_eq!(replacements.len(), 2);

        let restored = FormatProtector::restore_rust_format(&protected, &replacements);
        assert_eq!(restored, text);
    }
}
