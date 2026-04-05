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

    /// All comments query (excluding Javadoc comments)
    pub fn all_comments() -> &'static str {
        r#"
(line_comment) @comment

((block_comment) @comment
  (#not-match? @comment "^/\\*\\*"))
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

    /// Throw statement query
    /// Matches: throw new Exception("message"), throw new RuntimeException("message")
    pub fn throw_statements() -> &'static str {
        r#"
(throw_statement
  (object_creation_expression
    arguments: (argument_list
      (string_literal) @throw_string)))
"#
    }

    /// Assert statement query
    /// Matches: assert condition : "message";
    pub fn assert_statements() -> &'static str {
        r#"
(assert_statement
  (string_literal) @assert_string)
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_java::LANGUAGE;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_line_comments_query_syntax_valid() {
        let result = validate_query_syntax("line_comments", JavaQueries::line_comments());
        assert!(result.is_ok(), "Line comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_block_comments_query_syntax_valid() {
        let result = validate_query_syntax("block_comments", JavaQueries::block_comments());
        assert!(result.is_ok(), "Block comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", JavaQueries::all_comments());
        assert!(result.is_ok(), "All comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_javadoc_comments_query_syntax_valid() {
        let result = validate_query_syntax("javadoc_comments", JavaQueries::javadoc_comments());
        assert!(result.is_ok(), "Javadoc comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("string_literals", JavaQueries::string_literals());
        assert!(result.is_ok(), "String literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", JavaQueries::all_strings());
        assert!(result.is_ok(), "All strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_method_strings_query_syntax_valid() {
        let result = validate_query_syntax("method_strings", JavaQueries::method_strings());
        assert!(result.is_ok(), "Method strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_specific_methods_query_syntax_valid() {
        let specific = JavaQueries::specific_methods(&["println", "print"]);
        let result = validate_query_syntax("specific_methods", &specific);
        assert!(result.is_ok(), "Specific methods query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_throw_statements_query_syntax_valid() {
        let result = validate_query_syntax("throw_statements", JavaQueries::throw_statements());
        assert!(result.is_ok(), "Throw statements query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_assert_statements_query_syntax_valid() {
        let result = validate_query_syntax("assert_statements", JavaQueries::assert_statements());
        assert!(result.is_ok(), "Assert statements query syntax validation failed: {:?}", result.err());
    }
}
