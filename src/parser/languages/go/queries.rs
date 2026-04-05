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

    /// Panic expression query
    /// Matches: panic("message"), panic(fmt.Sprintf("format", args))
    pub fn panic_expressions() -> &'static str {
        r#"
(call_expression
  function: (identifier) @panic_name
  (#eq? @panic_name "panic")
  arguments: (argument_list
    (interpreted_string_literal) @panic_string))

(call_expression
  function: (identifier) @panic_name
  (#eq? @panic_name "panic")
  arguments: (argument_list
    (raw_string_literal) @panic_string))
"#
    }

    /// Error function call query (panic, fatal, etc.)
    pub fn error_functions() -> &'static str {
        r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^(panic|fatal|Fatalf|Panicf)$")
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
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_go::LANGUAGE;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", GoQueries::all_comments());
        assert!(result.is_ok(), "All comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_doc_comments_query_syntax_valid() {
        let result = validate_query_syntax("doc_comments", GoQueries::doc_comments());
        assert!(result.is_ok(), "Doc comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_interpreted_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("interpreted_string_literals", GoQueries::interpreted_string_literals());
        assert!(result.is_ok(), "Interpreted string literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_raw_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("raw_string_literals", GoQueries::raw_string_literals());
        assert!(result.is_ok(), "Raw string literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", GoQueries::all_strings());
        assert!(result.is_ok(), "All strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_function_call_strings_query_syntax_valid() {
        let result = validate_query_syntax("function_call_strings", GoQueries::function_call_strings());
        assert!(result.is_ok(), "Function call strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_specific_functions_query_syntax_valid() {
        let specific = GoQueries::specific_functions(&["fmt.Printf", "log.Println"]);
        let result = validate_query_syntax("specific_functions", &specific);
        assert!(result.is_ok(), "Specific functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_panic_expressions_query_syntax_valid() {
        let result = validate_query_syntax("panic_expressions", GoQueries::panic_expressions());
        assert!(result.is_ok(), "Panic expressions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_error_functions_query_syntax_valid() {
        let result = validate_query_syntax("error_functions", GoQueries::error_functions());
        assert!(result.is_ok(), "Error functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_format_functions_query_syntax_valid() {
        let result = validate_query_syntax("format_functions", GoQueries::format_functions());
        assert!(result.is_ok(), "Format functions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_log_functions_query_syntax_valid() {
        let result = validate_query_syntax("log_functions", GoQueries::log_functions());
        assert!(result.is_ok(), "Log functions query syntax validation failed: {:?}", result.err());
    }
}
