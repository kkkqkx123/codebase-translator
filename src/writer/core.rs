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

        Self::validate_translations(units)?;

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

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(line_units) = unit_map.get(&(line_num + 1)) {
                builder.push_str(&Self::apply_translations_to_line(line, line_units));
            } else {
                builder.push_str(line);
            }
            if line_num < lines.len() - 1 {
                builder.push_str(line_ending);
            }
        }

        Ok(builder)
    }

    /// Validate that all translatable units have been translated
    fn validate_translations(units: &[TranslationUnit]) -> Result<()> {
        let untranslated: Vec<&str> = units
            .iter()
            .filter(|u| u.should_translate && u.translated.is_none())
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
                unit.translated.as_ref().map(|translated| Replacement {
                    start_char: unit.start_pos.column.saturating_sub(1),
                    end_char: unit.end_pos.column.saturating_sub(1),
                    text: translated.clone(),
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
        }];

        let result = TranslationApplier::apply_translations("content", &units);
        assert!(result.is_err());
    }
}
