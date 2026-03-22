//! Tree-sitter queries for C++

/// C++ tree-sitter queries
pub struct CppQueries;

impl CppQueries {
    /// All comments query (excluding documentation comments)
    pub fn all_comments() -> &'static str {
        r#"
((comment) @comment
  (#not-match? @comment "^//[/!]")
  (#not-match? @comment "^/\\*[*!]"))
"#
    }

    /// Documentation comments query (Doxygen style: /// or /**)
    pub fn doc_comments() -> &'static str {
        r#"
((comment) @docstring
  (#match? @docstring "^//[/!]"))

((comment) @docstring
  (#match? @docstring "^/\\*[*!]"))
"#
    }

    /// String literal query
    pub fn string_literals() -> &'static str {
        "(string_literal) @string"
    }

    /// All string literals query (including raw strings)
    pub fn all_strings() -> &'static str {
        r#"
(string_literal) @string
(concatenated_string) @string
(raw_string_literal) @string
"#
    }

    /// Function call query with string arguments
    pub fn function_strings() -> &'static str {
        r#"
(call_expression
  function: (identifier) @func_name
  arguments: (argument_list
    (string_literal) @func_string))

(call_expression
  function: (identifier) @func_name
  arguments: (argument_list
    (concatenated_string) @func_string))

(call_expression
  function: (identifier) @func_name
  arguments: (argument_list
    (raw_string_literal) @func_string))

(call_expression
  function: (field_expression
    field: (field_identifier) @func_name)
  arguments: (argument_list
    (string_literal) @func_string))
"#
    }

    /// Specific function call query
    pub fn specific_functions(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return Self::function_strings().to_string();
        }

        let name_pattern = func_names.join("|");
        format!(
            r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (string_literal) @func_string))

(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (concatenated_string) @func_string))

(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (raw_string_literal) @func_string))
"#,
            name_pattern, name_pattern, name_pattern
        )
    }

    /// Throw statement query
    pub fn throw_statements() -> &'static str {
        r#"
(throw_statement
  (string_literal) @throw_string)

(throw_statement
  (concatenated_string) @throw_string)
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!CppQueries::all_comments().is_empty());
    }

    #[test]
    fn test_string_queries() {
        assert!(!CppQueries::string_literals().is_empty());
        assert!(!CppQueries::all_strings().is_empty());
    }

    #[test]
    fn test_function_queries() {
        assert!(!CppQueries::function_strings().is_empty());

        let specific = CppQueries::specific_functions(&["std::cout", "printf"]);
        assert!(specific.contains("printf"));
    }

    #[test]
    fn test_throw_queries() {
        assert!(!CppQueries::throw_statements().is_empty());
    }
}
