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

    #[test]
    fn test_comment_queries() {
        assert!(!CSharpQueries::all_comments().is_empty());
    }

    #[test]
    fn test_string_queries() {
        assert!(!CSharpQueries::string_literals().is_empty());
        assert!(!CSharpQueries::all_strings().is_empty());
    }

    #[test]
    fn test_method_queries() {
        assert!(!CSharpQueries::method_strings().is_empty());

        let specific = CSharpQueries::specific_methods(&["Console.WriteLine", "Debug.Log"]);
        assert!(specific.contains("Console.WriteLine"));
        assert!(specific.contains("Debug.Log"));
    }

    #[test]
    fn test_throw_queries() {
        assert!(!CSharpQueries::throw_statements().is_empty());
    }

    #[test]
    fn test_attribute_queries() {
        assert!(!CSharpQueries::doc_attributes().is_empty());
    }
}
