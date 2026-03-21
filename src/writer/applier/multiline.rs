//! Multi-line and block comment translation applier
//!
//! This module handles applying translations to multi-line content
//! and block comments (/* */), preserving the original formatting.

use crate::core::models::TranslationUnit;
use crate::writer::format::prefix::extract_comment_prefix;

/// Applies translations to multi-line content
pub struct MultilineApplier;

impl MultilineApplier {
    /// Apply translations to multi-line units
    ///
    /// # Arguments
    /// * `content` - Original file content
    /// * `units` - Multi-line translation units with translated content
    ///
    /// # Returns
    /// Modified content with translations applied
    pub fn apply(content: &str, units: &[&TranslationUnit]) -> String {
        let mut result = content.to_string();

        // Sort multiline units by position in reverse order to avoid offset issues
        let mut sorted_units: Vec<&TranslationUnit> = units.to_vec();
        sorted_units.sort_by(|a, b| b.start_pos.offset.cmp(&a.start_pos.offset));

        for unit in sorted_units {
            if let (Some(raw_match), Some(translated)) = (&unit.raw_match, &unit.translated) {
                result = Self::apply_single_unit(&result, unit, raw_match, translated);
            }
        }

        result
    }

    /// Apply a single multi-line translation unit
    fn apply_single_unit(
        content: &str,
        unit: &TranslationUnit,
        raw_match: &str,
        translated: &str,
    ) -> String {
        let start_byte = unit.start_pos.offset;
        let end_byte = unit.end_pos.offset;

        // Check if we have valid byte positions
        let has_valid_positions =
            start_byte > 0 && start_byte < end_byte && end_byte <= content.len();

        if has_valid_positions {
            // Use position-based extraction for accurate replacement
            let original_text = &content[start_byte..end_byte];
            let ends_with_newline = original_text.ends_with('\n');
            let formatted = Self::format_translation(raw_match, translated, ends_with_newline);
            content.replace(original_text, &formatted)
        } else {
            // Fall back to raw_match-based replacement for tests or legacy data
            let formatted = Self::format_translation(raw_match, translated, false);
            content.replace(raw_match, &formatted)
        }
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
    fn format_block_comment(raw_lines: &[&str], translated_lines: &[&str], translated: &str) -> String {
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
        let is_python_docstring = raw_lines.first().map(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''")
        }).unwrap_or(false);

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
                if !trimmed.is_empty()
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
                {
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
        let base_indent = raw_lines.first().map(|line| {
            let trimmed = line.trim_start();
            let leading_whitespace = &line[..(line.len() - trimmed.len())];
            leading_whitespace.to_string()
        }).unwrap_or_default();

        // Determine the content indentation (from the first non-empty content line)
        let content_indent = raw_lines.iter().skip(1).find(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("\"\"\"") && !trimmed.starts_with("'''")
        }).map(|line| {
            let trimmed = line.trim_start();
            let leading_whitespace = &line[..(line.len() - trimmed.len())];
            leading_whitespace.to_string()
        }).unwrap_or_else(|| base_indent.clone() + "    ");

        // Filter out empty lines from translated_lines to get only content lines
        let translated_content_lines: Vec<&str> = translated_lines.iter()
            .filter(|line| !line.trim().is_empty())
            .copied()
            .collect();

        // Count content lines in raw (excluding triple quotes and empty lines)
        let raw_content_line_count = raw_lines.iter()
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

            let is_triple_quote = trimmed_start.starts_with("\"\"\"") || trimmed_start.starts_with("'''");
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
            for i in translated_idx..translated_content_lines.len() {
                result.push('\n');
                result.push_str(&content_indent);
                result.push_str(translated_content_lines[i]);
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
}
