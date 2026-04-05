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

    /// All comments query (excluding doc comments)
    pub fn all_comments() -> &'static str {
        r#"
((line_comment) @comment
  (#not-match? @comment "^///")
  (#not-match? @comment "^//!"))

((block_comment) @comment
  (#not-match? @comment "^/\\*\\*"))
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

    /// Panic macro query
    /// Matches: panic!("message"), panic!("format {}", arg)
    pub fn panic_macros() -> &'static str {
        r#"
(macro_invocation
  macro: (identifier) @panic_name
  (#eq? @panic_name "panic")
  (token_tree
    (string_literal) @panic_string))

(macro_invocation
  macro: (identifier) @panic_name
  (#eq? @panic_name "panic")
  (token_tree
    (raw_string_literal) @panic_string))
"#
    }

    /// Assertion macro query
    /// Matches: assert!(cond, "message"), assert_eq!(a, b, "message"), assert_ne!(a, b, "message")
    pub fn assertion_macros() -> &'static str {
        r#"
(macro_invocation
  macro: (identifier) @assert_name
  (#match? @assert_name "^(assert|assert_eq|assert_ne|debug_assert|debug_assert_eq|debug_assert_ne)$")
  (token_tree
    (string_literal) @assert_string))

(macro_invocation
  macro: (identifier) @assert_name
  (#match? @assert_name "^(assert|assert_eq|assert_ne|debug_assert|debug_assert_eq|debug_assert_ne)$")
  (token_tree
    (raw_string_literal) @assert_string))
"#
    }

    /// Unimplemented macro query
    /// Matches: unimplemented!("message"), todo!("message")
    pub fn unimplemented_macros() -> &'static str {
        r#"
(macro_invocation
  macro: (identifier) @unimpl_name
  (#match? @unimpl_name "^(unimplemented|todo)$")
  (token_tree
    (string_literal) @unimpl_string))

(macro_invocation
  macro: (identifier) @unimpl_name
  (#match? @unimpl_name "^(unimplemented|todo)$")
  (token_tree
    (raw_string_literal) @unimpl_string))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_rust::LANGUAGE;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_line_comments_query_syntax_valid() {
        let result = validate_query_syntax("line_comments", RustQueries::line_comments());
        assert!(
            result.is_ok(),
            "Line comments query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_block_comments_query_syntax_valid() {
        let result = validate_query_syntax("block_comments", RustQueries::block_comments());
        assert!(
            result.is_ok(),
            "Block comments query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", RustQueries::all_comments());
        assert!(
            result.is_ok(),
            "All comments query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_doc_comments_query_syntax_valid() {
        let result = validate_query_syntax("doc_comments", RustQueries::doc_comments());
        assert!(
            result.is_ok(),
            "Doc comments query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_outer_doc_comments_query_syntax_valid() {
        let result = validate_query_syntax("outer_doc_comments", RustQueries::outer_doc_comments());
        assert!(
            result.is_ok(),
            "Outer doc comments query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_inner_doc_comments_query_syntax_valid() {
        let result = validate_query_syntax("inner_doc_comments", RustQueries::inner_doc_comments());
        assert!(
            result.is_ok(),
            "Inner doc comments query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("string_literals", RustQueries::string_literals());
        assert!(
            result.is_ok(),
            "String literals query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_raw_string_literals_query_syntax_valid() {
        let result =
            validate_query_syntax("raw_string_literals", RustQueries::raw_string_literals());
        assert!(
            result.is_ok(),
            "Raw string literals query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", RustQueries::all_strings());
        assert!(
            result.is_ok(),
            "All strings query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_macro_strings_query_syntax_valid() {
        let result = validate_query_syntax("macro_strings", RustQueries::macro_strings());
        assert!(
            result.is_ok(),
            "Macro strings query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_specific_macros_query_syntax_valid() {
        let specific = RustQueries::specific_macros(&["panic", "println"]);
        let result = validate_query_syntax("specific_macros", &specific);
        assert!(
            result.is_ok(),
            "Specific macros query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_attributes_query_syntax_valid() {
        let result = validate_query_syntax("attributes", RustQueries::attributes());
        assert!(
            result.is_ok(),
            "Attributes query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_doc_attributes_query_syntax_valid() {
        let result = validate_query_syntax("doc_attributes", RustQueries::doc_attributes());
        assert!(
            result.is_ok(),
            "Doc attributes query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_panic_macros_query_syntax_valid() {
        let result = validate_query_syntax("panic_macros", RustQueries::panic_macros());
        assert!(
            result.is_ok(),
            "Panic macros query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_assertion_macros_query_syntax_valid() {
        let result = validate_query_syntax("assertion_macros", RustQueries::assertion_macros());
        assert!(
            result.is_ok(),
            "Assertion macros query syntax validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_unimplemented_macros_query_syntax_valid() {
        let result =
            validate_query_syntax("unimplemented_macros", RustQueries::unimplemented_macros());
        assert!(
            result.is_ok(),
            "Unimplemented macros query syntax validation failed: {:?}",
            result.err()
        );
    }
}
