//! Text replacement utilities
//!
//! This module provides functionality for replacing extracted text
//! within raw match strings, handling various edge cases and formats.

use tracing;

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
pub fn replace_in_raw_match(raw_match: &str, extracted: &str, translated: &str) -> String {
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

    // Handle block comments where raw_match has more lines (markers) than extracted content
    // e.g., raw_match: [/**,  * Line 1,  * Line 2,  */]
    //       extracted: [Line 1, Line 2]
    if raw_lines.len() > extracted_lines.len() {
        return replace_block_comment(&raw_lines, &extracted_lines, translated);
    }

    if raw_lines.len() == extracted_lines.len() {
        replace_line_by_line(&raw_lines, &extracted_lines, translated)
    } else {
        // If all else fails, log a warning and return raw match as-is
        tracing::warn!(
            raw_match = %raw_match,
            extracted = %extracted,
            "Extracted text not found in raw match, skipping replacement"
        );
        raw_match.to_string()
    }
}

/// Replace text in block comments where raw_match has marker lines (/**, */, etc.)
///
/// This handles cases like:
///   raw_match: [/**,  * Line 1,  * Line 2,  */]
///   extracted: [Line 1, Line 2]
fn replace_block_comment(raw_lines: &[&str], extracted_lines: &[&str], translated: &str) -> String {
    let translated_lines: Vec<&str> = translated.lines().collect();
    let mut result = String::new();
    let mut extracted_idx = 0;

    for (i, raw_line) in raw_lines.iter().enumerate() {

        // Check if this line contains extracted content (not just markers)
        if extracted_idx < extracted_lines.len() {
            let extracted_line = extracted_lines[extracted_idx];

            // Try to find extracted_line in raw_line
            if let Some(pos) = raw_line.find(extracted_line) {
                let before = &raw_line[..pos];
                let after = &raw_line[pos + extracted_line.len()..];

                // For multi-line translations with same line count, preserve formatting
                if translated_lines.len() == extracted_lines.len() {
                    result.push_str(&format!("{}{}{}", before, translated_lines[extracted_idx], after));
                } else {
                    // If line count differs, use the whole translated text on first match
                    if extracted_idx == 0 {
                        result.push_str(&format!("{}{}{}", before, translated, after));
                    }
                    // Skip remaining extracted lines since we already placed all content
                }

                extracted_idx += 1;
            } else {
                // This line is a marker line (like /**, */, or * prefix without content)
                // Keep it as-is
                result.push_str(raw_line);
            }
        } else {
            // No more extracted content, keep remaining lines as-is (e.g., closing */)
            result.push_str(raw_line);
        }

        if i < raw_lines.len() - 1 {
            result.push('\n');
        }
    }

    result
}

/// Replace text line by line for multi-line content
///
/// Attempts to match each extracted line within the corresponding raw line
/// and replace with the appropriate translated line.
fn replace_line_by_line(raw_lines: &[&str], extracted_lines: &[&str], translated: &str) -> String {
    let translated_lines: Vec<&str> = translated.lines().collect();
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

    result
}

/// Convert byte position to character position in a string
pub fn byte_to_char_pos(s: &str, byte_pos: usize) -> usize {
    s.char_indices()
        .take_while(|(pos, _)| *pos < byte_pos)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_in_raw_match_direct() {
        let raw = "println!(\"Hello world\");";
        let extracted = "Hello world";
        let translated = "你好世界";

        let result = replace_in_raw_match(raw, extracted, translated);
        assert_eq!(result, "println!(\"你好世界\");");
    }

    #[test]
    fn test_replace_in_raw_match_multiline() {
        let raw = "// Line 1\n// Line 2";
        let extracted = "Line 1\nLine 2";
        let translated = "第一行\n第二行";

        let result = replace_in_raw_match(raw, extracted, translated);
        assert!(result.contains("// 第一行"));
        assert!(result.contains("// 第二行"));
    }

    #[test]
    fn test_replace_in_raw_match_with_comment_prefix() {
        // Simulate E2E test case: Chinese comment translated to English
        let raw = "// 这是一个简单的JavaScript文件，用于测试翻译功能";
        let extracted = "这是一个简单的JavaScript文件，用于测试翻译功能";
        let translated = "This is a simple JavaScript file to test the translation function";

        let result = replace_in_raw_match(raw, extracted, translated);
        // Should replace the content, not append
        assert_eq!(
            result,
            "// This is a simple JavaScript file to test the translation function"
        );
    }

    #[test]
    fn test_replace_in_raw_match_block_comment() {
        // Test block comment replacement where raw_match has more lines than extracted
        // raw_match:    [/**,  * 配置加载器,  * 支持多种配置文件格式,  */]
        // extracted:    [配置加载器, 支持多种配置文件格式]
        // translated:   [Configuration Loader, Supports multiple configuration file formats]
        let raw = "/**\n * 配置加载器\n * 支持多种配置文件格式\n */";
        let extracted = "配置加载器\n支持多种配置文件格式";
        let translated = "Configuration Loader\nSupports multiple configuration file formats";

        let result = replace_in_raw_match(raw, extracted, translated);

        // Verify exact format - should preserve structure with markers
        let expected = "/**\n * Configuration Loader\n * Supports multiple configuration file formats\n */";
        assert_eq!(result, expected, "Block comment replacement should preserve markers and replace only content");

        // Should preserve markers and replace content
        assert!(result.contains("/**"), "Opening marker should be preserved");
        assert!(result.contains("*/"), "Closing marker should be preserved");
        assert!(result.contains("Configuration Loader"), "First line should be translated");
        assert!(result.contains("Supports multiple configuration file formats"), "Second line should be translated");
        // Should not contain original Chinese text
        assert!(!result.contains("配置加载器"), "Original Chinese should be replaced");
        assert!(!result.contains("支持多种配置文件格式"), "Original Chinese should be replaced");
    }

    #[test]
    fn test_byte_to_char_pos() {
        let s = "Hello 世界";
        // "Hello " is 6 bytes in UTF-8 (space is 1 byte)
        assert_eq!(byte_to_char_pos(s, 0), 0);
        assert_eq!(byte_to_char_pos(s, 6), 6);
        // "世" is 3 bytes in UTF-8
        assert_eq!(byte_to_char_pos(s, 9), 7);
    }
}
