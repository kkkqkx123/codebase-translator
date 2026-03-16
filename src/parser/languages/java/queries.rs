//! Tree-sitter queries for Java

/// Java tree-sitter queries
pub struct JavaQueries;

impl JavaQueries {
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

    /// Javadoc comment query (/** ... */)
    pub fn javadoc_comments() -> &'static str {
        r#"
((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    /// String literal query
    pub fn string_literals() -> &'static str {
        "(string_literal) @string"
    }

    /// All string literals query
    pub fn all_strings() -> &'static str {
        "(string_literal) @string"
    }

    /// Method invocation with string arguments
    pub fn method_strings() -> &'static str {
        r#"
(method_invocation
  name: (identifier) @method_name
  arguments: (argument_list
    (string_literal) @method_string))
"#
    }

    /// Specific method invocation query
    pub fn specific_methods(method_names: &[&str]) -> String {
        if method_names.is_empty() {
            return Self::method_strings().to_string();
        }

        let name_pattern = method_names.join("|");
        format!(
            r#"
(method_invocation
  name: (identifier) @method_name
  (#match? @method_name "^({})$")
  arguments: (argument_list
    (string_literal) @method_string))
"#,
            name_pattern
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!JavaQueries::line_comments().is_empty());
        assert!(!JavaQueries::block_comments().is_empty());
        assert!(!JavaQueries::all_comments().is_empty());
    }

    #[test]
    fn test_javadoc_queries() {
        let javadoc_query = JavaQueries::javadoc_comments();
        // The query contains escaped regex pattern for matching Javadoc start
        assert!(javadoc_query.contains("#match?"));
        assert!(javadoc_query.contains("block_comment"));
        assert!(javadoc_query.contains("docstring"));
    }

    #[test]
    fn test_string_queries() {
        assert!(!JavaQueries::string_literals().is_empty());
        assert!(!JavaQueries::all_strings().is_empty());
    }

    #[test]
    fn test_method_queries() {
        assert!(!JavaQueries::method_strings().is_empty());

        let specific = JavaQueries::specific_methods(&["println", "print"]);
        assert!(specific.contains("println"));
        assert!(specific.contains("print"));
    }
}
