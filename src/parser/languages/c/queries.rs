//! Tree-sitter queries for C

/// C tree-sitter queries
pub struct CQueries;

impl CQueries {
    /// Line comment query
    pub fn line_comments() -> &'static str {
        "(comment) @comment"
    }

    /// All comments query
    pub fn all_comments() -> &'static str {
        "(comment) @comment"
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

    /// All string literals query (including wide strings)
    pub fn all_strings() -> &'static str {
        r#"
(string_literal) @string
(concatenated_string) @string
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
"#,
            name_pattern, name_pattern
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!CQueries::line_comments().is_empty());
        assert!(!CQueries::all_comments().is_empty());
    }

    #[test]
    fn test_string_queries() {
        assert!(!CQueries::string_literals().is_empty());
        assert!(!CQueries::all_strings().is_empty());
    }

    #[test]
    fn test_function_queries() {
        assert!(!CQueries::function_strings().is_empty());

        let specific = CQueries::specific_functions(&["printf", "fprintf"]);
        assert!(specific.contains("printf"));
        assert!(specific.contains("fprintf"));
    }
}

