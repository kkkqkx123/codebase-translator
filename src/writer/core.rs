//! Translation application logic
//!
//! This module contains high-level coordination logic for applying translations
//! to file content. The actual implementation details are delegated to:
//! - `applier::LineApplier` for single-line translations
//! - `applier::MultilineApplier` for multi-line and block comment translations
//! - `format` module for text formatting utilities

use crate::core::error::Result;
use crate::core::models::TranslationUnit;
use crate::writer::applier::{LineApplier, MultilineApplier};
use crate::writer::file::{detect_line_ending, normalize_line_ending};

/// Applies translations to file content
///
/// This is a high-level coordinator that delegates to specialized appliers:
/// - Multi-line units are processed first (in reverse order to avoid offset issues)
/// - Single-line units are then processed line by line
pub struct TranslationApplier;

impl TranslationApplier {
    /// Apply translations to content
    ///
    /// This method handles both single-line units and multi-line merged units.
    /// For multi-line units, it uses position information to extract and replace content.
    ///
    /// # Arguments
    /// * `content` - Original file content
    /// * `units` - Translation units with translated content
    ///
    /// # Returns
    /// Modified content with translations applied
    pub fn apply_translations(content: &str, units: &[TranslationUnit]) -> Result<String> {
        if units.is_empty() {
            return Ok(content.to_string());
        }

        let line_ending = detect_line_ending(content);
        let normalized_content = content.replace("\r\n", "\n");

        // Step 1: Handle multi-line merged units
        let multiline_units: Vec<&TranslationUnit> = units
            .iter()
            .filter(|u| {
                // Multi-line units have different start/end lines OR content contains newlines
                u.start_pos.line != u.end_pos.line || u.content.contains('\n')
            })
            .collect();

        let result = if multiline_units.is_empty() {
            normalized_content
        } else {
            MultilineApplier::apply(&normalized_content, &multiline_units)
        };

        // Step 2: Handle single-line units line by line
        let final_result = if has_single_line_units(units, &multiline_units) {
            Self::apply_single_line_units(&result, units, &multiline_units)
        } else {
            result
        };

        // Step 3: Restore original line endings
        let final_result = normalize_line_ending(&final_result, line_ending);

        Ok(final_result)
    }

    /// Apply single-line translation units to content
    fn apply_single_line_units(
        content: &str,
        all_units: &[TranslationUnit],
        multiline_units: &[&TranslationUnit],
    ) -> String {
        // Build a map of line number to units
        let mut unit_map: std::collections::HashMap<usize, Vec<&TranslationUnit>> =
            std::collections::HashMap::new();

        for unit in all_units {
            // Skip multiline units as they've already been handled
            if multiline_units.iter().any(|u| std::ptr::eq(*u, unit)) {
                continue;
            }

            if unit.start_pos.line >= 1 {
                unit_map.entry(unit.start_pos.line).or_default().push(unit);
            }
        }

        if unit_map.is_empty() {
            return content.to_string();
        }

        // Process each line
        let lines: Vec<&str> = content.split('\n').collect();
        let mut builder = String::with_capacity(content.len());

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            if let Some(line_units) = unit_map.get(&line_num) {
                builder.push_str(&LineApplier::apply(line, line_units));
            } else {
                builder.push_str(line);
            }

            if line_idx < lines.len() - 1 {
                builder.push('\n');
            }
        }

        builder
    }
}

/// Check if there are any single-line units to process
fn has_single_line_units(
    all_units: &[TranslationUnit],
    multiline_units: &[&TranslationUnit],
) -> bool {
    all_units.len() > multiline_units.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{NodeType, Position};

    fn create_test_unit(
        content: &str,
        line: usize,
        start_col: usize,
        end_col: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> TranslationUnit {
        TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: content.to_string(),
            start_pos: Position::new(line, start_col, start_offset),
            end_pos: Position::new(line, end_col, end_offset),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some(content.to_string()),
        }
    }

    #[test]
    fn test_apply_translations() {
        let content = "Hello world\nThis is a test";
        let mut unit = create_test_unit("Hello", 1, 1, 6, 0, 5);
        unit.set_translated("你好");

        let units = vec![unit];
        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        assert!(result.contains("你好"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_apply_translations_with_crlf() {
        let content = "Hello world\r\nThis is a test";
        let mut unit = create_test_unit("Hello", 1, 1, 6, 0, 5);
        unit.set_translated("你好");

        let units = vec![unit];
        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        assert!(result.contains("你好"));
        assert!(result.contains("\r\n"));
    }

    #[test]
    fn test_format_line_comment() {
        let content = "    // This is a comment\nint x = 5;";
        let mut unit = create_test_unit("    // This is a comment", 1, 5, 25, 4, 24);
        unit.content = "This is a comment".to_string();
        unit.set_translated("这是一个注释");

        let units = vec![unit];
        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        assert!(result.contains("    // 这是一个注释"));
    }

    #[test]
    fn test_empty_units() {
        let content = "Hello world";
        let units: Vec<TranslationUnit> = vec![];

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn test_multiline_doc_comment() {
        let content = "/// Line 1\n/// Line 2\npub fn test() {}";
        let mut unit = create_test_unit("/// Line 1\n/// Line 2", 1, 1, 10, 0, 20);
        unit.content = "Line 1\nLine 2".to_string();
        unit.end_pos.line = 2;
        unit.set_translated("第一行\n第二行");

        let units = vec![unit];
        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        assert!(result.contains("/// 第一行"));
        assert!(result.contains("/// 第二行"));
    }
}
