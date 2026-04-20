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

    /// Restore placeholders in translated text (with fault tolerance)
    pub fn restore_placeholders(&self, translated: &str, region: &TextRegion) -> String {
        if region.placeholders.is_empty() {
            return translated.to_string();
        }

        let mut result = translated.to_string();

        for (idx, placeholder) in region.placeholders.iter().enumerate() {
            // Try standard marker first
            let standard_marker = self.make_marker(idx);
            if result.contains(&standard_marker) {
                result = result.replace(&standard_marker, &placeholder.original);
                continue;
            }

            // Try fault-tolerant variants with word boundary checking
            let variants = self.generate_marker_variants(idx);
            for variant in variants {
                // Simple string replacement with boundary check
                // Check if variant exists and is surrounded by whitespace/punctuation
                if result.contains(&variant) {
                    // Replace with boundary checking
                    let replaced = result.replace(
                        &format!(" {} ", variant),
                        &format!(" {} ", placeholder.original),
                    );
                    if replaced != result {
                        result = replaced;
                        break;
                    }

                    // Try without surrounding spaces
                    let replaced = result.replace(&variant, &placeholder.original);
                    if replaced != result {
                        result = replaced;
                        break;
                    }
                }
            }
        }

        result
    }

    /// Generate marker variants for fault-tolerant matching
    fn generate_marker_variants(&self, index: usize) -> Vec<String> {
        let base = format!("{}", index);

        // Generate variants based on common corruption patterns
        // marker_prefix = "__PH_", marker_suffix = "__"
        // Missing trailing underscore: __PH_0 instead of __PH_0__
        let missing_trailing = format!("{}{}", self.marker_prefix, base);
        // Missing leading underscore: PH_0__ instead of __PH_0__
        let missing_leading = format!("{}{}", base, self.marker_suffix);
        // Missing underscore before number: __PH0__ instead of __PH_0__
        let prefix_without_underscore = &self.marker_prefix[..self.marker_prefix.len() - 1];
        let missing_underscore_before_num = format!(
            "{}{}{}",
            prefix_without_underscore, base, self.marker_suffix
        );
        // Missing both leading and trailing underscore: PH_0 instead of __PH_0__
        let missing_both = format!(
            "{}{}{}",
            &self.marker_prefix[..self.marker_prefix.len().saturating_sub(1)],
            base,
            &self.marker_suffix[1..]
        );

        vec![
            missing_trailing,
            missing_leading,
            missing_underscore_before_num,
            missing_both,
        ]
    }

    /// Make a marker for the given index
    fn make_marker(&self, index: usize) -> String {
        format!("{}{}{}", self.marker_prefix, index, self.marker_suffix)
    }

    /// Check if text contains any placeholder markers
    pub fn contains_markers(&self, text: &str) -> bool {
        text.contains(&self.marker_prefix)
    }

    /// Validate that all placeholder markers are intact after translation
    /// Returns true if all markers are present and correctly formatted
    pub fn validate_placeholders(&self, original: &str, translated: &str) -> (bool, Vec<String>) {
        let mut issues = Vec::new();

        // Extract markers from original text
        let original_markers = self.extract_markers(original);
        let translated_markers = self.extract_markers(translated);

        // Check if all original markers are present in translated text
        for (start, end, idx) in &original_markers {
            let marker = self.make_marker(*idx);
            if !translated.contains(&marker) {
                issues.push(format!(
                    "Placeholder marker '{}' missing in translation (found at {}-{} in original)",
                    marker, start, end
                ));
            }
        }

        // Check for extra markers in translated text (should not happen)
        for (start, end, idx) in &translated_markers {
            let marker = self.make_marker(*idx);
            if !original.contains(&marker) {
                issues.push(format!(
                    "Unexpected placeholder marker '{}' in translation (found at {}-{})",
                    marker, start, end
                ));
            }
        }

        // Check for malformed markers (e.g., __PH_0 instead of __PH_0__)
        let malformed_pattern = format!(r"{}(\d+)(?![{}])", self.marker_prefix, self.marker_suffix);
        if let Ok(re) = regex::Regex::new(&malformed_pattern) {
            for cap in re.captures_iter(translated) {
                if let Some(malformed) = cap.get(0) {
                    issues.push(format!(
                        "Malformed placeholder marker '{}' detected (missing suffix)",
                        malformed.as_str()
                    ));
                }
            }
        }

        (issues.is_empty(), issues)
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
        assert!(
            !prepared.contains("${name}"),
            "Placeholder should be replaced with marker"
        );
        assert!(
            !prepared.contains("${value}"),
            "Placeholder should be replaced with marker"
        );
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
        assert_eq!(
            restored,
            "你好 __PH_0__，值是 __PH_1__!".replace(
                "你好 __PH_0__，值是 __PH_1__!",
                "Hello ${name}, value is ${value}!"
            )
        );
    }

    #[test]
    fn test_placeholder_boundary_preservation() {
        let protector = PlaceholderProtector::new();
        // Content: "Hello ${username}!" (length 18)
        //          012345678901234567
        // ${username} = $(6){(7)u(8)s(9)e(10)r(11)n(12)a(13)m(14)e(15)}(16)
        // starts at 6, ends at 17 (exclusive, includes })
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            20, // `Hello ${username}!` has length 20
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 19) // Content is positions 1-18
        .with_placeholders(vec![PlaceholderSpan::new(6, 17, "${username}".to_string())]);

        let content = "`Hello ${username}!`";
        let prepared = protector.prepare_for_translation(&region, content);

        assert_eq!(prepared, "Hello __PH_0__!");
        assert!(
            !prepared.contains("${"),
            "Placeholder boundary ${{ should be protected"
        );
        assert!(
            !prepared.contains('}'),
            "Placeholder boundary }} should be protected"
        );

        let translated = "你好 __PH_0__!";
        let restored = protector.restore_placeholders(translated, &region);

        assert_eq!(restored, "你好 ${username}!");
        assert!(
            restored.contains("${username}"),
            "Complete placeholder should be restored"
        );
        assert!(
            restored.contains("${"),
            "Placeholder start boundary should be preserved"
        );
        assert!(
            restored.contains("}"),
            "Placeholder end boundary should be preserved"
        );
    }

    #[test]
    fn test_placeholder_boundary_breakage_simulation() {
        let protector = PlaceholderProtector::new();
        // Content: "String ${value} here" (length 20)
        //          01234567890123456789
        // ${value} starts at 7, ends at 15 (exclusive, includes })
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            22, // `String ${value} here` has length 22
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 21) // Content is positions 1-20
        .with_placeholders(vec![PlaceholderSpan::new(7, 15, "${value}".to_string())]);

        let content = "`String ${value} here`";
        let prepared = protector.prepare_for_translation(&region, content);

        assert_eq!(prepared, "String __PH_0__ here");

        // Simulate translation that preserves the marker
        let translated = "字符串 __PH_0__ 这里";
        let restored = protector.restore_placeholders(translated, &region);

        assert_eq!(restored, "字符串 ${value} 这里");
        assert!(
            restored.contains("${value}"),
            "Complete placeholder should be restored"
        );
        assert!(
            restored.contains("${"),
            "Placeholder start boundary should be preserved"
        );
        assert!(
            restored.contains('}'),
            "Placeholder end boundary should be preserved"
        );
    }

    #[test]
    fn test_fault_tolerant_restore_missing_trailing_underscore() {
        let protector = PlaceholderProtector::new();
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            22,
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 21)
        .with_placeholders(vec![PlaceholderSpan::new(7, 15, "${value}".to_string())]);

        // Simulate translation that lost trailing underscore: __PH_0 instead of __PH_0__
        let translated_with_error = "字符串 __PH_0 这里";
        let restored = protector.restore_placeholders(translated_with_error, &region);

        // The fault-tolerant mechanism should detect and restore the placeholder
        assert!(
            restored.contains("${value}"),
            "Should recover placeholder from malformed marker"
        );
    }

    #[test]
    fn test_fault_tolerant_restore_missing_leading_underscore() {
        let protector = PlaceholderProtector::new();
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            22,
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 21)
        .with_placeholders(vec![PlaceholderSpan::new(7, 15, "${value}".to_string())]);

        // Simulate translation that lost leading underscore: PH_0__ instead of __PH_0__
        let translated_with_error = "字符串 PH_0__ 这里";
        let restored = protector.restore_placeholders(translated_with_error, &region);

        // The fault-tolerant mechanism should detect and restore the placeholder
        assert!(
            restored.contains("${value}"),
            "Should recover placeholder from malformed marker"
        );
    }

    #[test]
    fn test_fault_tolerant_restore_missing_underscore_before_number() {
        let protector = PlaceholderProtector::new();
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            22,
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 21)
        .with_placeholders(vec![PlaceholderSpan::new(7, 15, "${value}".to_string())]);

        // Simulate translation that lost underscore before number: __PH0__ instead of __PH_0__
        let translated_with_error = "字符串 __PH0__ 这里";
        let restored = protector.restore_placeholders(translated_with_error, &region);

        // The fault-tolerant mechanism should detect and restore the placeholder
        assert!(
            restored.contains("${value}"),
            "Should recover placeholder from malformed marker"
        );
    }

    #[test]
    fn test_validate_placeholders_valid() {
        let protector = PlaceholderProtector::new();

        let original = "Error: __PH_0__, code: __PH_1__";
        let translated = "错误：__PH_0__，代码：__PH_1__";

        let (valid, issues) = protector.validate_placeholders(original, translated);

        assert!(valid, "Valid translation should pass validation");
        assert!(
            issues.is_empty(),
            "No issues should be reported for valid translation"
        );
    }

    #[test]
    fn test_validate_placeholders_missing_marker() {
        let protector = PlaceholderProtector::new();

        let original = "Error: __PH_0__, code: __PH_1__";
        let translated = "错误：__PH_0__，代码"; // Missing __PH_1__

        let (valid, issues) = protector.validate_placeholders(original, translated);

        assert!(
            !valid,
            "Translation with missing marker should fail validation"
        );
        assert!(!issues.is_empty(), "Should report missing marker issue");
        assert!(
            issues.iter().any(|i| i.contains("__PH_1__")),
            "Should report __PH_1__ as missing"
        );
    }

    #[test]
    fn test_validate_placeholders_malformed_marker() {
        let protector = PlaceholderProtector::new();

        let original = "Error: __PH_0__";
        let translated = "错误：__PH_0"; // Malformed: missing trailing __

        let (valid, issues) = protector.validate_placeholders(original, translated);

        assert!(
            !valid,
            "Translation with malformed marker should fail validation"
        );
        assert!(!issues.is_empty(), "Should report malformed marker issue");
    }

    #[test]
    fn test_multiple_placeholders_with_boundaries() {
        let protector = PlaceholderProtector::new();
        // `${first} ${last}` has length 18 (positions 0-17)
        // Content: "${first} ${last}" (length 16, positions 1-16)
        // ${first} = $(0){(1)f(2)i(3)r(4)s(5)t(6)}(7) (8)
        // ${last}  = $(9){(10)l(11)a(12)s(13)t(14)}(15)
        // ${first} starts at 0, ends at 8 (exclusive)
        // ${last} starts at 9, ends at 16 (exclusive)
        let region = TextRegion::new(
            crate::parser::scanner::region::TextRegionType::TemplateString,
            0,
            18, // `${first} ${last}` has length 18
        )
        .with_prefix("`")
        .with_suffix("`")
        .with_content_range(1, 17) // Content is positions 1-16
        .with_placeholders(vec![
            PlaceholderSpan::new(0, 8, "${first}".to_string()),
            PlaceholderSpan::new(9, 16, "${last}".to_string()),
        ]);

        let content = "`${first} ${last}`";
        let prepared = protector.prepare_for_translation(&region, content);

        assert_eq!(prepared, "__PH_0__ __PH_1__");

        let translated = "__PH_0__ __PH_1__";
        let restored = protector.restore_placeholders(translated, &region);

        assert_eq!(restored, "${first} ${last}");
        assert!(restored.contains("${first}"));
        assert!(restored.contains("${last}"));
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
