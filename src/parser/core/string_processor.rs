//! String processing utilities

use tracing::trace;

/// Result of cleaning a string literal
#[derive(Debug, Clone)]
pub struct CleanedString {
    /// The cleaned text content (without quotes and escapes)
    pub text: String,
    /// Extracted format placeholders
    pub placeholders: Vec<String>,
}

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

/// Result of cleaning a comment
#[derive(Debug, Clone)]
pub struct CleanedComment {
    /// The cleaned text content (without comment markers)
    pub text: String,
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
    ///
    /// Preserves newlines and removes leading '*' characters from each line
    /// (commonly used in Javadoc-style comments).
    fn clean_block_comment(&self, text: &str) -> String {
        // Only trim leading/trailing whitespace on the outer edges, not internal newlines
        let text = text
            .trim_start()
            .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

        // Remove /* and */
        let content = text
            .strip_prefix("/*")
            .and_then(|s| s.strip_suffix("*/"))
            .unwrap_or(text);

        // Process each line: trim whitespace and remove leading '*' if present
        let lines: Vec<&str> = content.lines().collect();
        let processed_lines: Vec<String> = lines
            .iter()
            .map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with('*') {
                    trimmed[1..].trim_start().to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .collect();

        // Join lines back together, preserving newlines
        processed_lines.join("\n").trim_end().to_string()
    }

    /// Clean documentation comment (/// or //! or /**)
    ///
    /// For line-based doc comments (/// and //!), processes each line separately
    /// to preserve newlines while removing the doc comment markers.
    /// For block doc comments (/**), delegates to block comment cleaning.
    pub fn clean_doc_comment(&self, text: &str) -> String {
        let text = text
            .trim_start()
            .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

        // Handle Rust outer doc: ///
        if text.starts_with("///") {
            return text
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim_start();
                    // Only process lines that start with ///
                    // Skip lines that don't start with /// (e.g., code lines)
                    trimmed
                        .strip_prefix("///")
                        .map(|s| s.trim_start())
                })
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Handle Rust inner doc: //!
        if text.starts_with("//!") {
            return text
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim_start();
                    // Only process lines that start with //!
                    // Skip lines that don't start with //! (e.g., code lines)
                    trimmed
                        .strip_prefix("//!")
                        .map(|s| s.trim_start())
                })
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Handle block doc comments: /** */ (same as block comments but with extra leading '*')
        if text.starts_with("/**") {
            let content = text
                .strip_prefix("/**")
                .and_then(|s| s.strip_suffix("*/"))
                .unwrap_or(text);

            // Process each line: trim whitespace and remove leading '*' if present
            let lines: Vec<&str> = content.lines().collect();
            let processed_lines: Vec<String> = lines
                .iter()
                .map(|line| {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with('*') {
                        trimmed[1..].trim_start().to_string()
                    } else {
                        trimmed.to_string()
                    }
                })
                .collect();

            return processed_lines.join("\n").trim_end().to_string();
        }

        self.clean_line_comment(text)
    }

    /// Clean string literal by removing quotes and handling escape sequences
    ///
    /// Supports:
    /// - Regular strings: "hello"
    /// - Raw strings: r"hello", r#"hello "world""#, etc.
    /// - Byte strings: b"hello"
    /// - F-strings: f"hello"
    pub fn clean_string_literal(&self, text: &str) -> String {
        trace!(
            text = %text,
            "Cleaning string literal"
        );

        let result = if text.starts_with('`') && text.ends_with('`') {
            // Handle Go raw strings: `...`
            let content = &text[1..text.len() - 1];
            content.to_string()
        } else if text.starts_with('r') && text.len() > 1 {
            // Handle raw strings: r"...", r#"..."#, r##"..."##, etc.
            self.process_raw_string(text)
        } else if text.starts_with('b') && text.len() > 1 && text.chars().nth(1) == Some('"') {
            // Handle byte strings: b"..."
            let content = &text[2..text.len() - 1];
            self.unescape(content)
        } else if text.starts_with('f') && text.len() > 1 && text.chars().nth(1) == Some('"') {
            // Handle f-strings: f"..."
            let content = &text[2..text.len() - 1];
            self.unescape(content)
        } else if text.starts_with('\'') && text.ends_with('\'') {
            // Handle single-quoted strings: '...'
            let content = &text[1..text.len() - 1];
            self.unescape(content)
        } else {
            // Regular string: remove quotes and unescape
            let text = text.trim_matches('"');
            self.unescape(text)
        };

        trace!(
            original_len = text.len(),
            cleaned_len = result.len(),
            "String literal cleaned"
        );

        result
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

    #[test]
    fn test_clean_doc_comment_with_extra_lines() {
        let processor = StringProcessor::new();

        // Test case: tree-sitter returns node text that includes code lines
        // This should only extract the doc comment content, not the code
        let text = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {";
        let result = processor.clean_doc_comment(text);
        assert_eq!(result, "乘法运算", "Should only extract doc comment, not code lines");

        // Test with multiple doc comment lines followed by code
        let text2 = "/// 第一行\n/// 第二行\npub fn add(a: i32, b: i32) -> i32 {";
        let result2 = processor.clean_doc_comment(text2);
        assert_eq!(result2, "第一行\n第二行", "Should extract all doc comment lines, not code");

        // Test with inner doc comment
        let text3 = "//! 模块文档\npub mod my_module;";
        let result3 = processor.clean_doc_comment(text3);
        assert_eq!(result3, "模块文档", "Should only extract inner doc comment");
    }

    #[test]
    fn test_multiline_outer_doc_comment() {
        let processor = StringProcessor::new();

        // Standard multiline outer doc comment
        let text = "/// 第一行\n/// 第二行\n/// 第三行";
        let result = processor.clean_doc_comment(text);
        assert_eq!(result, "第一行\n第二行\n第三行");
    }

    #[test]
    fn test_multiline_inner_doc_comment() {
        let processor = StringProcessor::new();

        // Standard multiline inner doc comment
        let text = "//! 模块文档第一行\n//! 模块文档第二行";
        let result = processor.clean_doc_comment(text);
        assert_eq!(result, "模块文档第一行\n模块文档第二行");
    }

    #[test]
    fn test_multiline_doc_with_empty_lines() {
        let processor = StringProcessor::new();

        // Multiline doc with empty lines (common in Rust documentation)
        let text = "/// 标题\n/// \n/// 内容描述";
        let result = processor.clean_doc_comment(text);
        // Empty lines after stripping /// should be preserved
        assert_eq!(result, "标题\n\n内容描述");
    }

    #[test]
    fn test_multiline_doc_with_code_example() {
        let processor = StringProcessor::new();

        // Doc comment with code example (markdown code block)
        let text = "/// 示例代码\n/// ```\n/// let x = 1;\n/// ```";
        let result = processor.clean_doc_comment(text);
        assert_eq!(result, "示例代码\n```\nlet x = 1;\n```");
    }

    #[test]
    fn test_multiline_doc_with_trailing_newline() {
        let processor = StringProcessor::new();

        // Multiline doc with trailing newline (from tree-sitter)
        let text = "/// 第一行\n/// 第二行\n";
        let result = processor.clean_doc_comment(text);
        assert_eq!(result, "第一行\n第二行");
    }
}
