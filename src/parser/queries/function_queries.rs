//! Predefined function call extraction queries

/// Predefined function call queries for various languages
pub struct FunctionQueries;

impl FunctionQueries {
    /// Build a Rust macro invocation query for specific macros
    pub fn rust_macros(macro_names: &[&str]) -> String {
        if macro_names.is_empty() {
            return Self::rust_all_macros().to_string();
        }

        let name_list = macro_names.join("|");
        format!(
            r#"
(macro_invocation
  macro: (identifier) @macro_name
  (#match? @macro_name "^({})$")
  (token_tree
    (string_literal) @macro_string))

(macro_invocation
  macro: (identifier) @macro_name
  (#match? @macro_name "^({})$")
  (token_tree
    (raw_string_literal) @macro_string))
"#,
            name_list, name_list
        )
    }

    /// Query for all Rust macros with string arguments
    pub fn rust_all_macros() -> &'static str {
        r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (string_literal) @macro_string))

(macro_invocation
  macro: (identifier) @macro_name
  (token_tree
    (raw_string_literal) @macro_string))
"#
    }

    /// Build a Go function call query for specific functions
    pub fn go_functions(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return String::new();
        }

        let name_list = func_names.join("|");
        format!(
            r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (interpreted_string_literal) @func_string))

(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (raw_string_literal) @func_string))
"#,
            name_list, name_list
        )
    }

    /// Build a Python function call query for specific functions
    pub fn python_functions(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return String::new();
        }

        let name_list = func_names.join("|");
        format!(
            r#"
(call
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (string) @func_string))
"#,
            name_list
        )
    }

    /// Build a JavaScript function call query for specific functions
    pub fn javascript_functions(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return String::new();
        }

        let name_list = func_names.join("|");
        format!(
            r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (arguments
    (string) @func_string))
"#,
            name_list
        )
    }

    /// Build a Java function call query for specific methods
    pub fn java_functions(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return String::new();
        }

        let name_list = func_names.join("|");
        format!(
            r#"
(method_invocation
  name: (identifier) @method_name
  (#match? @method_name "^({})$")
  arguments: (argument_list
    (string_literal) @method_string))
"#,
            name_list
        )
    }

    /// Common error function names across languages
    pub fn common_error_functions() -> &'static [&'static str] {
        &[
            "panic", "error", "fatal", "throw", "raise", "assert", "fail",
        ]
    }

    /// Common logging function names across languages
    pub fn common_log_functions() -> &'static [&'static str] {
        &[
            "log",
            "info",
            "warn",
            "warning",
            "debug",
            "trace",
            "print",
            "println",
            "printf",
            "fprintf",
            "console.log",
            "console.error",
            "console.warn",
        ]
    }

    /// Common format function names across languages
    pub fn common_format_functions() -> &'static [&'static str] {
        &[
            "format",
            "sprintf",
            "fmt.Sprintf",
            "printf",
            "println",
            "String.format",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_macros() {
        let query = FunctionQueries::rust_macros(&["panic", "println"]);
        assert!(query.contains("panic"));
        assert!(query.contains("println"));
        assert!(query.contains("macro_invocation"));
    }

    #[test]
    fn test_go_functions() {
        let query = FunctionQueries::go_functions(&["fmt.Printf", "log.Println"]);
        assert!(query.contains("call_expression"));
        assert!(query.contains("fmt.Printf"));
    }

    #[test]
    fn test_python_functions() {
        let query = FunctionQueries::python_functions(&["print", "logging.info"]);
        assert!(query.contains("call"));
        assert!(query.contains("print"));
    }

    #[test]
    fn test_common_functions() {
        assert!(!FunctionQueries::common_error_functions().is_empty());
        assert!(!FunctionQueries::common_log_functions().is_empty());
        assert!(!FunctionQueries::common_format_functions().is_empty());
    }
}
