//! Predefined comment extraction queries

use tracing::debug;

/// Predefined comment queries for various languages
pub struct CommentQueries;

impl CommentQueries {
    // Rust queries

    /// Rust line comment query
    pub fn rust_line() -> &'static str {
        debug!(query_name = "rust_line", "Executing query");
        "(line_comment) @comment"
    }

    /// Rust block comment query
    pub fn rust_block() -> &'static str {
        debug!(query_name = "rust_block", "Executing query");
        "(block_comment) @comment"
    }

    /// Rust doc comment query (/// and //!)
    pub fn rust_doc() -> &'static str {
        debug!(query_name = "rust_doc", "Executing query");
        r#"
((line_comment) @docstring
  (#match? @docstring "^///"))

((line_comment) @docstring
  (#match? @docstring "^//!"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    /// Rust all comments query
    pub fn rust_all() -> &'static str {
        debug!(query_name = "rust_all", "Executing query");
        r#"
(line_comment) @comment
(block_comment) @comment
"#
    }

    // Go queries

    /// Go comment query
    pub fn go_comment() -> &'static str {
        debug!(query_name = "go_comment", "Executing query");
        "(comment) @comment"
    }

    /// Go doc comment query
    pub fn go_doc() -> &'static str {
        debug!(query_name = "go_doc", "Executing query");
        r#"
((comment) @docstring
  (#match? @docstring "^// "))
"#
    }

    // Python queries

    /// Python comment query
    pub fn python_comment() -> &'static str {
        debug!(query_name = "python_comment", "Executing query");
        "(comment) @comment"
    }

    /// Python docstring query
    pub fn python_docstring() -> &'static str {
        debug!(query_name = "python_docstring", "Executing query");
        r#"
((expression_statement (string)) @docstring)
(function_definition
  body: (block
    (expression_statement (string)) @docstring))
(class_definition
  body: (block
    (expression_statement (string)) @docstring))
"#
    }

    // JavaScript/TypeScript queries

    /// JavaScript comment query
    pub fn javascript_comment() -> &'static str {
        debug!(query_name = "javascript_comment", "Executing query");
        r#"
((comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// JavaScript JSDoc query
    pub fn javascript_jsdoc() -> &'static str {
        debug!(query_name = "javascript_jsdoc", "Executing query");
        r#"
((comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    // Java queries

    /// Java comment query
    pub fn java_comment() -> &'static str {
        debug!(query_name = "java_comment", "Executing query");
        r#"
((line_comment) @comment)
((block_comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// Java Javadoc query
    pub fn java_javadoc() -> &'static str {
        debug!(query_name = "java_javadoc", "Executing query");
        r#"
((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    // C/C++ queries

    /// C/C++ comment query
    pub fn c_comment() -> &'static str {
        debug!(query_name = "c_comment", "Executing query");
        r#"
((comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// C/C++ doc comment query
    pub fn c_doc() -> &'static str {
        debug!(query_name = "c_doc", "Executing query");
        r#"
((comment) @docstring
  (#match? @comment "^/\\*\\*"))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_queries() {
        assert!(!CommentQueries::rust_line().is_empty());
        assert!(!CommentQueries::rust_doc().is_empty());
        assert!(CommentQueries::rust_doc().contains("///"));
    }

    #[test]
    fn test_go_queries() {
        assert!(!CommentQueries::go_comment().is_empty());
        assert!(!CommentQueries::go_doc().is_empty());
    }

    #[test]
    fn test_python_queries() {
        assert!(!CommentQueries::python_comment().is_empty());
        assert!(!CommentQueries::python_docstring().is_empty());
    }
}
