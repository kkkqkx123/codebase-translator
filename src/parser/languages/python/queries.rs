//! Tree-sitter queries for Python

/// Python tree-sitter queries
pub struct PythonQueries;

impl PythonQueries {
    /// All comments query (both line and block comments)
    pub fn all_comments() -> &'static str {
        "(comment) @comment"
    }

    /// Docstring query (string literals that appear as the first statement in modules, classes, or functions)
    /// In Python, docstrings are string literals that appear as the first statement
    pub fn docstrings() -> &'static str {
        r#"
(module
  (expression_statement
    (string) @docstring))

(function_definition
  body: (block
    (expression_statement
      (string) @docstring)))

(class_definition
  body: (block
    (expression_statement
      (string) @docstring)))
"#
    }

    /// String literal query
    pub fn string_literals() -> &'static str {
        "(string) @string"
    }

    /// F-string query (formatted string literals)
    pub fn f_strings() -> &'static str {
        "(string) @fstring
(#match? @fstring \"^[fF]\")"
    }

    /// All string literals query
    pub fn all_strings() -> &'static str {
        "(string) @string"
    }

    /// Function call with string argument query
    pub fn function_call_strings() -> &'static str {
        r#"
(call
  function: (identifier) @func_name
  arguments: (argument_list
    (string) @func_string))

(call
  function: (attribute
    object: (_) @operand
    attribute: (identifier) @func_name)
  arguments: (argument_list
    (string) @func_string))
"#
    }

    /// Specific function call query
    pub fn specific_functions(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return Self::function_call_strings().to_string();
        }

        let name_pattern = func_names.join("|");
        format!(
            r#"
(call
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (string) @func_string))

(call
  function: (attribute
    attribute: (identifier) @func_name)
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (string) @func_string))
"#,
            name_pattern, name_pattern
        )
    }

    /// Raise statement query
    /// Matches: raise Exception("message"), raise ValueError("message")
    pub fn raise_statements() -> &'static str {
        r#"
(raise_statement
  (call
    arguments: (argument_list
      (string) @raise_string)))

(raise_statement
  (string) @raise_string)
"#
    }

    /// Assertion statement query
    /// Matches: assert condition, "message"
    pub fn assert_statements() -> &'static str {
        r#"
(assert_statement
  (string) @assert_string)
"#
    }

    /// Error function call query (raise, assert, etc.)
    pub fn error_functions() -> &'static str {
        r#"
(call
  function: (identifier) @func_name
  (#match? @func_name "^(raise|assert)$")
  arguments: (argument_list
    (string) @error_string))

(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @func_name)
  (#eq? @obj "sys")
  (#match? @func_name "^(exit)$")
  arguments: (argument_list
    (string) @error_string))

(raise_statement
  (call
    arguments: (argument_list
      (string) @error_string)))
"#
    }

    /// Format function call query (print, format, etc.)
    pub fn format_functions() -> &'static str {
        r#"
(call
  function: (identifier) @func_name
  (#match? @func_name "^(print|format)$")
  arguments: (argument_list
    (string) @format_string))

(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @func_name)
  (#eq? @obj "str")
  (#eq? @func_name "format")
  arguments: (argument_list
    (string) @format_string))
"#
    }

    /// Log function call query (logging.info, logger.debug, etc.)
    pub fn log_functions() -> &'static str {
        r#"
(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @func_name)
  (#match? @obj "^(logging|logger|log)$")
  (#match? @func_name "^(info|debug|warning|error|critical|exception)$")
  arguments: (argument_list
    (string) @log_string))
"#
    }

    /// Test description query
    /// Matches: pytest decorators, unittest assert methods with message
    pub fn test_descriptions() -> &'static str {
        r#"
(call
  function: (attribute
    object: (identifier) @pytest_obj
    (#match? @pytest_obj "^(pytest)$")
    attribute: (identifier) @pytest_method
    (#match? @pytest_method "^(skip|xfail)$"))
  arguments: (argument_list
    (string) @test_description))

(call
  function: (attribute
    object: (attribute
      object: (identifier) @pytest_mark
      (#eq? @pytest_mark "pytest")
      attribute: (identifier) @mark
      (#eq? @mark "mark"))
    attribute: (identifier) @pytest_method
    (#match? @pytest_method "^(skip|xfail|parametrize)$"))
  arguments: (argument_list
    (string) @test_description))

(call
  function: (attribute
    object: (identifier) @self_obj
    (#eq? @self_obj "self")
    attribute: (identifier) @assert_method
    (#match? @assert_method "^(assertEqual|assertNotEqual|assertTrue|assertFalse|assertIs|assertIsNot|assertIsNone|assertIsNotNone|assertIn|assertNotIn|assertIsInstance|assertNotIsInstance|assertGreater|assertGreaterEqual|assertLess|assertLessEqual|assertRegex|assertNotRegex|assertCountEqual|assertMultiLineEqual|assertSequenceEqual|assertListEqual|assertTupleEqual|assertSetEqual|assertDictEqual|assertAlmostEqual|assertNotAlmostEqual|assertRaises|assertRaisesRegex|assertWarns|assertWarnsRegex|fail)$"))
  arguments: (argument_list
    (string) @test_description))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_python::LANGUAGE;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", PythonQueries::all_comments());
        assert!(result.is_ok(), "All comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_docstrings_query_syntax_valid() {
        let result = validate_query_syntax("docstrings", PythonQueries::docstrings());
        assert!(result.is_ok(), "Docstrings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("string_literals", PythonQueries::string_literals());
        assert!(result.is_ok(), "String literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_f_strings_query_syntax_valid() {
        let result = validate_query_syntax("f_strings", PythonQueries::f_strings());
        assert!(result.is_ok(), "F-strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", PythonQueries::all_strings());
        assert!(result.is_ok(), "All strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_function_call_strings_query_syntax_valid() {
        let result = validate_query_syntax("function_call_strings", PythonQueries::function_call_strings());
        assert!(result.is_ok(), "Function call strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_specific_functions_query_syntax_valid() {
        let specific = PythonQueries::specific_functions(&["print", "logging.info"]);
        let result = validate_query_syntax("specific_functions", &specific);
        assert!(result.is_ok(), "Specific functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_raise_statements_query_syntax_valid() {
        let result = validate_query_syntax("raise_statements", PythonQueries::raise_statements());
        assert!(result.is_ok(), "Raise statements query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_assert_statements_query_syntax_valid() {
        let result = validate_query_syntax("assert_statements", PythonQueries::assert_statements());
        assert!(result.is_ok(), "Assert statements query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_error_functions_query_syntax_valid() {
        let result = validate_query_syntax("error_functions", PythonQueries::error_functions());
        assert!(result.is_ok(), "Error functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_format_functions_query_syntax_valid() {
        let result = validate_query_syntax("format_functions", PythonQueries::format_functions());
        assert!(result.is_ok(), "Format functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_log_functions_query_syntax_valid() {
        let result = validate_query_syntax("log_functions", PythonQueries::log_functions());
        assert!(result.is_ok(), "Log functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_test_descriptions_query_syntax_valid() {
        let result = validate_query_syntax("test_descriptions", PythonQueries::test_descriptions());
        assert!(result.is_ok(), "Test descriptions query syntax validation failed: {:?}", result.err());
    }
}
