//! Tree-sitter queries for C

/// C tree-sitter queries
pub struct CQueries;

impl CQueries {
    /// Line comment query
    pub fn line_comments() -> &'static str {
        "(comment) @comment"
    }

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

    /// Assert macro call query
    /// Matches: assert(condition && "message"), assert(condition)
    pub fn assert_calls() -> &'static str {
        r#"
(call_expression
  function: (identifier) @assert_name
  (#eq? @assert_name "assert")
  arguments: (argument_list
    (string_literal) @assert_string))
"#
    }

    /// Error function call query (errx, err, warnx, warn)
    pub fn error_functions() -> &'static str {
        r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^(err|errx|warn|warnx|error)$")
  arguments: (argument_list
    (string_literal) @error_string))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_c::LANGUAGE;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_line_comments_query_syntax_valid() {
        let result = validate_query_syntax("line_comments", CQueries::line_comments());
        assert!(result.is_ok(), "Line comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", CQueries::all_comments());
        assert!(result.is_ok(), "All comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_doc_comments_query_syntax_valid() {
        let result = validate_query_syntax("doc_comments", CQueries::doc_comments());
        assert!(result.is_ok(), "Doc comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("string_literals", CQueries::string_literals());
        assert!(result.is_ok(), "String literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", CQueries::all_strings());
        assert!(result.is_ok(), "All strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_function_strings_query_syntax_valid() {
        let result = validate_query_syntax("function_strings", CQueries::function_strings());
        assert!(result.is_ok(), "Function strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_specific_functions_query_syntax_valid() {
        let specific = CQueries::specific_functions(&["printf", "fprintf"]);
        let result = validate_query_syntax("specific_functions", &specific);
        assert!(result.is_ok(), "Specific functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_assert_calls_query_syntax_valid() {
        let result = validate_query_syntax("assert_calls", CQueries::assert_calls());
        assert!(result.is_ok(), "Assert calls query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_error_functions_query_syntax_valid() {
        let result = validate_query_syntax("error_functions", CQueries::error_functions());
        assert!(result.is_ok(), "Error functions query syntax validation failed: {:?}", result.err());
    }
}
