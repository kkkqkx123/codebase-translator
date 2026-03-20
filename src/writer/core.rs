//! Translation application logic
//!
//! This module contains logic for applying translations to file content,
//! with support for different file types (generic and markdown).

use crate::core::error::Result;
use crate::core::models::TranslationUnit;

/// Applies translations to file content
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

        let line_ending = super::file::detect_line_ending(content);
        let normalized_content = content.replace("\r\n", "\n");

        // First, handle multi-line merged units using position-based replacement
        let mut result = normalized_content.clone();
        let multiline_units: Vec<&TranslationUnit> = units
            .iter()
            .filter(|u| {
                // Multi-line units have different start/end lines OR content contains newlines
                u.start_pos.line != u.end_pos.line || u.content.contains('\n')
            })
            .collect();

        // Sort multiline units by position in reverse order to avoid offset issues
        let mut sorted_multiline: Vec<&TranslationUnit> = multiline_units;
        sorted_multiline.sort_by(|a, b| b.start_pos.offset.cmp(&a.start_pos.offset));

        for unit in sorted_multiline {
            if let (Some(raw_match), Some(translated)) = (&unit.raw_match, &unit.translated) {
                // Extract the actual text from original content using byte positions
                // This ensures we get the accurate text including empty lines
                let start_byte = unit.start_pos.offset;
                let end_byte = unit.end_pos.offset;

                // Check if we have valid byte positions (non-zero and within bounds)
                let has_valid_positions =
                    start_byte > 0 && start_byte < end_byte && end_byte <= normalized_content.len();

                if has_valid_positions {
                    // Use position-based extraction for accurate replacement
                    let original_text = &normalized_content[start_byte..end_byte];
                    // Format the translation using raw_match as template for consistent formatting
                    let formatted = Self::format_multiline_translation(raw_match, translated);
                    // Replace using the actual original text from the file
                    result = result.replace(original_text, &formatted);
                } else {
                    // Fall back to raw_match-based replacement for tests or legacy data
                    let formatted = Self::format_multiline_translation(raw_match, translated);
                    result = result.replace(raw_match.as_str(), &formatted);
                }
            }
        }

        // Then handle single-line units line by line
        let lines: Vec<&str> = result.split('\n').collect();
        let mut unit_map: std::collections::HashMap<usize, Vec<&TranslationUnit>> =
            std::collections::HashMap::new();

        for unit in units {
            // Skip multiline units as they've already been handled
            // Use the same logic as multiline_units filter
            if unit.start_pos.line != unit.end_pos.line || unit.content.contains('\n') {
                continue;
            }

            if unit.start_pos.line >= 1 {
                unit_map.entry(unit.start_pos.line).or_default().push(unit);
            }
        }

        let mut builder = String::with_capacity(result.len());
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

    /// Format a multiline translation by applying it line by line to the raw match
    fn format_multiline_translation(raw_match: &str, translated: &str) -> String {
        let raw_lines: Vec<&str> = raw_match.lines().collect();
        let translated_lines: Vec<&str> = translated.lines().collect();

        // Check if raw_match ends with newline - we need to preserve this
        let ends_with_newline = raw_match.ends_with('\n');

        // Check if this is a block comment (starts with /* or /**)
        let is_block_comment = raw_lines
            .first()
            .map(|line| line.trim().starts_with("/*"))
            .unwrap_or(false);

        let mut result = if is_block_comment {
            // For block comments, we need to preserve the structure
            // Extract content lines (lines between /* and */)
            let mut content_lines: Vec<(usize, &str)> = Vec::new();
            for (i, line) in raw_lines.iter().enumerate() {
                let trimmed = line.trim();
                // Skip pure marker lines (just "/*" or "*/")
                if trimmed != "/*" && trimmed != "*/" && !trimmed.ends_with("*/") {
                    content_lines.push((i, *line));
                }
            }

            let mut result = String::new();
            let mut translated_idx = 0;

            for (i, raw_line) in raw_lines.iter().enumerate() {
                let trimmed = raw_line.trim();

                if trimmed == "/*" || trimmed == "/**" {
                    // First line marker
                    result.push_str(raw_line);
                } else if trimmed == "*/" || trimmed.ends_with("*/") {
                    // Last line marker
                    if trimmed == "*/" {
                        // Preserve original whitespace
                        result.push_str(raw_line);
                    } else {
                        // Line like " */" or " * text */"
                        if let Some(pos) = trimmed.find("*/") {
                            let before_marker = trimmed[..pos].trim();
                            if before_marker.is_empty() || before_marker == "*" {
                                result.push_str(" */");
                            } else {
                                result.push_str(" * ");
                                if translated_idx < translated_lines.len() {
                                    result.push_str(translated_lines[translated_idx]);
                                    translated_idx += 1;
                                }
                                result.push_str(" */");
                            }
                        } else {
                            result.push_str(raw_line);
                        }
                    }
                } else {
                    // Content line
                    let prefix = Self::extract_comment_prefix(raw_line);
                    result.push_str(&prefix);
                    if translated_idx < translated_lines.len() {
                        result.push_str(translated_lines[translated_idx]);
                        translated_idx += 1;
                    }
                }

                if i < raw_lines.len() - 1 {
                    result.push('\n');
                }
            }

            result
        } else {
            // Non-block comment: simpler case
            if raw_lines.len() == translated_lines.len() {
                let mut result = String::new();
                for (i, raw_line) in raw_lines.iter().enumerate() {
                    let prefix = Self::extract_comment_prefix(raw_line);
                    result.push_str(&prefix);
                    result.push_str(translated_lines[i]);
                    if i < raw_lines.len() - 1 {
                        result.push('\n');
                    }
                }
                result
            } else {
                let mut result = String::new();
                for (i, raw_line) in raw_lines.iter().enumerate() {
                    let prefix = Self::extract_comment_prefix(raw_line);
                    result.push_str(&prefix);
                    if i < translated_lines.len() {
                        result.push_str(translated_lines[i]);
                    }
                    if i < raw_lines.len() - 1 {
                        result.push('\n');
                    }
                }
                result
            }
        };

        // Preserve trailing newline if raw_match had one
        if ends_with_newline && !result.ends_with('\n') {
            result.push('\n');
        }

        result
    }

    /// Extract comment prefix from a line (e.g., "// ", "/// ", "* ", "/* ")
    fn extract_comment_prefix(line: &str) -> String {
        let trimmed = line.trim_start();
        let leading_whitespace = &line[..(line.len() - trimmed.len())];

        if trimmed.starts_with("///") {
            format!("{}/// ", leading_whitespace)
        } else if trimmed.starts_with("//!") {
            format!("{}//! ", leading_whitespace)
        } else if trimmed.starts_with("//") {
            format!("{}// ", leading_whitespace)
        } else if trimmed.starts_with("/**") {
            format!("{}/** ", leading_whitespace)
        } else if trimmed.starts_with("/*") {
            format!("{}/* ", leading_whitespace)
        } else if trimmed.starts_with('*') {
            format!("{}* ", leading_whitespace)
        } else if trimmed.starts_with('#') {
            format!("{}# ", leading_whitespace)
        } else {
            leading_whitespace.to_string()
        }
    }

    /// Convert byte position to character position in a string
    fn byte_to_char_pos(s: &str, byte_pos: usize) -> usize {
        s.char_indices()
            .take_while(|(pos, _)| *pos < byte_pos)
            .count()
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
                            // pos is byte position, convert to char position
                            let start = Self::byte_to_char_pos(line, pos);
                            let end = Self::byte_to_char_pos(line, pos + raw_match.len());
                            (start, end)
                        } else {
                            // column positions from tree-sitter are byte-based
                            // convert to char positions
                            let start_col = unit.start_pos.column.saturating_sub(1);
                            let end_col = unit.end_pos.column.saturating_sub(1);
                            (
                                Self::byte_to_char_pos(line, start_col),
                                Self::byte_to_char_pos(line, end_col),
                            )
                        }
                    } else {
                        // column positions from tree-sitter are byte-based
                        // convert to char positions
                        let start_col = unit.start_pos.column.saturating_sub(1);
                        let end_col = unit.end_pos.column.saturating_sub(1);
                        (
                            Self::byte_to_char_pos(line, start_col),
                            Self::byte_to_char_pos(line, end_col),
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
        // First try direct match
        if let Some(pos) = raw_match.find(extracted) {
            // pos is byte position, need to convert to char position for slicing
            let start_byte = pos;
            let end_byte = pos + extracted.len();
            let before = &raw_match[..start_byte];
            let after = &raw_match[end_byte..];
            return format!("{}{}{}", before, translated, after);
        }

        // If direct match fails, try line-by-line matching for cleaned comments
        // This handles cases where extracted text has comment markers removed
        let raw_lines: Vec<&str> = raw_match.lines().collect();
        let extracted_lines: Vec<&str> = extracted.lines().collect();

        if raw_lines.len() == extracted_lines.len() {
            let mut result = String::new();
            let mut extracted_idx = 0;

            for (i, raw_line) in raw_lines.iter().enumerate() {
                if extracted_idx < extracted_lines.len() {
                    let extracted_line = extracted_lines[extracted_idx];

                    // Try to find extracted_line in raw_line
                    if let Some(pos) = raw_line.find(extracted_line) {
                        let before = &raw_line[..pos];
                        let after = &raw_line[pos + extracted_line.len()..];

                        // For multi-line translations, split translated text by lines
                        let translated_lines: Vec<&str> = translated.lines().collect();
                        if translated_lines.len() == extracted_lines.len() {
                            result.push_str(&format!("{}{}{}", before, translated_lines[i], after));
                        } else {
                            // If translated text doesn't have same line count, use as-is
                            result.push_str(&format!("{}{}{}", before, translated, after));
                        }

                        extracted_idx += 1;
                    } else {
                        // Can't find extracted line, keep raw line as-is
                        result.push_str(raw_line);
                    }

                    if i < raw_lines.len() - 1 {
                        result.push('\n');
                    }
                } else {
                    result.push_str(raw_line);
                    if i < raw_lines.len() - 1 {
                        result.push('\n');
                    }
                }
            }

            return result;
        }

        // If all else fails, log a warning and return raw match as-is
        tracing::warn!(
            raw_match = %raw_match,
            extracted = %extracted,
            "Extracted text not found in raw match, skipping replacement"
        );
        raw_match.to_string()
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
            content: "This is a\nmulti-line comment".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 37),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/*\n * This is a\n * multi-line comment\n */".to_string()),
        }];

        units[0].set_translated("这是一个\n多行注释");

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
            content: "This is a doc comment".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 37),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/**\n * This is a doc comment\n */".to_string()),
        }];

        units[0].set_translated("这是一个文档注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/**\n * 这是一个文档注释\n */"));
    }

    #[test]
    fn test_format_multiline_translated_text() {
        let content = "/*\n * Line 1\n * Line 2\n */\nint x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Line 1\nLine 2".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 22),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/*\n * Line 1\n * Line 2\n */".to_string()),
        }];

        units[0].set_translated("第一行\n第二行");

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

    #[test]
    fn test_apply_translations_with_chinese_content() {
        // Test case for the bug: byte position vs char position mismatch
        // When raw_match contains Chinese characters, the byte length is different from char length
        let content = "    /// 获取计算器名称\n    pub fn get_name(&self) -> &str {";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "获取计算器名称".to_string(),
            // Byte positions: 4 spaces (4) + /// (3) + space (1) + 获取计算器名称 (24) = 32 bytes
            // Char positions: 4 spaces (4) + /// (3) + space (1) + 获取计算器名称 (8) = 16 chars
            start_pos: Position::new(1, 9, 8), // column is byte-based from tree-sitter
            end_pos: Position::new(1, 33, 32), // column is byte-based from tree-sitter
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("    /// 获取计算器名称".to_string()),
        }];

        units[0].set_translated("Get Calculator Name");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        // The translation should replace the Chinese text correctly
        assert!(result.contains("/// Get Calculator Name"));
        // The next line should remain intact (not merged)
        assert!(result.contains("\n    pub fn get_name(&self) -> &str {"));
    }

    #[test]
    fn test_apply_translations_with_chinese_content_fallback_to_column() {
        // Test case where raw_match is not found in line, fallback to column positions
        // This simulates the actual E2E scenario where raw_match might not match exactly
        let content = "    /// 获取计算器名称\n    pub fn get_name(&self) -> &str {";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "获取计算器名称".to_string(),
            // Simulate tree-sitter byte positions (1-based column)
            // Line: "    /// 获取计算器名称"
            // Bytes: 4 spaces + 3 /// + 1 space + 24 Chinese = 32 bytes
            // Content "获取计算器名称" starts at byte 9 (4+3+1+1, 1-based)
            // Content "获取计算器名称" ends at byte 33 (4+3+1+24+1, 1-based)
            start_pos: Position::new(1, 9, 8), // column 9 = byte 8 (0-based offset)
            end_pos: Position::new(1, 33, 32), // column 33 = byte 32 (0-based offset)
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            // Use different raw_match to force fallback to column positions
            raw_match: Some("/// 获取计算器名称".to_string()),
        }];

        units[0].set_translated("/// Get Calculator Name");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        println!("Result: {:?}", result);
        // The translation should replace the Chinese text correctly
        assert!(
            result.contains("/// Get Calculator Name"),
            "Expected translation not found in: {}",
            result
        );
        // The next line should remain intact (not merged)
        assert!(
            result.contains("\n    pub fn get_name(&self) -> &str {"),
            "Next line was merged incorrectly"
        );
    }

    #[test]
    fn test_chinese_content_line_merging_issue() {
        // Reproduce the exact issue from E2E test:
        // Original: "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {"
        // Expected: "/// multiplication\npub fn multiply(a: i32, b: i32) -> i32 {"
        // Actual (bug): "/// multiplicationpub fn multiply(a: i32, b: i32) -> i32 {"

        let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {\n    a * b\n}";

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            // Line: "/// 乘法运算"
            // Bytes: /// (3) + space (1) + 乘法运算 (12) = 16 bytes
            // Content starts at column 5 (after "/// ")
            // Content ends at column 17 (after content)
            start_pos: Position::new(1, 5, 4),
            end_pos: Position::new(1, 17, 16),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算".to_string()),
        }];

        units[0].set_translated("multiplication");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        println!("Original:\n{}", content);
        println!("\nResult:\n{}", result);

        // The key assertion: translation should be on its own line
        assert!(
            result.contains("/// multiplication\n"),
            "Translation should end with newline. Result: {:?}",
            result
        );
        assert!(
            result.contains("pub fn multiply(a: i32, b: i32) -> i32 {"),
            "Function should be on separate line. Result: {:?}",
            result
        );

        // Make sure they are NOT merged
        assert!(
            !result.contains("multiplicationpub fn"),
            "Lines should not be merged! Result: {:?}",
            result
        );
    }

    #[test]
    fn test_chinese_content_with_actual_parser_positions() {
        // This test simulates the actual positions returned by tree-sitter parser
        // The key difference is that tree-sitter returns byte positions for columns
        // For "/// 乘法运算":
        // - /// is at bytes 0-2
        // - space is at byte 3
        // - 乘法运算 is at bytes 4-15 (4 chars * 3 bytes each)
        // Total: 16 bytes, 8 chars
        //
        // If tree-sitter returns start_pos.column = 1 (start of line),
        // and end_pos.column = 17 (1-based byte position after the line),
        // then the replacement range would be the entire line.

        let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";

        // Simulate what tree-sitter might return
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            // Tree-sitter might return positions for the entire line including prefix
            // If start_pos.column = 1 (start of line, 1-based byte position)
            // and end_pos.column = 17 (end of line + 1, 1-based byte position)
            start_pos: Position::new(1, 1, 0), // column 1 = byte 0
            end_pos: Position::new(1, 17, 16), // column 17 = byte 16
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算".to_string()),
        }];

        units[0].set_translated("/// multiplication");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        println!("Test with actual parser positions:");
        println!("Original:\n{}", content);
        println!("\nResult:\n{}", result);

        // Check the result
        assert!(
            result.contains("/// multiplication\n"),
            "Translation should have newline. Result: {:?}",
            result
        );
        assert!(
            !result.contains("multiplicationpub fn"),
            "Lines should not be merged! Result: {:?}",
            result
        );
    }

    #[test]
    fn test_raw_match_with_newline() {
        // This test simulates the case where raw_match contains a newline
        // but end_pos doesn't include it (due to query_executor adjustment)
        // This can happen when tree-sitter returns a node that includes newline
        // but end_pos is adjusted to exclude it

        let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";

        // Simulate the problematic case:
        // - raw_match includes newline: "/// 乘法运算\n"
        // - but end_pos points to end of line without newline
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            // Positions for the line without newline
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 17, 16), // End of "/// 乘法运算" without newline
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            // raw_match includes newline (this is the bug!)
            raw_match: Some("/// 乘法运算\n".to_string()),
        }];

        units[0].set_translated("multiplication");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        println!("Test with raw_match containing newline:");
        println!("Original:\n{}", content);
        println!("\nResult:\n{}", result);

        // The result should NOT merge lines
        assert!(
            !result.contains("multiplicationpub fn"),
            "Lines should not be merged! Result: {:?}",
            result
        );
    }

    #[test]
    fn test_translated_with_trailing_newline() {
        // This test checks if the bug is caused by translated text containing trailing newline
        let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            start_pos: Position::new(1, 5, 4),
            end_pos: Position::new(1, 17, 16),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算".to_string()),
        }];

        units[0].set_translated("multiplication\n");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        println!("Test with translated containing trailing newline:");
        println!("Original:\n{}", content);
        println!("\nResult:\n{}", result);

        if result.contains("multiplicationpub fn") {
            panic!("Lines merged! Result: {:?}", result);
        }
    }

    #[test]
    fn test_raw_match_with_crlf() {
        let content = "/// 乘法运算\r\npub fn multiply(a: i32, b: i32) -> i32 {";

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            start_pos: Position::new(1, 5, 4),
            end_pos: Position::new(1, 18, 17),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算\r".to_string()),
        }];

        units[0].set_translated("multiplication");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        if result.contains("multiplicationpub fn") {
            panic!("Lines merged! Result: {:?}", result);
        }
    }

    #[test]
    fn test_content_with_newline() {
        let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算\npub fn multiply".to_string(),
            start_pos: Position::new(1, 5, 4),
            end_pos: Position::new(2, 40, 60),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算\npub fn multiply".to_string()),
        }];

        units[0].set_translated("multiplication\npub fn multiply");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        println!("Test with content containing newline:");
        println!("Original:\n{}", content);
        println!("\nResult:\n{}", result);

        if result.contains("multiplicationpub fn") {
            println!("\n*** FOUND THE BUG! ***");
        }
    }

    #[test]
    fn test_translated_with_extra_content() {
        // This test verifies that writer correctly applies translation even when
        // translated content differs from original line count.
        // Note: It's the translator's responsibility to return correct content.
        // Writer's job is to faithfully apply the translation.

        let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";

        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            start_pos: Position::new(1, 5, 4),
            end_pos: Position::new(1, 17, 16),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算".to_string()),
        }];

        // Test with correct translation
        units[0].set_translated("multiplication");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();

        println!("Test with correct translation:");
        println!("Original:\n{}", content);
        println!("\nResult:\n{}", result);

        // Verify correct translation is applied
        assert!(
            result.contains("/// multiplication"),
            "Translation not applied correctly: {}",
            result
        );
        assert!(
            !result.contains("乘法运算"),
            "Original content still present: {}",
            result
        );
    }
}
