//! Tree-sitter queries for TypeScript

/// TypeScript tree-sitter queries
pub struct TypeScriptQueries;

impl TypeScriptQueries {
    /// Comment query (includes both line and block comments)
    pub fn comments() -> &'static str {
        "(comment) @comment"
    }

    /// All comments query (excludes JSDoc)
    pub fn all_comments() -> &'static str {
        r#"
((comment) @comment
  (#not-match? @comment "^/\\*\\*"))
"#
    }

    /// JSDoc comment query (/** ... */)
    pub fn jsdoc_comments() -> &'static str {
        r#"
((comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
    }

    /// String literal query
    pub fn string_literals() -> &'static str {
        "(string) @string"
    }

    /// Template string query
    pub fn template_strings() -> &'static str {
        "(template_string) @template"
    }

    /// All strings query (includes template strings)
    pub fn all_strings() -> &'static str {
        r#"
(string) @string
(template_string) @template
"#
    }

    /// Call expression with string arguments
    pub fn call_strings() -> &'static str {
        r#"
(call_expression
  function: (identifier) @func_name
  arguments: (arguments
    (string) @call_string))

(call_expression
  function: (member_expression
    property: (property_identifier) @method_name)
  arguments: (arguments
    (string) @call_string))
"#
    }

    /// Specific function call query
    pub fn specific_calls(func_names: &[&str]) -> String {
        if func_names.is_empty() {
            return Self::call_strings().to_string();
        }

        let name_pattern = func_names.join("|");
        format!(
            r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (arguments
    (string) @call_string))

(call_expression
  function: (member_expression
    property: (property_identifier) @method_name
    (#match? @method_name "^({})$")
  )
  arguments: (arguments
    (string) @call_string))
"#,
            name_pattern, name_pattern
        )
    }

    /// Console method calls
    pub fn console_calls() -> &'static str {
        r#"
(call_expression
  function: (member_expression
    object: (identifier) @console_obj
    (#eq? @console_obj "console")
    property: (property_identifier) @console_method)
  arguments: (arguments
    . (string) @console_string))
"#
    }

    /// Throw statement query for TypeScript
    /// Matches: throw new Error("message"), throw "message", throw new CustomError("message")
    pub fn throw_statements() -> &'static str {
        r#"
(throw_statement
  (string) @throw_string)

(throw_statement
  (new_expression
    constructor: (identifier) @error_class
    arguments: (arguments
      (string) @throw_string)))

(throw_statement
  (new_expression
    constructor: (member_expression
      property: (property_identifier) @error_class)
    arguments: (arguments
      (string) @throw_string)))
"#
    }

    /// Assertion expression query
    /// Matches: assert(...), expect(...).toThrow(...), console.assert(...)
    pub fn assertion_expressions() -> &'static str {
        r#"
(call_expression
  function: (identifier) @assert_func
  (#match? @assert_func "^(assert|assertStrictEquals|assertEquals|assertThrows|rejects|throws)$")
  arguments: (arguments
    (string) @assert_string))

(call_expression
  function: (member_expression
    object: (identifier) @assert_obj
    (#match? @assert_obj "^(assert|console|expect|chai)$")
    property: (property_identifier) @assert_method
    (#match? @assert_method "^(assert|fail|throws|rejects|toThrow|toThrowError)$"))
  arguments: (arguments
    (string) @assert_string))

(call_expression
  function: (member_expression
    object: (call_expression) @expect_call
    property: (property_identifier) @expect_method
    (#match? @expect_method "^(toThrow|toThrowError|rejects)$"))
  arguments: (arguments
    (string) @assert_string))
"#
    }

    /// Variable assignment string query
    /// Matches: const x = "message", let x = "message", var x = "message"
    pub fn variable_assignments() -> &'static str {
        r#"
(variable_declarator
  name: (identifier) @var_name
  value: (string) @var_string)

(variable_declarator
  name: (identifier) @var_name
  value: (template_string) @var_template)
"#
    }

    /// Object property string query
    /// Matches: { key: "value" }, { "key": "value" }
    pub fn object_properties() -> &'static str {
        r#"
(pair
  key: (property_identifier) @prop_key
  value: (string) @prop_string)

(pair
  key: (string) @prop_key
  value: (string) @prop_string)

(pair
  key: (property_identifier) @prop_key
  value: (template_string) @prop_template)
"#
    }

    /// Export variable assignment query
    /// Matches: export const x = "message"
    pub fn export_variable_assignments() -> &'static str {
        r#"
(export_statement
  (variable_declaration
    (variable_declarator
      name: (identifier) @export_var_name
      value: (string) @export_var_string)))

(export_statement
  (variable_declaration
    (variable_declarator
      name: (identifier) @export_var_name
      value: (template_string) @export_var_template)))
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    fn validate_query_syntax(query_name: &str, query_str: &str) -> Result<(), String> {
        let lang = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        match Query::new(&lang.into(), query_str) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Query '{}' syntax error: {:?}", query_name, e)),
        }
    }

    #[test]
    fn test_comments_query_syntax_valid() {
        let result = validate_query_syntax("comments", TypeScriptQueries::comments());
        assert!(result.is_ok(), "Comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_comments_query_syntax_valid() {
        let result = validate_query_syntax("all_comments", TypeScriptQueries::all_comments());
        assert!(result.is_ok(), "All comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_jsdoc_comments_query_syntax_valid() {
        let result = validate_query_syntax("jsdoc_comments", TypeScriptQueries::jsdoc_comments());
        assert!(result.is_ok(), "JSDoc comments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_string_literals_query_syntax_valid() {
        let result = validate_query_syntax("string_literals", TypeScriptQueries::string_literals());
        assert!(result.is_ok(), "String literals query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_template_strings_query_syntax_valid() {
        let result = validate_query_syntax("template_strings", TypeScriptQueries::template_strings());
        assert!(result.is_ok(), "Template strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_all_strings_query_syntax_valid() {
        let result = validate_query_syntax("all_strings", TypeScriptQueries::all_strings());
        assert!(result.is_ok(), "All strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_call_strings_query_syntax_valid() {
        let result = validate_query_syntax("call_strings", TypeScriptQueries::call_strings());
        assert!(result.is_ok(), "Call strings query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_specific_calls_query_syntax_valid() {
        let specific = TypeScriptQueries::specific_calls(&["log", "error", "warn"]);
        let result = validate_query_syntax("specific_calls", &specific);
        assert!(result.is_ok(), "Specific calls query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_console_calls_query_syntax_valid() {
        let result = validate_query_syntax("console_calls", TypeScriptQueries::console_calls());
        assert!(result.is_ok(), "Console calls query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_throw_statements_query_syntax_valid() {
        let result = validate_query_syntax("throw_statements", TypeScriptQueries::throw_statements());
        assert!(result.is_ok(), "Throw statements query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_assertion_expressions_query_syntax_valid() {
        let result = validate_query_syntax("assertion_expressions", TypeScriptQueries::assertion_expressions());
        assert!(result.is_ok(), "Assertion expressions query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_variable_assignments_query_syntax_valid() {
        let result = validate_query_syntax("variable_assignments", TypeScriptQueries::variable_assignments());
        assert!(result.is_ok(), "Variable assignments query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_object_properties_query_syntax_valid() {
        let result = validate_query_syntax("object_properties", TypeScriptQueries::object_properties());
        assert!(result.is_ok(), "Object properties query syntax validation failed: {:?}", result.err());
    }

    #[test]
    fn test_export_variable_assignments_query_syntax_valid() {
        let result = validate_query_syntax("export_variable_assignments", TypeScriptQueries::export_variable_assignments());
        assert!(result.is_ok(), "Export variable assignments query syntax validation failed: {:?}", result.err());
    }
}
