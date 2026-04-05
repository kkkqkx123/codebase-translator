//! Tree-sitter queries for C#

/// C# tree-sitter queries
pub struct CSharpQueries;

impl CSharpQueries {
    /// All comments query
    pub fn all_comments() -> &'static str {
        "(comment) @comment"
    }

    /// String literal query
    pub fn string_literals() -> &'static str {
        "(string_literal) @string"
    }

    /// All string literals query (including verbatim and interpolated strings)
    pub fn all_strings() -> &'static str {
        r#"
(string_literal) @string
(verbatim_string_literal) @string
(interpolated_string_expression) @string
"#
    }

    /// Method invocation query with string arguments
    pub fn method_strings() -> &'static str {
        r#"
(invocation_expression
  function: (member_access_expression
    name: (identifier) @method_name)
  arguments: (argument_list
    (argument
      (string_literal) @method_string)))

(invocation_expression
  function: (member_access_expression
    name: (identifier) @method_name)
  arguments: (argument_list
    (argument
      (verbatim_string_literal) @method_string)))

(invocation_expression
  function: (identifier) @method_name
  arguments: (argument_list
    (argument
      (string_literal) @method_string)))

(invocation_expression
  function: (identifier) @method_name
  arguments: (argument_list
    (argument
      (verbatim_string_literal) @method_string)))
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
(invocation_expression
  function: (member_access_expression
    name: (identifier) @method_name
    (#match? @method_name "^({})$"))
  arguments: (argument_list
    (argument
      (string_literal) @method_string)))

(invocation_expression
  function: (member_access_expression
    name: (identifier) @method_name
    (#match? @method_name "^({})$"))
  arguments: (argument_list
    (argument
      (verbatim_string_literal) @method_string)))

(invocation_expression
  function: (identifier) @method_name
  (#match? @method_name "^({})$")
  arguments: (argument_list
    (argument
      (string_literal) @method_string)))

(invocation_expression
  function: (identifier) @method_name
  (#match? @method_name "^({})$")
  arguments: (argument_list
    (argument
      (verbatim_string_literal) @method_string)))
"#,
            name_pattern, name_pattern, name_pattern, name_pattern
        )
    }

    /// Throw statement query
    pub fn throw_statements() -> &'static str {
        r#"
(throw_statement
  (string_literal) @throw_string)

(throw_statement
  (verbatim_string_literal) @throw_string)

(throw_statement
  (object_creation_expression
    arguments: (argument_list
      (argument
        (string_literal) @throw_string))))

(throw_statement
  (object_creation_expression
    arguments: (argument_list
      (argument
        (verbatim_string_literal) @throw_string))))
"#
    }

    /// Assert statement query
    /// Matches: Debug.Assert(condition, "message"), Assert.IsTrue(condition, "message")
    pub fn assert_calls() -> &'static str {
        r#"
(invocation_expression
  function: (member_access_expression
    name: (identifier) @assert_method
    (#match? @assert_method "^(Assert|AssertTrue|AssertFalse|Fail)$"))
  arguments: (argument_list
    (argument
      (string_literal) @assert_string)))

(invocation_expression
  function: (member_access_expression
    name: (identifier) @assert_method
    (#match? @assert_method "^(Assert|AssertTrue|AssertFalse|Fail)$"))
  arguments: (argument_list
    (argument
      (verbatim_string_literal) @assert_string)))
"#
    }

    /// Attribute query for documentation
    pub fn doc_attributes() -> &'static str {
        r#"
(attribute
  (identifier) @attr_name
  (#match? @attr_name "^(Obsolete|GeneratedCode|SuppressMessage|Description|DisplayName)$")
  (attribute_argument_list
    (attribute_argument
      (string_literal) @attr_string)))

(attribute
  (identifier) @attr_name
  (#match? @attr_name "^(Obsolete|GeneratedCode|SuppressMessage|Description|DisplayName)$")
  (attribute_argument_list
    (attribute_argument
      (verbatim_string_literal) @attr_string)))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_c_sharp::LANGUAGE;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", CSharpQueries::all_comments());
        assert!(result.is_ok(), "All comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("string_literals", CSharpQueries::string_literals());
        assert!(result.is_ok(), "String literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", CSharpQueries::all_strings());
        assert!(result.is_ok(), "All strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_method_strings_query_syntax_valid() {
        let result = validate_query_syntax("method_strings", CSharpQueries::method_strings());
        assert!(result.is_ok(), "Method strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_specific_methods_query_syntax_valid() {
        let specific = CSharpQueries::specific_methods(&["Console.WriteLine", "Debug.Log"]);
        let result = validate_query_syntax("specific_methods", &specific);
        assert!(result.is_ok(), "Specific methods query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_throw_statements_query_syntax_valid() {
        let result = validate_query_syntax("throw_statements", CSharpQueries::throw_statements());
        assert!(result.is_ok(), "Throw statements query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_assert_calls_query_syntax_valid() {
        let result = validate_query_syntax("assert_calls", CSharpQueries::assert_calls());
        assert!(result.is_ok(), "Assert calls query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_doc_attributes_query_syntax_valid() {
        let result = validate_query_syntax("doc_attributes", CSharpQueries::doc_attributes());
        assert!(result.is_ok(), "Doc attributes query syntax validation failed: {:?}", result.err());
    }
}
