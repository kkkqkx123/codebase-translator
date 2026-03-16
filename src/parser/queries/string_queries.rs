//! Predefined string extraction queries

/// Predefined string queries for various languages
pub struct StringQueries;

impl StringQueries {
    // Rust queries

    /// Rust string literal query
    pub fn rust_string() -> &'static str {
        "(string_literal) @string"
    }

    /// Rust raw string literal query
    pub fn rust_raw_string() -> &'static str {
        "(raw_string_literal) @string"
    }

    /// Rust all strings query
    pub fn rust_all() -> &'static str {
        r#"
(string_literal) @string
(raw_string_literal) @string
"#
    }

    /// Rust macro string arguments query
    pub fn rust_macro_strings() -> &'static str {
        r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))

(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (raw_string_literal) @macro_string))
"#
    }

    // Go queries

    /// Go interpreted string literal query
    pub fn go_interpreted_string() -> &'static str {
        "(interpreted_string_literal) @string"
    }

    /// Go raw string literal query
    pub fn go_raw_string() -> &'static str {
        "(raw_string_literal) @string"
    }

    /// Go all strings query
    pub fn go_all() -> &'static str {
        r#"
(interpreted_string_literal) @string
(raw_string_literal) @string
"#
    }

    // Python queries

    /// Python string query
    pub fn python_string() -> &'static str {
        "(string) @string"
    }

    // JavaScript/TypeScript queries

    /// JavaScript string query
    pub fn javascript_string() -> &'static str {
        r#"
(string) @string
(template_string) @string
"#
    }

    // Java queries

    /// Java string literal query
    pub fn java_string() -> &'static str {
        "(string_literal) @string"
    }

    // C/C++ queries

    /// C/C++ string literal query
    pub fn c_string() -> &'static str {
        "(string_literal) @string"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_queries() {
        assert!(!StringQueries::rust_string().is_empty());
        assert!(!StringQueries::rust_raw_string().is_empty());
        assert!(!StringQueries::rust_macro_strings().is_empty());
    }

    #[test]
    fn test_go_queries() {
        assert!(!StringQueries::go_interpreted_string().is_empty());
        assert!(!StringQueries::go_raw_string().is_empty());
    }
}
