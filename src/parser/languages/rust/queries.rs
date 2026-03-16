//! Tree-sitter queries for Rust

/// Rust tree-sitter queries
pub struct RustQueries;

impl RustQueries {
    /// Line comment query
    pub fn line_comments() -> &'static str {
        "(line_comment) @comment"
    }

    /// Block comment query
    pub fn block_comments() -> &'static str {
        "(block_comment) @comment"
    }

    /// All comments query
    pub fn all_comments() -> &'static str {
        r#"
(line_comment) @comment
(block_comment) @comment
"#
    }

    /// Doc comment query (/// and //!)
    pub fn doc_comments() -> &'static str {
        r#"
((line_comment) @docstring
  (#match? @docstring "^///"))

((line_comment) @docstring
  (#match? @docstring "^//!"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    /// Outer doc comments only (///)
    pub fn outer_doc_comments() -> &'static str {
        r#"
((line_comment) @docstring
  (#match? @docstring "^///"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    /// Inner doc comments only (!//)
    pub fn inner_doc_comments() -> &'static str {
        r#"
((line_comment) @docstring
  (#match? @docstring "^//!"))
"#
    }

    /// String literal query
    pub fn string_literals() -> &'static str {
        "(string_literal) @string"
    }

    /// Raw string literal query
    pub fn raw_string_literals() -> &'static str {
        "(raw_string_literal) @string"
    }

    /// All string literals query
    pub fn all_strings() -> &'static str {
        r#"
(string_literal) @string
(raw_string_literal) @string
"#
    }

    /// Macro invocation query with string arguments
    pub fn macro_strings() -> &'static str {
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

    /// Specific macro invocation query
    pub fn specific_macros(macro_names: &[&str]) -> String {
        if macro_names.is_empty() {
            return Self::macro_strings().to_string();
        }

        let name_pattern = macro_names.join("|");
        format!(
            r#"
(macro_invocation
  macro: (identifier) @macro_name
  (#match? @macro_name "^({})$")
  (token_tree
    (string_literal) @macro_string))

(macro_invocation
  macro: (identifier) @macro_name
  (#match? @macro_name "^({})$")
  (token_tree
    (raw_string_literal) @macro_string))
"#,
            name_pattern, name_pattern
        )
    }

    /// Attribute query
    pub fn attributes() -> &'static str {
        r#"
(attribute
  (identifier) @attr_name
  arguments: (token_tree
    (string_literal) @attr_value)?)
"#
    }

    /// Doc attribute query (#[doc = "..."])
    pub fn doc_attributes() -> &'static str {
        r#"
(attribute
  (identifier) @attr_name
  (#eq? @attr_name "doc")
  arguments: (token_tree
    (string_literal) @doc_value))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!RustQueries::line_comments().is_empty());
        assert!(!RustQueries::block_comments().is_empty());
        assert!(!RustQueries::doc_comments().is_empty());
    }

    #[test]
    fn test_string_queries() {
        assert!(!RustQueries::string_literals().is_empty());
        assert!(!RustQueries::raw_string_literals().is_empty());
    }

    #[test]
    fn test_macro_queries() {
        assert!(!RustQueries::macro_strings().is_empty());

        let specific = RustQueries::specific_macros(&["panic", "println"]);
        assert!(specific.contains("panic"));
        assert!(specific.contains("println"));
    }

    #[test]
    fn test_doc_comment_patterns() {
        let doc_query = RustQueries::doc_comments();
        assert!(doc_query.contains("///"));
        assert!(doc_query.contains("//!"));
    }
}
