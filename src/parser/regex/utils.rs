//! Utility functions for regex parser

use crate::core::models::Position;

/// Check if character is punctuation
pub fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        '!'
            | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// Convert byte offset to line/column position
pub fn byte_to_position(content: &str, byte_offset: usize) -> Position {
    let content_up_to_offset = &content[..byte_offset.min(content.len())];
    let lines: Vec<&str> = content_up_to_offset.lines().collect();

    let line = lines.len();
    let column = lines.last().map(|l| l.len() + 1).unwrap_or(1);

    Position::new(line, column, byte_offset)
}

/// Check if content should be included based on filters
pub fn should_include(text: &str, min_length: usize, max_length: usize) -> bool {
    let len = text.len();
    if len < min_length || len > max_length {
        return false;
    }

    // Skip if only symbols/whitespace
    if text.chars().all(|c| c.is_whitespace() || is_punctuation(c)) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_punctuation() {
        assert!(is_punctuation('!'));
        assert!(is_punctuation('.'));
        assert!(is_punctuation('@'));
        assert!(!is_punctuation('a'));
        assert!(!is_punctuation('1'));
    }

    #[test]
    fn test_byte_to_position() {
        let content = "line1\nline2\nline3";
        let pos = byte_to_position(content, 7);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 7);
    }

    #[test]
    fn test_should_include() {
        assert!(should_include("hello", 2, 100));
        assert!(!should_include("h", 2, 100)); // Too short
        assert!(!should_include("!@#$%", 2, 100)); // Only punctuation
    }
}
