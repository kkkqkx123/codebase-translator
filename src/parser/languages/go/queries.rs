//! Tree-sitter queries for Go

/// Go tree-sitter queries
pub struct GoQueries;

impl GoQueries {
    /// All comments query (both line and block comments)
    pub fn all_comments() -> &'static str {
        "(comment) @comment"
    }

    /// Doc comment query for Go
    /// In Go, doc comments are regular comments that precede a declaration.
    /// Since tree-sitter query cannot reliably determine if a comment precedes a declaration,
    /// we return an empty query. All comments should be extracted via all_comments().
    pub fn doc_comments() -> &'static str {
        // Go doc comments are syntactically identical to regular comments.
        // They are identified by position (preceding a declaration) and convention.
        // This requires AST-level analysis beyond simple queries.
        ""
    }

    /// String literal query (interpreted string literals)
    pub fn interpreted_string_literals() -> &'static str {
        "(interpreted_string_literal) @string"
    }

    /// Raw string literal query (raw string literals using backticks)
    pub fn raw_string_literals() -> &'static str {
        "(raw_string_literal) @string"
    }

    /// All string literals query
    pub fn all_strings() -> &'static str {
        r#"
(interpreted_string_literal) @string
(raw_string_literal) @string
"#
    }

    /// Function call with string argument query
    pub fn function_call_strings() -> &'static str {
        r#"
(call_expression
  function: (identifier) @func_name
  arguments: (argument_list
    (interpreted_string_literal) @func_string))

(call_expression
  function: (identifier) @func_name
  arguments: (argument_list
    (raw_string_literal) @func_string))

(call_expression
  function: (selector_expression
    operand: (_) @operand
    field: (field_identifier) @func_name)
  arguments: (argument_list
    (interpreted_string_literal) @func_string))

(call_expression
  function: (selector_expression
    operand: (_) @operand
    field: (field_identifier) @func_name)
  arguments: (argument_list
    (raw_string_literal) @func_string))
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

(call_expression
  function: (selector_expression
    field: (field_identifier) @func_name)
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (interpreted_string_literal) @func_string))

(call_expression
  function: (selector_expression
    field: (field_identifier) @func_name)
  (#match? @func_name "^({})$")
  arguments: (argument_list
    (raw_string_literal) @func_string))
"#,
            name_pattern, name_pattern, name_pattern, name_pattern
        )
    }

    /// Error function call query (panic, fatal, etc.)
    pub fn error_functions() -> &'static str {
        r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^(panic|fatal| Fatalf| Panicf)$")
  arguments: (argument_list
    (interpreted_string_literal) @error_string))

(call_expression
  function: (selector_expression
    field: (field_identifier) @func_name)
  (#match? @func_name "^(Panic|Panicf|Fatal|Fatalf)$")
  arguments: (argument_list
    (interpreted_string_literal) @error_string))
"#
    }

    /// Format function call query (fmt.Printf, fmt.Sprintf, etc.)
    pub fn format_functions() -> &'static str {
        r#"
(call_expression
  function: (selector_expression
    operand: (identifier) @pkg
    (#eq? @pkg "fmt")
    field: (field_identifier) @func_name)
  (#match? @func_name "^(Print|Printf|Println|Sprintf|Fprintf|Sprintln|Fprintln)$")
  arguments: (argument_list
    (interpreted_string_literal) @format_string))

(call_expression
  function: (selector_expression
    operand: (identifier) @pkg
    (#eq? @pkg "fmt")
    field: (field_identifier) @func_name)
  (#match? @func_name "^(Print|Printf|Println|Sprintf|Fprintf|Sprintln|Fprintln)$")
  arguments: (argument_list
    (raw_string_literal) @format_string))
"#
    }

    /// Log function call query (log.Printf, log.Fatal, etc.)
    pub fn log_functions() -> &'static str {
        r#"
(call_expression
  function: (selector_expression
    operand: (identifier) @pkg
    (#eq? @pkg "log")
    field: (field_identifier) @func_name)
  (#match? @func_name "^(Print|Printf|Println|Fatal|Fatalf|Panic|Panicf|Panicln)$")
  arguments: (argument_list
    (interpreted_string_literal) @log_string))

(call_expression
  function: (selector_expression
    operand: (identifier) @pkg
    (#eq? @pkg "log")
    field: (field_identifier) @func_name)
  (#match? @func_name "^(Print|Printf|Println|Fatal|Fatalf|Panic|Panicf|Panicln)$")
  arguments: (argument_list
    (raw_string_literal) @log_string))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!GoQueries::all_comments().is_empty());
        // Go doc comments are syntactically identical to regular comments,
        // so doc_comments() returns an empty query. All comments are extracted
        // via all_comments() and treated as regular comments.
        assert!(GoQueries::doc_comments().is_empty());
    }

    #[test]
    fn test_string_queries() {
        assert!(!GoQueries::interpreted_string_literals().is_empty());
        assert!(!GoQueries::raw_string_literals().is_empty());
        assert!(!GoQueries::all_strings().is_empty());
    }

    #[test]
    fn test_function_queries() {
        assert!(!GoQueries::function_call_strings().is_empty());

        let specific = GoQueries::specific_functions(&["fmt.Printf", "log.Println"]);
        assert!(specific.contains("fmt.Printf"));
        assert!(specific.contains("log.Println"));
    }

    #[test]
    fn test_error_function_queries() {
        assert!(!GoQueries::error_functions().is_empty());
    }

    #[test]
    fn test_format_function_queries() {
        assert!(!GoQueries::format_functions().is_empty());
    }

    #[test]
    fn test_log_function_queries() {
        assert!(!GoQueries::log_functions().is_empty());
    }
}
