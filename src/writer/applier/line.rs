//! Single-line translation applier
//!
//! This module handles applying translations to single lines,
//! supporting multiple translation units per line.

use crate::core::models::TranslationUnit;
use crate::writer::format::replacement::{byte_to_char_pos, replace_in_raw_match};

/// Applies translations to a single line
pub struct LineApplier;

impl LineApplier {
    /// Apply translations to a single line
    ///
    /// # Arguments
    /// * `line` - The line content
    /// * `units` - Translation units that apply to this line
    ///
    /// # Returns
    /// Modified line with translations applied
    pub fn apply(line: &str, units: &[&TranslationUnit]) -> String {
        if units.is_empty() {
            return line.to_string();
        }

        let replacements = Self::collect_replacements(line, units);
        if replacements.is_empty() {
            return line.to_string();
        }

        Self::apply_replacements(line, replacements)
    }

    /// Collect replacement operations from translation units
    fn collect_replacements(line: &str, units: &[&TranslationUnit]) -> Vec<Replacement> {
        let mut replacements: Vec<Replacement> = units
            .iter()
            .filter(|unit| unit.should_translate)
            .filter_map(|unit| {
                unit.translated.as_ref().map(|translated| {
                    let formatted_text = if let Some(raw_match) = &unit.raw_match {
                        replace_in_raw_match(raw_match, &unit.content, translated)
                    } else {
                        translated.clone()
                    };

                    let (start_char, end_char) = Self::calculate_char_positions(line, unit);

                    Replacement {
                        start_char,
                        end_char,
                        text: formatted_text,
                    }
                })
            })
            .collect();

        // Sort by start position to ensure correct order
        replacements.sort_by_key(|r| r.start_char);
        replacements
    }

    /// Calculate character positions for a translation unit
    fn calculate_char_positions(line: &str, unit: &TranslationUnit) -> (usize, usize) {
        if let Some(raw_match) = &unit.raw_match {
            if let Some(pos) = line.find(raw_match) {
                // pos is byte position, convert to char position
                let start = byte_to_char_pos(line, pos);
                let end = byte_to_char_pos(line, pos + raw_match.len());
                return (start, end);
            }
        }

        // Fall back to column positions from tree-sitter (byte-based)
        // Convert to char positions
        let start_col = unit.start_pos.column.saturating_sub(1);
        let end_col = unit.end_pos.column.saturating_sub(1);
        (
            byte_to_char_pos(line, start_col),
            byte_to_char_pos(line, end_col),
        )
    }

    /// Apply sorted replacements to a line
    fn apply_replacements(line: &str, replacements: Vec<Replacement>) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut result = String::with_capacity(line.len());
        let mut last_end = 0;

        for repl in replacements {
            let start_char = repl.start_char;
            let end_char = repl.end_char.min(chars.len());

            if start_char >= end_char {
                continue;
            }

            if start_char > last_end {
                result.extend(&chars[last_end..start_char]);
            }
            result.push_str(&repl.text);
            last_end = end_char;
        }

        if last_end < chars.len() {
            result.extend(&chars[last_end..]);
        }

        result
    }
}

/// Represents a text replacement operation
#[derive(Debug)]
struct Replacement {
    start_char: usize,
    end_char: usize,
    text: String,
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
    ) -> TranslationUnit {
        TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: content.to_string(),
            start_pos: Position::new(line, start_col, start_col - 1),
            end_pos: Position::new(line, end_col, end_col - 1),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some(content.to_string()),
        }
    }

    #[test]
    fn test_apply_single_translation() {
        let line = "Hello world";
        let mut unit = create_test_unit("Hello", 1, 1, 6);
        unit.set_translated("你好");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = LineApplier::apply(line, &units);

        assert_eq!(result, "你好 world");
    }

    #[test]
    fn test_apply_no_units() {
        let line = "Hello world";
        let units: Vec<&TranslationUnit> = vec![];
        let result = LineApplier::apply(line, &units);

        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_apply_should_not_translate() {
        let line = "Hello world";
        let mut unit = create_test_unit("Hello", 1, 1, 6);
        unit.set_translated("你好");
        unit.should_translate = false;

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = LineApplier::apply(line, &units);

        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_apply_with_comment_prefix() {
        let line = "    // This is a comment";
        let mut unit = create_test_unit("    // This is a comment", 1, 5, 25);
        unit.content = "This is a comment".to_string();
        unit.set_translated("这是一个注释");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = LineApplier::apply(line, &units);

        assert_eq!(result, "    // 这是一个注释");
    }

    #[test]
    fn test_apply_chinese_to_english_translation() {
        // Simulate E2E test case: Chinese comment translated to English
        let line = "// 这是一个简单的JavaScript文件，用于测试翻译功能";
        let mut unit = create_test_unit(
            "// 这是一个简单的JavaScript文件，用于测试翻译功能",
            1,
            1,
            46,
        );
        unit.content = "这是一个简单的JavaScript文件，用于测试翻译功能".to_string();
        unit.set_translated("This is a simple JavaScript file to test the translation function");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = LineApplier::apply(line, &units);

        // Should replace the content, not append
        assert_eq!(
            result,
            "// This is a simple JavaScript file to test the translation function"
        );
    }
}
