//! Predefined comment extraction queries

/// Predefined comment queries for various languages
pub struct CommentQueries;

impl CommentQueries {
    // Rust queries

    /// Rust line comment query
    pub fn rust_line() -> &'static str {
        "(line_comment) @comment"
    }

    /// Rust block comment query
    pub fn rust_block() -> &'static str {
        "(block_comment) @comment"
    }

    /// Rust doc comment query (/// and //!)
    pub fn rust_doc() -> &'static str {
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
        r#"
(line_comment) @comment
(block_comment) @comment
"#
    }

    // Go queries

    /// Go comment query
    pub fn go_comment() -> &'static str {
        "(comment) @comment"
    }

    /// Go doc comment query
    pub fn go_doc() -> &'static str {
        r#"
((comment) @docstring
  (#match? @docstring "^// "))
"#
    }

    // Python queries

    /// Python comment query
    pub fn python_comment() -> &'static str {
        "(comment) @comment"
    }

    /// Python docstring query
    pub fn python_docstring() -> &'static str {
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
        r#"
((comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// JavaScript JSDoc query
    pub fn javascript_jsdoc() -> &'static str {
        r#"
((comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    // Java queries

    /// Java comment query
    pub fn java_comment() -> &'static str {
        r#"
((line_comment) @comment)
((block_comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// Java Javadoc query
    pub fn java_javadoc() -> &'static str {
        r#"
((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    // C/C++ queries

    /// C/C++ comment query
    pub fn c_comment() -> &'static str {
        r#"
((comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// C/C++ doc comment query
    pub fn c_doc() -> &'static str {
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
