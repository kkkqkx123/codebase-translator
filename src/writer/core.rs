//! Translation application logic
//!
//! This module contains logic for applying translations to file content,
//! with support for different file types (generic and markdown).

use crate::core::error::{Result, TranslateError};
use crate::core::models::TranslationUnit;

/// Applies translations to file content
pub struct TranslationApplier;

impl TranslationApplier {
    /// Apply translations to content
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

        let line_ending = super::file::detect_line_ending(content);
        let normalized_content = content.replace("\r\n", "\n");
        let lines: Vec<&str> = normalized_content.split('\n').collect();

        let mut unit_map: std::collections::HashMap<usize, Vec<&TranslationUnit>> =
            std::collections::HashMap::new();

        for unit in units {
            if unit.start_pos.line >= 1 {
                unit_map.entry(unit.start_pos.line).or_default().push(unit);
            }
        }

        let mut builder = String::with_capacity(content.len());
        let mut line_idx = 0;

        while line_idx < lines.len() {
            let line_num = line_idx + 1;

            if let Some(line_units) = unit_map.get(&line_num) {
                builder.push_str(&Self::apply_translations_to_line(
                    lines[line_idx],
                    line_units,
                ));
            } else {
                builder.push_str(lines[line_idx]);
            }

            if line_idx < lines.len() - 1 {
                builder.push_str(line_ending);
            }

            line_idx += 1;
        }

        Ok(builder)
    }

    /// Apply translations to a single line
    fn apply_translations_to_line(line: &str, units: &[&TranslationUnit]) -> String {
        if units.is_empty() {
            return line.to_string();
        }

        #[derive(Debug)]
        struct Replacement {
            start_char: usize,
            end_char: usize,
            text: String,
        }

        let mut replacements: Vec<Replacement> = units
            .iter()
            .filter(|unit| unit.should_translate)
            .filter_map(|unit| {
                unit.translated.as_ref().map(|translated| {
                    let formatted_text = if let Some(raw_match) = &unit.raw_match {
                        Self::replace_in_raw_match(raw_match, &unit.content, translated)
                    } else {
                        translated.clone()
                    };

                    let (start_char, end_char) = if let Some(raw_match) = &unit.raw_match {
                        if let Some(pos) = line.find(raw_match) {
                            (pos, pos + raw_match.len())
                        } else {
                            (
                                unit.start_pos.column.saturating_sub(1),
                                unit.end_pos.column.saturating_sub(1),
                            )
                        }
                    } else {
                        (
                            unit.start_pos.column.saturating_sub(1),
                            unit.end_pos.column.saturating_sub(1),
                        )
                    };

                    Replacement {
                        start_char,
                        end_char,
                        text: formatted_text,
                    }
                })
            })
            .collect();

        if replacements.is_empty() {
            return line.to_string();
        }

        replacements.sort_by_key(|r| r.start_char);

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

    /// Replace extracted text within raw match with translated text
    ///
    /// This is used for regex/state machine extraction where we need to preserve
    /// the original format (prefixes, function calls, etc.) and only replace
    /// the extracted content.
    ///
    /// # Arguments
    /// * `raw_match` - The complete matched text including format markers
    /// * `extracted` - The extracted text that was translated
    /// * `translated` - The translated text to insert
    ///
    /// # Returns
    /// The raw match with the extracted text replaced by translated text
    fn replace_in_raw_match(raw_match: &str, extracted: &str, translated: &str) -> String {
        // Find the position of extracted text in raw match
        // Use the first occurrence if there are multiple
        if let Some(pos) = raw_match.find(extracted) {
            let start = pos;
            let end = start + extracted.len();

            // Build the result by combining:
            // 1. The part before the extracted text
            // 2. The translated text
            // 3. The part after the extracted text
            let before = &raw_match[..start];
            let after = &raw_match[end..];

            format!("{}{}{}", before, translated, after)
        } else {
            // If extracted text is not found in raw match, log a warning
            // and return the raw match as-is (don't break the file)
            tracing::warn!(
                raw_match = %raw_match,
                extracted = %extracted,
                "Extracted text not found in raw match, skipping replacement"
            );
            raw_match.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{NodeType, Position};

    #[test]
    fn test_apply_translations() {
        let content = "Hello world\nThis is a test";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("Hello".to_string()),
        }];

        units[0].set_translated("你好");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("你好"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_apply_translations_with_crlf() {
        let content = "Hello world\r\nThis is a test";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("Hello".to_string()),
        }];

        units[0].set_translated("你好");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("你好"));
        assert!(result.contains("\r\n"));
    }

    #[test]
    fn test_format_line_comment() {
        let content = "    // This is a comment\nint x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "This is a comment".to_string(),
            start_pos: Position::new(1, 5, 4),
            end_pos: Position::new(1, 22, 21),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("    // This is a comment".to_string()),
        }];

        units[0].set_translated("这是一个注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("    // 这是一个注释"));
    }

    #[test]
    fn test_format_single_line_block_comment() {
        let content = "/* This is a comment */\nint x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "/* This is a comment */".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 22, 21),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/* This is a comment */".to_string()),
        }];

        units[0].set_translated("/* 这是一个注释 */");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/* 这是一个注释 */"));
    }

    #[test]
    fn test_format_multiline_block_comment() {
        let content = "/*\n * This is a\n * multi-line comment\n */\nint x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "/*\n * This is a\n * multi-line comment\n */".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 37),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/*\n * This is a\n * multi-line comment\n */".to_string()),
        }];

        units[0].set_translated("/*\n * 这是一个\n * 多行注释\n */");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/*\n * 这是一个\n * 多行注释\n */"));
    }

    #[test]
    fn test_format_multiline_block_comment_with_indent() {
        let content =
            "    /*\n     * This is a\n     * multi-line comment\n     */\n    int x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "This is a".to_string(),
            start_pos: Position::new(2, 9, 0),
            end_pos: Position::new(2, 17, 0),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("This is a".to_string()),
        }];

        units[0].set_translated("这是一个");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("    /*\n     * 这是一个\n     * multi-line comment\n     */"));
    }

    #[test]
    fn test_format_doc_outer_comment() {
        let content = "/// This is a doc comment\npub fn foo() {}";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "/// This is a doc comment".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 24, 23),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// This is a doc comment".to_string()),
        }];

        units[0].set_translated("/// 这是一个文档注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/// 这是一个文档注释"));
    }

    #[test]
    fn test_format_doc_block_comment() {
        let content = "/**\n * This is a doc comment\n */\npub fn foo() {}";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "/**\n * This is a doc comment\n */".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 37),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/**\n * This is a doc comment\n */".to_string()),
        }];

        units[0].set_translated("/**\n * 这是一个\n * 文档注释\n */");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/**\n * 这是一个\n * 文档注释\n */"));
    }

    #[test]
    fn test_format_multiline_translated_text() {
        let content = "/*\n * Line 1\n * Line 2\n */\nint x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "/*\n * Line 1\n * Line 2\n */".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 22),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/*\n * Line 1\n * Line 2\n */".to_string()),
        }];

        units[0].set_translated("/*\n * 第一行\n * 第二行\n */");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/*\n * 第一行\n * 第二行\n */"));
    }

    #[test]
    fn test_format_with_raw_match() {
        let content = "Hello world\nThis is a test";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("Hello".to_string()),
        }];

        units[0].set_translated("你好");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("你好"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_multiple_translations_with_formats() {
        let content = "    // First comment\n    // Second comment\nint x = 5;";
        let mut units = vec![
            TranslationUnit {
                id: "1".to_string(),
                node_type: NodeType::Comment,
                content: "// First comment".to_string(),
                start_pos: Position::new(1, 5, 4),
                end_pos: Position::new(1, 21, 20),
                language: None,
                should_translate: true,
                translated: None,
                pattern_type: None,
                pattern_name: None,
                raw_match: Some("    // First comment".to_string()),
            },
            TranslationUnit {
                id: "2".to_string(),
                node_type: NodeType::Comment,
                content: "// Second comment".to_string(),
                start_pos: Position::new(2, 5, 25),
                end_pos: Position::new(2, 22, 42),
                language: None,
                should_translate: true,
                translated: None,
                pattern_type: None,
                pattern_name: None,
                raw_match: Some("    // Second comment".to_string()),
            },
        ];

        units[0].set_translated("// 第一个注释");
        units[1].set_translated("// 第二个注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("// 第一个注释"));
        assert!(result.contains("// 第二个注释"));
    }
}
