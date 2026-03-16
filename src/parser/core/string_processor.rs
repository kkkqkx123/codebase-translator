//! String processing utilities

/// Type of comment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentType {
    /// Line comment: // or #
    Line,
    /// Block comment: /* */
    Block,
    /// Documentation comment: /// or //! or /**
    Doc,
}

/// String processor for cleaning and transforming string literals
pub struct StringProcessor;

impl StringProcessor {
    /// Create a new string processor
    pub fn new() -> Self {
        Self
    }

    /// Clean comment content by removing comment markers
    ///
    /// Supports:
    /// - Line comments: //, #
    /// - Block comments: /* */
    /// - Doc comments: ///, //!, /**
    pub fn clean_comment(&self, text: &str, comment_type: CommentType) -> String {
        match comment_type {
            CommentType::Line => self.clean_line_comment(text),
            CommentType::Block => self.clean_block_comment(text),
            CommentType::Doc => self.clean_doc_comment(text),
        }
    }

    /// Clean line comment (// or #)
    fn clean_line_comment(&self, text: &str) -> String {
        let text = text.trim();

        // Handle Python/Ruby/YAML style: #
        if let Some(content) = text.strip_prefix('#') {
            return content.trim_start().to_string();
        }

        // Handle C-style: //
        if let Some(content) = text.strip_prefix("//") {
            return content.trim_start().to_string();
        }

        text.to_string()
    }

    /// Clean block comment (/* */)
    fn clean_block_comment(&self, text: &str) -> String {
        let text = text.trim();

        // Remove /* and */
        let content = text
            .strip_prefix("/*")
            .and_then(|s| s.strip_suffix("*/"))
            .unwrap_or(text);

        content.trim().to_string()
    }

    /// Clean documentation comment (/// or //! or /**)
    fn clean_doc_comment(&self, text: &str) -> String {
        let text = text.trim();

        // Handle Rust outer doc: ///
        if let Some(content) = text.strip_prefix("///") {
            return content.trim_start().to_string();
        }

        // Handle Rust inner doc: //!
        if let Some(content) = text.strip_prefix("//!") {
            return content.trim_start().to_string();
        }

        // Handle Javadoc/Rust block doc: /**
        if text.starts_with("/**") {
            let content = text
                .strip_prefix("/**")
                .and_then(|s| s.strip_suffix("*/"))
                .unwrap_or(text);
            return content.trim().to_string();
        }

        // Handle JavaScript JSDoc: /**
        if text.starts_with("/**") {
            let content = text
                .strip_prefix("/**")
                .and_then(|s| s.strip_suffix("*/"))
                .unwrap_or(text);
            return content.trim().to_string();
        }

        self.clean_line_comment(text)
    }

    /// Clean string literal by removing quotes and handling escape sequences
    ///
    /// Supports:
    /// - Regular strings: "hello"
    /// - Raw strings: r"hello", r#"hello "world""#, etc.
    pub fn clean_string_literal(&self, text: &str) -> String {
        // Handle Go raw strings: `...`
        if text.starts_with('`') && text.ends_with('`') {
            let content = &text[1..text.len() - 1];
            return content.to_string();
        }

        // Handle raw strings: r"...", r#"..."#, r##"..."##, etc.
        if text.starts_with('r') {
            self.process_raw_string(text)
        } else {
            // Regular string: remove quotes and unescape
            let text = text.trim_matches('"');
            self.unescape(text)
        }
    }

    /// Process raw string literal
    fn process_raw_string(&self, text: &str) -> String {
        let mut chars = text.chars().peekable();
        chars.next(); // Skip 'r'

        // Count leading #
        let mut hash_count = 0;
        while chars.peek() == Some(&'#') {
            hash_count += 1;
            chars.next();
        }

        // Skip opening quote
        if chars.peek() == Some(&'"') {
            chars.next();
        }

        // Collect content
        let content: String = chars.collect();

        // Remove trailing quote and hashes
        if hash_count > 0 {
            let end_pattern = "\"".to_string() + &"#".repeat(hash_count);
            content
                .strip_suffix(&end_pattern)
                .map(|s| s.to_string())
                .unwrap_or(content)
        } else {
            content
                .strip_suffix('"')
                .map(|s| s.to_string())
                .unwrap_or(content)
        }
    }

    /// Unescape string sequences
    pub fn unescape(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('0') => result.push('\0'),
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Check if text contains only symbols/whitespace
    pub fn is_only_symbols(&self, text: &str) -> bool {
        text.chars()
            .all(|c| c.is_whitespace() || self.is_punctuation(c))
    }

    /// Check if character is punctuation
    fn is_punctuation(&self, c: char) -> bool {
        matches!(
            c,
            '!' | '"'
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
}

impl Default for StringProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_regular_string() {
        let processor = StringProcessor::new();
        assert_eq!(
            processor.clean_string_literal("\"hello world\""),
            "hello world"
        );
    }

    #[test]
    fn test_clean_string_with_escapes() {
        let processor = StringProcessor::new();
        assert_eq!(
            processor.clean_string_literal("\"hello\\nworld\""),
            "hello\nworld"
        );
    }

    #[test]
    fn test_clean_raw_string() {
        let processor = StringProcessor::new();
        assert_eq!(
            processor.clean_string_literal(r##"r#"hello world"#"##),
            "hello world"
        );
    }

    #[test]
    fn test_clean_raw_string_with_quotes() {
        let processor = StringProcessor::new();
        // Raw strings don't process escape sequences
        assert_eq!(
            processor.clean_string_literal(r##"r#"hello "world""#"##),
            "hello \"world\""
        );
    }

    #[test]
    fn test_unescape() {
        let processor = StringProcessor::new();
        assert_eq!(processor.unescape("hello\\nworld"), "hello\nworld");
        assert_eq!(processor.unescape("hello\\tworld"), "hello\tworld");
        assert_eq!(processor.unescape("hello\\\\world"), "hello\\world");
        assert_eq!(processor.unescape("hello\\\"world"), "hello\"world");
    }

    #[test]
    fn test_is_only_symbols() {
        let processor = StringProcessor::new();
        assert!(processor.is_only_symbols("!@#$%"));
        assert!(processor.is_only_symbols("   "));
        assert!(!processor.is_only_symbols("hello"));
        assert!(!processor.is_only_symbols("hello world"));
    }
}
