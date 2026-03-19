//! String processing utilities

use crate::core::models::{CommentStyle, FormatInfo, FormatPlaceholder, StringStyle};
use tracing::trace;

/// Result of cleaning a string literal, including the cleaned text and format info
#[derive(Debug, Clone)]
pub struct CleanedString {
    /// The cleaned text content (without quotes and escapes)
    pub text: String,
    /// Format information for preserving the original formatting
    pub format_info: FormatInfo,
    /// Extracted format placeholders
    pub placeholders: Vec<FormatPlaceholder>,
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

/// Result of cleaning a comment, including the cleaned text and format info
#[derive(Debug, Clone)]
pub struct CleanedComment {
    /// The cleaned text content (without comment markers)
    pub text: String,
    /// Format information for preserving the original formatting
    pub format_info: FormatInfo,
}

/// String processor for cleaning and transforming string literals
pub struct StringProcessor;

impl StringProcessor {
    /// Create a new string processor
    pub fn new() -> Self {
        Self
    }

    /// Clean comment content by removing comment markers (legacy method)
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

    /// Clean comment and extract format information
    ///
    /// This method returns both the cleaned text and format information
    /// needed to reconstruct the comment with proper formatting.
    pub fn clean_comment_with_format(
        &self,
        text: &str,
        comment_type: CommentType,
    ) -> CleanedComment {
        trace!(
            text = %text,
            comment_type = ?comment_type,
            "Cleaning comment"
        );

        let result = match comment_type {
            CommentType::Line => self.clean_line_comment_with_format(text),
            CommentType::Block => self.clean_block_comment_with_format(text),
            CommentType::Doc => self.clean_doc_comment_with_format(text),
        };

        trace!(
            original_len = text.len(),
            cleaned_len = result.text.len(),
            "Comment cleaned"
        );

        result
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

    /// Clean line comment and extract format information
    fn clean_line_comment_with_format(&self, text: &str) -> CleanedComment {
        // Extract base indentation
        let base_indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();
        let trimmed = text.trim_start();

        // Handle Python/Ruby/YAML style: #
        if let Some(content) = trimmed.strip_prefix('#') {
            return CleanedComment {
                text: content.trim_start().to_string(),
                format_info: FormatInfo::line_comment(base_indent),
            };
        }

        // Handle C-style: //
        if let Some(content) = trimmed.strip_prefix("//") {
            return CleanedComment {
                text: content.trim_start().to_string(),
                format_info: FormatInfo::line_comment(base_indent),
            };
        }

        CleanedComment {
            text: trimmed.to_string(),
            format_info: FormatInfo::line_comment(base_indent),
        }
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

    /// Clean block comment and extract format information
    fn clean_block_comment_with_format(&self, text: &str) -> CleanedComment {
        // Extract base indentation from the first line
        let base_indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();

        // Only trim leading/trailing whitespace on the outer edges, not internal newlines
        let trimmed = text
            .trim_start()
            .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

        // Remove /* and */
        let content = trimmed
            .strip_prefix("/*")
            .and_then(|s| s.strip_suffix("*/"))
            .unwrap_or(trimmed);

        // Check if this is a multi-line comment with asterisk prefixes
        let lines: Vec<&str> = content.lines().collect();
        let is_multiline = lines.len() > 1;

        // Detect line prefix pattern (e.g., " * " in Javadoc)
        let line_prefix = if is_multiline {
            lines
                .iter()
                .skip(1) // Skip first line (after /*)
                .filter(|line| !line.trim().is_empty())
                .find_map(|line| {
                    let trimmed_line = line.trim_start();
                    if trimmed_line.starts_with('*') {
                        // Calculate the prefix: indentation + " * "
                        let indent = line.len() - trimmed_line.len();
                        Some(line[..indent + 1].to_string()) // include the '*'
                    } else {
                        None
                    }
                })
        } else {
            None
        };

        // Process each line: trim whitespace and remove leading '*' if present
        let processed_lines: Vec<String> = lines
            .iter()
            .map(|line| {
                let trimmed_line = line.trim_start();
                if trimmed_line.starts_with('*') {
                    trimmed_line[1..].trim_start().to_string()
                } else {
                    trimmed_line.to_string()
                }
            })
            .collect();

        let cleaned_text = processed_lines.join("\n").trim_end().to_string();

        let format_info = if is_multiline && line_prefix.is_some() {
            FormatInfo::block_multi(base_indent, line_prefix.unwrap())
        } else {
            FormatInfo::block_single(base_indent)
        };

        CleanedComment {
            text: cleaned_text,
            format_info,
        }
    }

    /// Clean documentation comment (/// or //! or /**)
    ///
    /// For line-based doc comments (/// and //!), processes each line separately
    /// to preserve newlines while removing the doc comment markers.
    /// For block doc comments (/**), delegates to block comment cleaning.
    fn clean_doc_comment(&self, text: &str) -> String {
        let text = text
            .trim_start()
            .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

        // Handle Rust outer doc: ///
        if text.starts_with("///") {
            return text
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    trimmed
                        .strip_prefix("///")
                        .map(|s| s.trim_start())
                        .unwrap_or(trimmed)
                })
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Handle Rust inner doc: //!
        if text.starts_with("//!") {
            return text
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    trimmed
                        .strip_prefix("//!")
                        .map(|s| s.trim_start())
                        .unwrap_or(trimmed)
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

    /// Clean doc comment and extract format information
    fn clean_doc_comment_with_format(&self, text: &str) -> CleanedComment {
        // Extract base indentation
        let base_indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();

        let trimmed = text
            .trim_start()
            .trim_end_matches(|c: char| c.is_whitespace() && c != '\n');

        // Handle Rust outer doc: ///
        if trimmed.starts_with("///") {
            let cleaned = trimmed
                .lines()
                .map(|line| {
                    let line_trimmed = line.trim_start();
                    line_trimmed
                        .strip_prefix("///")
                        .map(|s| s.trim_start())
                        .unwrap_or(line_trimmed)
                })
                .collect::<Vec<_>>()
                .join("\n");

            return CleanedComment {
                text: cleaned,
                format_info: FormatInfo {
                    style: CommentStyle::DocOuter,
                    base_indent,
                    line_prefix: Some("/// ".to_string()),
                    ends_with_newline: false,
                    is_multiline: false,
                    string_style: None,
                    placeholders: None,
                    quote_char: None,
                },
            };
        }

        // Handle Rust inner doc: //!
        if trimmed.starts_with("//!") {
            let cleaned = trimmed
                .lines()
                .map(|line| {
                    let line_trimmed = line.trim_start();
                    line_trimmed
                        .strip_prefix("//!")
                        .map(|s| s.trim_start())
                        .unwrap_or(line_trimmed)
                })
                .collect::<Vec<_>>()
                .join("\n");

            return CleanedComment {
                text: cleaned,
                format_info: FormatInfo {
                    style: CommentStyle::DocInner,
                    base_indent,
                    line_prefix: Some("//! ".to_string()),
                    ends_with_newline: false,
                    is_multiline: false,
                    string_style: None,
                    placeholders: None,
                    quote_char: None,
                },
            };
        }

        // Handle block doc comments: /** */
        if trimmed.starts_with("/**") {
            let content = trimmed
                .strip_prefix("/**")
                .and_then(|s| s.strip_suffix("*/"))
                .unwrap_or(trimmed);

            let lines: Vec<&str> = content.lines().collect();
            let is_multiline = lines.len() > 1;

            // Detect line prefix pattern
            let line_prefix = if is_multiline {
                lines
                    .iter()
                    .skip(1)
                    .filter(|line| !line.trim().is_empty())
                    .find_map(|line| {
                        let trimmed_line = line.trim_start();
                        if trimmed_line.starts_with('*') {
                            let indent = line.len() - trimmed_line.len();
                            Some(line[..indent + 1].to_string())
                        } else {
                            None
                        }
                    })
            } else {
                None
            };

            let processed_lines: Vec<String> = lines
                .iter()
                .map(|line| {
                    let trimmed_line = line.trim_start();
                    if trimmed_line.starts_with('*') {
                        trimmed_line[1..].trim_start().to_string()
                    } else {
                        trimmed_line.to_string()
                    }
                })
                .collect();

            let cleaned_text = processed_lines.join("\n").trim_end().to_string();

            let format_info = if is_multiline && line_prefix.is_some() {
                FormatInfo {
                    style: CommentStyle::DocBlock,
                    base_indent,
                    line_prefix,
                    ends_with_newline: false,
                    is_multiline: true,
                    string_style: None,
                    placeholders: None,
                    quote_char: None,
                }
            } else {
                FormatInfo {
                    style: CommentStyle::DocBlock,
                    base_indent,
                    line_prefix: None,
                    ends_with_newline: false,
                    is_multiline: false,
                    string_style: None,
                    placeholders: None,
                    quote_char: None,
                }
            };

            return CleanedComment {
                text: cleaned_text,
                format_info,
            };
        }

        // Fall back to line comment handling
        self.clean_line_comment_with_format(text)
    }

    /// Clean string literal by removing quotes and handling escape sequences
    ///
    /// Supports:
    /// - Regular strings: "hello"
    /// - Raw strings: r"hello", r#"hello "world""#, etc.
    pub fn clean_string_literal(&self, text: &str) -> String {
        trace!(
            text = %text,
            "Cleaning string literal"
        );

        let result = if text.starts_with('`') && text.ends_with('`') {
            // Handle Go raw strings: `...`
            let content = &text[1..text.len() - 1];
            content.to_string()
        } else if text.starts_with('r') {
            // Handle raw strings: r"...", r#"..."#, r##"..."##, etc.
            self.process_raw_string(text)
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

    /// Clean string literal and extract format information
    ///
    /// This method extracts the pure text content while preserving format information
    /// needed to reconstruct the original string literal. Note: Only the outermost
    /// structure is guaranteed to be correct; complex internal structures are left
    /// to the translator to handle, otherwise code complexity would become unmanageable.
    ///
    /// Supports:
    /// - Regular strings: "hello"
    /// - Raw strings: r"hello", r#"hello "world""#
    /// - Byte strings: b"hello"
    /// - Formatted strings: f"hello {name}"
    /// - Go raw strings: `hello`
    pub fn clean_string_literal_with_format(&self, text: &str) -> CleanedString {
        trace!(text = %text, "Cleaning string literal with format");

        // Extract base indentation
        let base_indent: String = text.chars().take_while(|c| c.is_whitespace()).collect();
        let trimmed = text.trim_start();

        // Detect string style and extract content
        let (string_style, quote_char, content) = self.detect_and_extract_string(trimmed);

        // Process escape sequences (except for raw strings)
        let cleaned_text = match string_style {
            StringStyle::Raw { .. } | StringStyle::Backtick => content.to_string(),
            _ => self.unescape(&content),
        };

        // Extract format placeholders
        let placeholders = self.extract_placeholders(&cleaned_text, &string_style);

        // Build format info
        let format_info = FormatInfo {
            style: CommentStyle::Line, // Strings use Line as base style
            base_indent,
            line_prefix: None,
            ends_with_newline: false,
            is_multiline: cleaned_text.contains('\n'),
            string_style: Some(string_style),
            placeholders: Some(placeholders.clone()),
            quote_char: Some(quote_char),
        };

        trace!(
            original_len = text.len(),
            cleaned_len = cleaned_text.len(),
            style = ?string_style,
            "String literal cleaned with format"
        );

        CleanedString {
            text: cleaned_text,
            format_info,
            placeholders,
        }
    }

    /// Detect string style and extract content
    ///
    /// Returns: (string_style, quote_char, content)
    fn detect_and_extract_string(&self, text: &str) -> (StringStyle, char, String) {
        // Check for prefixed strings (r", b", f", etc.)
        if text.len() >= 2 {
            let prefix = text.chars().next().unwrap();
            let second_char = text.chars().nth(1).unwrap();

            match prefix {
                'r' | 'R' => {
                    // Raw string: r"...", r#"..."#
                    let hash_count = text[1..].chars().take_while(|&c| c == '#').count() as u8;
                    let quote_start = 1 + hash_count as usize;
                    
                    if text.chars().nth(quote_start) == Some('"') {
                        let content_start = quote_start + 1;
                        let end_pattern = format!("\"{}", "#".repeat(hash_count as usize));
                        
                        if let Some(content_end) = text[content_start..].find(&end_pattern) {
                            let content = text[content_start..content_start + content_end].to_string();
                            return (StringStyle::Raw { hash_count }, '"', content);
                        }
                    }
                }
                'b' | 'B' if second_char == '"' || second_char == '\'' => {
                    // Byte string: b"...", b'...'
                    let quote = second_char;
                    let content = text[2..text.len() - 1].to_string();
                    return (StringStyle::ByteString, quote, content);
                }
                'f' | 'F' if second_char == '"' || second_char == '\'' => {
                    // Formatted string: f"...", f'...'
                    let quote = second_char;
                    let content = text[2..text.len() - 1].to_string();
                    return (StringStyle::Formatted, quote, content);
                }
                _ => {}
            }
        }

        // Check for backtick strings (Go raw strings or JS template strings)
        if text.starts_with('`') && text.ends_with('`') {
            let content = text[1..text.len() - 1].to_string();
            // Check if it contains ${...} pattern (JS template)
            if content.contains("${") {
                return (StringStyle::Template, '`', content);
            } else {
                return (StringStyle::Backtick, '`', content);
            }
        }

        // Check for single-quoted strings
        if text.starts_with('\'') && text.ends_with('\'') {
            let content = text[1..text.len() - 1].to_string();
            return (StringStyle::SingleQuoted, '\'', content);
        }

        // Default: double-quoted string
        let content = if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
            text[1..text.len() - 1].to_string()
        } else {
            text.to_string()
        };
        (StringStyle::DoubleQuoted, '"', content)
    }

    /// Extract format placeholders from string content
    ///
    /// Note: Only the outermost structure is guaranteed to be correct.
    /// Complex internal structures are left to the translator to handle.
    fn extract_placeholders(&self, text: &str, style: &StringStyle) -> Vec<FormatPlaceholder> {
        let mut placeholders = Vec::new();

        match style {
            StringStyle::Formatted => {
                // Python f-string: {name} or {name!r} or {name:.2f}
                // Note: Simple regex matching, complex nested braces are not handled
                if let Ok(re) = regex::Regex::new(r"\{([^{}]+)\}") {
                    for cap in re.captures_iter(text) {
                        placeholders.push(FormatPlaceholder::FString(cap[1].to_string()));
                    }
                }
            }
            StringStyle::Template => {
                // JS template: ${name}
                if let Ok(re) = regex::Regex::new(r"\$\{([^{}]+)\}") {
                    for cap in re.captures_iter(text) {
                        placeholders.push(FormatPlaceholder::JSTemplate(cap[1].to_string()));
                    }
                }
            }
            _ => {
                // Check for printf-style placeholders: %s, %d, %f, etc.
                // Note: Simple pattern matching, complex format specs are not handled
                if let Ok(re) = regex::Regex::new(r"%[sdifoxXeEcgG%]") {
                    for mat in re.find_iter(text) {
                        placeholders.push(FormatPlaceholder::CStyle(mat.as_str().to_string()));
                    }
                }

                // Check for Rust-style placeholders: {}
                if let Ok(re) = regex::Regex::new(r"\{\}") {
                    for mat in re.find_iter(text) {
                        placeholders.push(FormatPlaceholder::RustStyle(mat.as_str().to_string()));
                    }
                }
            }
        }

        placeholders
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
