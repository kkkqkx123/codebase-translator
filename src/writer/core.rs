//! Translation application logic
//!
//! This module contains logic for applying translations to file content,
//! with support for different file types (generic and markdown).

use crate::core::error::{Result, TranslateError};
use crate::core::models::{CommentStyle, FormatInfo, TranslationUnit};

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

        Self::validate_translations(units)?;

        let line_ending = super::file::detect_line_ending(content);
        let normalized_content = content.replace("\r\n", "\n");
        let lines: Vec<&str> = normalized_content.split('\n').collect();

        // Separate multiline and single-line units
        let mut multiline_units: Vec<&TranslationUnit> = Vec::new();
        let mut unit_map: std::collections::HashMap<usize, Vec<&TranslationUnit>> =
            std::collections::HashMap::new();

        for unit in units {
            if unit.start_pos.line >= 1 {
                // Check if this is a multiline unit:
                // 1. If format_info is set, use its is_multiline flag
                // 2. Otherwise, check if start and end positions are on different lines
                let is_multiline = unit
                    .format_info
                    .as_ref()
                    .map(|f| f.is_multiline)
                    .unwrap_or_else(|| unit.start_pos.line != unit.end_pos.line);

                if is_multiline {
                    multiline_units.push(unit);
                } else {
                    unit_map.entry(unit.start_pos.line).or_default().push(unit);
                }
            }
        }

        let mut builder = String::with_capacity(content.len());
        let mut line_idx = 0;

        while line_idx < lines.len() {
            let line_num = line_idx + 1;

            // Check if this line is part of a multiline comment
            let multiline_unit = multiline_units.iter().find(|u| {
                if line_num < u.start_pos.line {
                    return false;
                }
                // Use end_pos.line to determine the span
                // This works for both with and without format_info
                line_num <= u.end_pos.line
            });

            if let Some(unit) = multiline_unit {
                // This line is part of a multiline comment
                if line_num == unit.start_pos.line {
                    // First line of multiline comment - apply the full replacement
                    if let Some(translated) = &unit.translated {
                        let formatted = if let Some(format) = &unit.format_info {
                            Self::format_translated_text(translated, format)
                        } else {
                            translated.clone()
                        };

                        // Calculate how many lines this multiline comment spans
                        let line_count = (unit.end_pos.line - unit.start_pos.line + 1) as usize;

                        // For multiline comments, the formatted text already includes all necessary prefixes
                        // So we should replace the entire comment content, not preserve any prefix
                        // The formatted text includes the full comment with all prefixes
                        builder.push_str(&formatted);

                        // Add line ending after the multiline comment (only if not the last line of file)
                        if line_idx + line_count < lines.len() {
                            builder.push_str(line_ending);
                        }

                        // Skip the remaining lines of this multiline comment
                        line_idx += line_count;
                        continue;
                    }
                } else {
                    // Skip intermediate lines of multiline comment
                    line_idx += 1;
                    continue;
                }
            }

            // Regular single-line processing
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

    /// Validate that all translatable units have been translated
    fn validate_translations(units: &[TranslationUnit]) -> Result<()> {
        let untranslated: Vec<&str> = units
            .iter()
            .filter(|u| u.should_translate && u.translated.is_none() && u.format_info.is_some())
            .map(|u| u.id.as_str())
            .collect();

        if !untranslated.is_empty() {
            return Err(TranslateError::Translation(format!(
                "Missing translations for units: {}",
                untranslated.join(", ")
            )));
        }

        Ok(())
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
                    // Format the translated text if format_info is available
                    let formatted_text = if let Some(format) = &unit.format_info {
                        Self::format_translated_text(translated, format)
                    } else {
                        translated.clone()
                    };

                    // For comments with line_prefix, we need to preserve the prefix
                    // The start_pos.column points to the start of the content (after prefix)
                    // So we should replace from start_pos.column - 1 to end_pos.column - 1
                    // But we also need to preserve the prefix itself
                    let (start_char, end_char) = if let Some(format) = &unit.format_info {
                        if format.line_prefix.is_some() {
                            // Preserve the prefix by replacing only the content
                            let prefix_len = format.line_prefix.as_ref().map(|p| p.len()).unwrap_or(0);
                            let base_indent_len = format.base_indent.len();
                            (
                                unit.start_pos.column.saturating_sub(1),
                                unit.end_pos.column.saturating_sub(1),
                            )
                        } else {
                            // No prefix, replace the entire range
                            (
                                unit.start_pos.column.saturating_sub(1),
                                unit.end_pos.column.saturating_sub(1),
                            )
                        }
                    } else {
                        // No format info, replace the entire range
                        (
                            unit.start_pos.column.saturating_sub(1),
                            unit.end_pos.column.saturating_sub(1),
                        )
                    };

                    let replacement = Replacement {
                        start_char,
                        end_char,
                        text: formatted_text,
                    };

                    replacement
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

    /// Format translated text according to the original format
    fn format_translated_text(translated: &str, format: &FormatInfo) -> String {
        // Check if this is a multiline comment that needs special handling
        if format.is_multiline {
            return Self::format_multiline_comment(translated, format);
        }

        match format.style {
            CommentStyle::Line => {
                // For line comments, return the text as-is (prefix is preserved in original line)
                translated.to_string()
            }
            CommentStyle::BlockSingle => {
                // Single-line block comment: /* text */
                format!("/* {} */", translated)
            }
            CommentStyle::BlockMulti => {
                // Multi-line block comment with preserved formatting
                Self::format_multiline_block_comment(translated, format)
            }
            CommentStyle::DocOuter => {
                // Outer doc comment: /// text
                // The comment prefix is preserved in the original line
                translated.to_string()
            }
            CommentStyle::DocInner => {
                // Inner doc comment: //! text
                // The comment prefix is preserved in the original line
                translated.to_string()
            }
            CommentStyle::DocBlock => {
                // Block doc comment: /** ... */
                Self::format_multiline_block_comment(translated, format)
            }
        }
    }

    /// Format a multiline comment (merged from multiple lines)
    fn format_multiline_comment(translated: &str, format: &FormatInfo) -> String {
        let lines: Vec<&str> = translated.lines().collect();

        if lines.is_empty() {
            return String::new();
        }

        match format.style {
            CommentStyle::DocOuter | CommentStyle::DocInner => {
                // For merged doc comments, add prefix and base_indent to each line
                let prefix = format.line_prefix.as_deref().unwrap_or("");
                let base_indent = &format.base_indent;
                lines
                    .iter()
                    .map(|line| format!("{}{}{}", base_indent, prefix, line))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            CommentStyle::BlockMulti | CommentStyle::DocBlock => {
                // For block comments, use the existing multiline block comment formatter
                Self::format_multiline_block_comment(translated, format)
            }
            _ => {
                // For other styles, just join with newlines
                lines.join("\n")
            }
        }
    }

    /// Format a multi-line block comment with proper indentation and prefixes
    fn format_multiline_block_comment(translated: &str, format: &FormatInfo) -> String {
        let lines: Vec<&str> = translated.lines().collect();

        if lines.len() == 1 {
            // Single line - use simple format
            let start_marker = if format.style == CommentStyle::DocBlock {
                "/**"
            } else {
                "/*"
            };
            return format!("{} {} */", start_marker, translated);
        }

        let mut result = String::new();
        let start_marker = if format.style == CommentStyle::DocBlock {
            "/**"
        } else {
            "/*"
        };
        result.push_str(start_marker);
        result.push('\n');

        for (i, line) in lines.iter().enumerate() {
            result.push_str(&format.base_indent);
            if let Some(prefix) = &format.line_prefix {
                result.push_str(prefix);
            }
            result.push_str(line);

            // Add newline after each line's content
            // If ends_with_newline is true, add newline after last line's content
            // If ends_with_newline is false, don't add newline after last line's content
            if i < lines.len() - 1 || format.ends_with_newline {
                result.push('\n');
            }
        }

        // Always put the closing marker on a new line
        result.push_str(&format.base_indent);
        result.push_str(" */");

        result
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
            format_info: None,
            pattern_type: None,
            pattern_name: None,
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
            format_info: None,
            pattern_type: None,
            pattern_name: None,
        }];

        units[0].set_translated("你好");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("你好"));
        assert!(result.contains("\r\n"));
    }

    #[test]
    fn test_validate_translations_missing() {
        let units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "Hello".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(1, 6, 5),
            language: None,
            should_translate: true,
            translated: None,
            format_info: Some(FormatInfo {
                style: CommentStyle::Line,
                base_indent: String::new(),
                line_prefix: None,
                ends_with_newline: false,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
        }];

        let result = TranslationApplier::apply_translations("content", &units);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_line_comment() {
        let content = "    // This is a comment\nint x = 5;";
        let mut units = vec![TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "This is a comment".to_string(),
            start_pos: Position::new(1, 8, 7),
            end_pos: Position::new(1, 25, 24),
            language: None,
            should_translate: true,
            translated: None,
            format_info: Some(FormatInfo {
                style: CommentStyle::Line,
                base_indent: "    ".to_string(),
                line_prefix: Some("// ".to_string()),
                ends_with_newline: false,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
        }];

        units[0].set_translated("这是一个注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        println!("Result: {:?}", result);
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
            format_info: Some(FormatInfo {
                style: CommentStyle::BlockSingle,
                base_indent: "".to_string(),
                line_prefix: None,
                ends_with_newline: false,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
        }];

        units[0].set_translated("这是一个注释");

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
            format_info: Some(FormatInfo {
                style: CommentStyle::BlockMulti,
                base_indent: "".to_string(),
                line_prefix: Some(" * ".to_string()),
                ends_with_newline: true,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
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
            content: "    /*\n     * This is a\n     * multi-line comment\n     */".to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(4, 5, 37),
            language: None,
            should_translate: true,
            translated: None,
            format_info: Some(FormatInfo {
                style: CommentStyle::BlockMulti,
                base_indent: "    ".to_string(),
                line_prefix: Some(" * ".to_string()),
                ends_with_newline: true,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
        }];

        units[0].set_translated("这是一个\n多行注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        println!("Result: {:?}", result);
        assert!(result.contains("/*\n     * 这是一个\n     * 多行注释\n     */"));
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
            format_info: Some(FormatInfo {
                style: CommentStyle::DocOuter,
                base_indent: "".to_string(),
                line_prefix: Some("/// ".to_string()),
                ends_with_newline: false,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
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
            format_info: Some(FormatInfo {
                style: CommentStyle::DocBlock,
                base_indent: "".to_string(),
                line_prefix: Some(" * ".to_string()),
                ends_with_newline: true,
                is_multiline: true,
            }),
            pattern_type: None,
            pattern_name: None,
        }];

        units[0].set_translated("这是一个\n文档注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        println!("Result: {:?}", result);
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
            format_info: Some(FormatInfo {
                style: CommentStyle::BlockMulti,
                base_indent: "".to_string(),
                line_prefix: Some(" * ".to_string()),
                ends_with_newline: true,
                is_multiline: false,
            }),
            pattern_type: None,
            pattern_name: None,
        }];

        units[0].set_translated("第一行\n第二行");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("/*\n * 第一行\n * 第二行\n */"));
    }

    #[test]
    fn test_format_without_format_info() {
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
            format_info: None,
            pattern_type: None,
            pattern_name: None,
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
                format_info: Some(FormatInfo {
                    style: CommentStyle::Line,
                    base_indent: "    ".to_string(),
                    line_prefix: None,
                    ends_with_newline: false,
                    is_multiline: false,
                }),
                pattern_type: None,
                pattern_name: None,
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
                format_info: Some(FormatInfo {
                    style: CommentStyle::Line,
                    base_indent: "    ".to_string(),
                    line_prefix: None,
                    ends_with_newline: false,
                    is_multiline: false,
                }),
                pattern_type: None,
                pattern_name: None,
            },
        ];

        units[0].set_translated("// 第一个注释");
        units[1].set_translated("// 第二个注释");

        let result = TranslationApplier::apply_translations(content, &units).unwrap();
        assert!(result.contains("// 第一个注释"));
        assert!(result.contains("// 第二个注释"));
    }
}
