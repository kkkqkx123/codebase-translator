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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!PythonQueries::all_comments().is_empty());
    }

    #[test]
    fn test_docstring_queries() {
        assert!(!PythonQueries::docstrings().is_empty());
    }

    #[test]
    fn test_string_queries() {
        assert!(!PythonQueries::string_literals().is_empty());
        assert!(!PythonQueries::f_strings().is_empty());
        assert!(!PythonQueries::all_strings().is_empty());
    }

    #[test]
    fn test_function_queries() {
        assert!(!PythonQueries::function_call_strings().is_empty());

        let specific = PythonQueries::specific_functions(&["print", "logging.info"]);
        assert!(specific.contains("print"));
        assert!(specific.contains("logging.info"));
    }

    #[test]
    fn test_error_function_queries() {
        assert!(!PythonQueries::error_functions().is_empty());
    }

    #[test]
    fn test_format_function_queries() {
        assert!(!PythonQueries::format_functions().is_empty());
    }

    #[test]
    fn test_log_function_queries() {
        assert!(!PythonQueries::log_functions().is_empty());
    }
}
