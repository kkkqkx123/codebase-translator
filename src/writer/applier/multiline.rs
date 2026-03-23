//! Multi-line and block comment translation applier
//!
//! This module handles applying translations to multi-line content
//! and block comments (/* */), preserving the original formatting.

use crate::core::models::TranslationUnit;
use crate::writer::format::prefix::extract_comment_prefix;

/// A replacement segment to be applied to content
#[derive(Debug, Clone)]
struct Replacement {
    /// Start byte offset in the original content
    start_offset: usize,
    /// End byte offset in the original content
    end_offset: usize,
    /// The text to replace with
    replacement: String,
    /// The raw_match text (for verification)
    raw_match: String,
}

/// Applies translations to multi-line content
pub struct MultilineApplier;

impl MultilineApplier {
    /// Apply translations to multi-line units
    ///
    /// Uses a two-phase approach to avoid offset issues:
    /// 1. Collect all replacements with their original positions
    /// 2. Apply all replacements in reverse order (from end to start)
    ///
    /// # Arguments
    /// * `content` - Original file content
    /// * `units` - Multi-line translation units with translated content
    ///
    /// # Returns
    /// Modified content with translations applied
    pub fn apply(content: &str, units: &[&TranslationUnit]) -> String {
        if units.is_empty() {
            return content.to_string();
        }

        // Phase 1: Collect all valid replacements
        let replacements = Self::collect_replacements(content, units);

        if replacements.is_empty() {
            return content.to_string();
        }

        // Phase 2: Apply replacements in reverse order (from end to start)
        // This ensures that earlier replacements don't affect later ones
        Self::apply_replacements(content, &replacements)
    }

    /// Collect all valid replacements from translation units
    fn collect_replacements(content: &str, units: &[&TranslationUnit]) -> Vec<Replacement> {
        let mut replacements = Vec::new();

        for unit in units {
            if let (Some(raw_match), Some(translated)) = (&unit.raw_match, &unit.translated) {
                // Skip if content and translation are the same
                if raw_match.trim() == translated.trim() {
                    continue;
                }

                let formatted = Self::format_translation(raw_match, translated, false);

                // Try to find the raw_match in content using offset information
                if let Some(replacement) =
                    Self::find_replacement(content, unit, raw_match, &formatted)
                {
                    replacements.push(replacement);
                }
            }
        }

        // Sort by start_offset in descending order (process from end to start)
        replacements.sort_by(|a, b| b.start_offset.cmp(&a.start_offset));

        replacements
    }

    /// Find a replacement location in content
    fn find_replacement(
        content: &str,
        unit: &TranslationUnit,
        raw_match: &str,
        formatted: &str,
    ) -> Option<Replacement> {
        let start_offset = unit.start_pos.offset;
        let end_offset = unit.end_pos.offset;

        // Strategy 1: Try exact match at the reported offset
        if let Some(slice) = content.get(start_offset..end_offset) {
            if slice == raw_match {
                return Some(Replacement {
                    start_offset,
                    end_offset,
                    replacement: formatted.to_string(),
                    raw_match: raw_match.to_string(),
                });
            }
        }

        // Strategy 2: Try to find raw_match anywhere in content
        if let Some(pos) = content.find(raw_match) {
            return Some(Replacement {
                start_offset: pos,
                end_offset: pos + raw_match.len(),
                replacement: formatted.to_string(),
                raw_match: raw_match.to_string(),
            });
        }

        // Strategy 3: Try with normalized line endings
        let normalized_raw_match = raw_match.replace("\r\n", "\n");
        let normalized_content = content.replace("\r\n", "\n");

        if let Some(pos) = normalized_content.find(&normalized_raw_match) {
            // Map position back to original content
            let original_pos = Self::map_normalized_to_original(content, pos);
            let original_end =
                Self::map_normalized_to_original(content, pos + normalized_raw_match.len());

            return Some(Replacement {
                start_offset: original_pos,
                end_offset: original_end,
                replacement: formatted.to_string(),
                raw_match: raw_match.to_string(),
            });
        }

        None
    }

    /// Map a position in normalized content (with \n) back to original content (with \r\n)
    fn map_normalized_to_original(original: &str, normalized_pos: usize) -> usize {
        let mut normalized_count = 0;

        for (i, c) in original.char_indices() {
            if normalized_count >= normalized_pos {
                return i;
            }

            if c != '\r' {
                // \r is not counted in normalized content
                normalized_count += c.len_utf8();
            }
        }

        original.len()
    }

    /// Apply all replacements to content
    fn apply_replacements(content: &str, replacements: &[Replacement]) -> String {
        let mut result = content.to_string();
        let mut offset_adjustment: isize = 0;

        // Process replacements in reverse order (already sorted)
        for replacement in replacements.iter().rev() {
            let adjusted_start =
                (replacement.start_offset as isize + offset_adjustment).max(0) as usize;
            let adjusted_end =
                (replacement.end_offset as isize + offset_adjustment).max(0) as usize;

            // Verify the slice matches expected content
            if let Some(slice) = result.get(adjusted_start..adjusted_end) {
                let expected = &replacement.raw_match;
                let normalized_slice = slice.replace("\r\n", "\n");
                let normalized_expected = expected.replace("\r\n", "\n");

                if normalized_slice == normalized_expected {
                    // Apply the replacement
                    let old_len = adjusted_end - adjusted_start;
                    let new_len = replacement.replacement.len();
                    let diff = new_len as isize - old_len as isize;

                    result.replace_range(adjusted_start..adjusted_end, &replacement.replacement);
                    offset_adjustment += diff;
                }
            }
        }

        result
    }

    /// Format a multiline translation by applying it line by line to the raw match
    ///
    /// # Arguments
    /// * `raw_match` - The original matched text from the parser
    /// * `translated` - The translated text
    /// * `force_trailing_newline` - Whether to force a trailing newline (from original content)
    pub fn format_translation(
        raw_match: &str,
        translated: &str,
        force_trailing_newline: bool,
    ) -> String {
        let raw_lines: Vec<&str> = raw_match.lines().collect();
        let translated_lines: Vec<&str> = translated.lines().collect();

        // Check if raw_match ends with newline - we need to preserve this
        let ends_with_newline = raw_match.ends_with('\n') || force_trailing_newline;

        // Check if this is a block comment (starts with /* or /**)
        let is_block_comment = raw_lines
            .first()
            .map(|line| line.trim().starts_with("/*"))
            .unwrap_or(false);

        let result = if is_block_comment {
            Self::format_block_comment(&raw_lines, &translated_lines, translated)
        } else {
            Self::format_simple_multiline(&raw_lines, &translated_lines, translated)
        };

        // Preserve trailing newline if raw_match had one
        if ends_with_newline && !result.ends_with('\n') {
            format!("{}\n", result)
        } else {
            result
        }
    }

    /// Format block comments (/* */ or /** */)
    fn format_block_comment(
        raw_lines: &[&str],
        translated_lines: &[&str],
        translated: &str,
    ) -> String {
        // Check if translated already contains block comment markers
        // If so, use it directly without reformatting
        let trimmed_translated = translated.trim();
        if trimmed_translated.starts_with("/*") && trimmed_translated.ends_with("*/") {
            return translated.to_string();
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
                let prefix = extract_comment_prefix(raw_line);
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
    }

    /// Format simple multi-line content (non-block comments)
    fn format_simple_multiline(
        raw_lines: &[&str],
        translated_lines: &[&str],
        translated: &str,
    ) -> String {
        // Check if translated already has comment prefixes
        // If so, use it directly without reformatting
        let trimmed_translated = translated.trim_start();
        if trimmed_translated.starts_with("//") || trimmed_translated.starts_with("/*") {
            return translated.to_string();
        }

        // Check if this is a Python docstring (starts with """ or ''')
        let is_python_docstring = raw_lines
            .first()
            .map(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''")
            })
            .unwrap_or(false);

        if is_python_docstring {
            return Self::format_python_docstring(raw_lines, translated_lines, translated);
        }

        if raw_lines.len() == translated_lines.len() {
            let mut result = String::new();
            for (i, raw_line) in raw_lines.iter().enumerate() {
                let prefix = extract_comment_prefix(raw_line);
                result.push_str(&prefix);
                result.push_str(translated_lines[i]);
                if i < raw_lines.len() - 1 {
                    result.push('\n');
                }
            }
            result
        } else {
            // Line count mismatch - distribute translated content across raw lines
            // This handles cases where raw_match has empty lines or different structure
            let mut result = String::new();
            let mut translated_idx = 0;

            for (i, raw_line) in raw_lines.iter().enumerate() {
                let trimmed = raw_line.trim();
                let prefix = extract_comment_prefix(raw_line);
                result.push_str(&prefix);

                // If this raw line has actual content (not just prefix), add translated content
                if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                    // This line has content beyond the prefix
                    if translated_idx < translated_lines.len() {
                        result.push_str(translated_lines[translated_idx]);
                        translated_idx += 1;
                    }
                } else if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    // This is a comment line - check if it has content after the prefix
                    let content_start = raw_line
                        .find(|c: char| !c.is_whitespace())
                        .map(|i| {
                            if raw_line[i..].starts_with("///") {
                                i + 3
                            } else if raw_line[i..].starts_with("//") {
                                i + 2
                            } else {
                                i
                            }
                        })
                        .unwrap_or(raw_line.len());

                    let after_prefix = &raw_line[content_start..];
                    let after_prefix_trimmed = after_prefix.trim_start();

                    if !after_prefix_trimmed.is_empty() {
                        // This line has content after the comment marker
                        if translated_idx < translated_lines.len() {
                            result.push_str(translated_lines[translated_idx]);
                            translated_idx += 1;
                        }
                    }
                    // If empty after prefix, just keep the prefix (handles empty comment lines)
                }

                if i < raw_lines.len() - 1 {
                    result.push('\n');
                }
            }

            // If there's remaining translated content, append it
            if translated_idx < translated_lines.len() {
                for i in translated_idx..translated_lines.len() {
                    result.push_str(translated_lines[i]);
                    if i < translated_lines.len() - 1 {
                        result.push('\n');
                    }
                }
            }

            result
        }
    }

    /// Format Python docstring preserving triple quotes and indentation
    fn format_python_docstring(
        raw_lines: &[&str],
        translated_lines: &[&str],
        _translated: &str,
    ) -> String {
        let mut result = String::new();

        // Determine the base indentation from the opening triple quote line
        let base_indent = raw_lines
            .first()
            .map(|line| {
                let trimmed = line.trim_start();
                let leading_whitespace = &line[..(line.len() - trimmed.len())];
                leading_whitespace.to_string()
            })
            .unwrap_or_default();

        // Determine the content indentation (from the first non-empty content line)
        let content_indent = raw_lines
            .iter()
            .skip(1)
            .find(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("\"\"\"") && !trimmed.starts_with("'''")
            })
            .map(|line| {
                let trimmed = line.trim_start();
                let leading_whitespace = &line[..(line.len() - trimmed.len())];
                leading_whitespace.to_string()
            })
            .unwrap_or_else(|| base_indent.clone() + "    ");

        // Filter out empty lines from translated_lines to get only content lines
        let translated_content_lines: Vec<&str> = translated_lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .copied()
            .collect();

        // Count content lines in raw (excluding triple quotes and empty lines)
        let raw_content_line_count = raw_lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                let trimmed_start = line.trim_start();
                !trimmed.is_empty()
                    && !trimmed_start.starts_with("\"\"\"")
                    && !trimmed_start.starts_with("'''")
            })
            .count();

        // Check if the number of content lines matches
        let line_counts_match = raw_content_line_count == translated_content_lines.len();

        // Build the result
        let mut translated_idx = 0;

        for (i, raw_line) in raw_lines.iter().enumerate() {
            let trimmed = raw_line.trim();
            let trimmed_start = raw_line.trim_start();

            let is_triple_quote =
                trimmed_start.starts_with("\"\"\"") || trimmed_start.starts_with("'''");
            let is_empty = trimmed.is_empty();

            if is_triple_quote {
                // Preserve triple quote line with its original indentation
                result.push_str(raw_line);
            } else if is_empty {
                // Preserve empty lines with original indentation
                result.push_str(raw_line);
            } else {
                // This is a content line with actual text
                if line_counts_match && translated_idx < translated_content_lines.len() {
                    // Line counts match - use corresponding translated content line
                    result.push_str(&content_indent);
                    result.push_str(translated_content_lines[translated_idx]);
                    translated_idx += 1;
                } else if translated_idx < translated_content_lines.len() {
                    // Line count mismatch - use sequential mapping
                    result.push_str(&content_indent);
                    result.push_str(translated_content_lines[translated_idx]);
                    translated_idx += 1;
                } else {
                    // No more translated content, preserve original line
                    result.push_str(raw_line);
                }
            }

            if i < raw_lines.len() - 1 {
                result.push('\n');
            }
        }

        // If there's remaining translated content, append it before the closing triple quote
        if translated_idx < translated_content_lines.len() {
            for line in translated_content_lines.iter().skip(translated_idx) {
                result.push('\n');
                result.push_str(&content_indent);
                result.push_str(line);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{NodeType, Position};

    fn create_multiline_unit(
        content: &str,
        raw_match: &str,
        start_line: usize,
        end_line: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> TranslationUnit {
        TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: content.to_string(),
            start_pos: Position::new(start_line, 1, start_offset),
            end_pos: Position::new(end_line, 10, end_offset),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some(raw_match.to_string()),
        }
    }

    #[test]
    fn test_format_block_comment() {
        let raw = "/*\n * Line 1\n * Line 2\n */";
        let translated = "第一行\n第二行";

        let result = MultilineApplier::format_translation(raw, translated, false);

        assert!(result.contains("/*"));
        assert!(result.contains(" * 第一行"));
        assert!(result.contains(" * 第二行"));
        assert!(result.contains(" */"));
    }

    #[test]
    fn test_format_simple_multiline() {
        let raw = "// Line 1\n// Line 2";
        let translated = "第一行\n第二行";

        let result = MultilineApplier::format_translation(raw, translated, false);

        assert!(result.contains("// 第一行"));
        assert!(result.contains("// 第二行"));
    }

    #[test]
    fn test_preserve_trailing_newline() {
        let raw = "// Line 1\n// Line 2\n";
        let translated = "第一行\n第二行";

        let result = MultilineApplier::format_translation(raw, translated, true);

        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_apply_multiline_units() {
        let content = "/*\n * Hello\n * World\n */\nother code";
        let mut unit =
            create_multiline_unit("Hello\nWorld", "/*\n * Hello\n * World\n */", 1, 4, 0, 25);
        unit.set_translated("你好\n世界");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = MultilineApplier::apply(content, &units);

        assert!(result.contains(" * 你好"));
        assert!(result.contains(" * 世界"));
        assert!(result.contains("other code"));
    }

    // Regression tests for Windows line ending handling
    #[test]
    fn test_apply_with_crlf_in_raw_match() {
        // raw_match has \r\n but content has \n (common on Windows)
        let content = "//! Hello World\n//! Second line\npub mod test;";
        let mut unit = create_multiline_unit(
            "Hello World",
            "//! Hello World\r\n",
            1,
            1,
            0,
            15,
        );
        unit.set_translated("你好世界");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = MultilineApplier::apply(content, &units);

        assert!(result.contains("//! 你好世界"));
        assert!(result.contains("//! Second line"));
        assert!(result.contains("pub mod test;"));
    }

    #[test]
    fn test_apply_with_crlf_in_content() {
        // content has \r\n but raw_match has \n
        let content = "//! Hello World\r\n//! Second line\r\npub mod test;";
        let mut unit = create_multiline_unit(
            "Hello World",
            "//! Hello World\n",
            1,
            1,
            0,
            14,
        );
        unit.set_translated("你好世界");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = MultilineApplier::apply(content, &units);

        assert!(result.contains("//! 你好世界"));
        assert!(result.contains("//! Second line"));
        assert!(result.contains("pub mod test;"));
    }

    #[test]
    fn test_apply_with_mixed_line_endings() {
        // Mixed line endings in content
        let content = "//! Line 1\r\n//! Line 2\n//! Line 3\r\npub mod test;";
        let mut unit = create_multiline_unit(
            "Line 2",
            "//! Line 2\r\n",
            2,
            2,
            14,
            28,
        );
        unit.set_translated("第二行");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = MultilineApplier::apply(content, &units);

        assert!(result.contains("//! Line 1"));
        assert!(result.contains("//! 第二行"));
        assert!(result.contains("//! Line 3"));
    }

    #[test]
    fn test_apply_multiline_with_crlf_line_endings() {
        // Test full apply with multiple units having CRLF
        let content = "//! 通用基础设施模块\r\n//!\r\n//! 这个模块包含了所有通用的基础设施代码\r\npub mod id;";
        let mut unit1 =
            create_multiline_unit("通用基础设施模块", "//! 通用基础设施模块\r\n", 1, 1, 0, 20);
        unit1.set_translated("Common Infrastructure Module");

        let mut unit2 = create_multiline_unit(
            "这个模块包含了所有通用的基础设施代码",
            "//! 这个模块包含了所有通用的基础设施代码\r\n",
            3,
            3,
            0,
            40,
        );
        unit2.set_translated("This module contains all the general infrastructure code");

        let units: Vec<&TranslationUnit> = vec![&unit1, &unit2];
        let result = MultilineApplier::apply(content, &units);

        assert!(result.contains("//! Common Infrastructure Module"));
        assert!(result.contains("//! This module contains all the general infrastructure code"));
        assert!(result.contains("pub mod id;"));
    }

    #[test]
    fn test_apply_no_match() {
        // When raw_match doesn't exist in content, content should remain unchanged
        let content = "//! Some content\n//! More content";
        let mut unit = create_multiline_unit(
            "Non-existent",
            "//! Non-existent content\r\n",
            1,
            1,
            0,
            10,
        );
        unit.set_translated("翻译内容");

        let units: Vec<&TranslationUnit> = vec![&unit];
        let result = MultilineApplier::apply(content, &units);

        // Content should remain unchanged since raw_match doesn't match
        assert_eq!(result, content);
    }

    #[test]
    fn test_apply_multiple_units_offset_handling() {
        // Test that multiple units are correctly applied without offset issues
        let content = "//! First\n//! Second\n//! Third\npub mod test;";

        let mut unit1 = create_multiline_unit(
            "First",
            "//! First\n",
            1,
            1,
            0,
            10,
        );
        unit1.set_translated("第一");

        let mut unit2 = create_multiline_unit(
            "Second",
            "//! Second\n",
            2,
            2,
            10,
            21,
        );
        unit2.set_translated("第二");

        let mut unit3 = create_multiline_unit(
            "Third",
            "//! Third\n",
            3,
            3,
            21,
            31,
        );
        unit3.set_translated("第三");

        let units: Vec<&TranslationUnit> = vec![&unit1, &unit2, &unit3];
        let result = MultilineApplier::apply(content, &units);

        assert!(result.contains("//! 第一"));
        assert!(result.contains("//! 第二"));
        assert!(result.contains("//! 第三"));
        assert!(result.contains("pub mod test;"));
    }
}
