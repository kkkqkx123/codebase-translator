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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_queries() {
        assert!(!TypeScriptQueries::comments().is_empty());
        assert!(!TypeScriptQueries::all_comments().is_empty());
    }

    #[test]
    fn test_jsdoc_queries() {
        let jsdoc_query = TypeScriptQueries::jsdoc_comments();
        assert!(jsdoc_query.contains("#match?"));
        assert!(jsdoc_query.contains("comment"));
        assert!(jsdoc_query.contains("docstring"));
    }

    #[test]
    fn test_string_queries() {
        assert!(!TypeScriptQueries::string_literals().is_empty());
        assert!(!TypeScriptQueries::template_strings().is_empty());
        assert!(!TypeScriptQueries::all_strings().is_empty());
    }

    #[test]
    fn test_call_queries() {
        assert!(!TypeScriptQueries::call_strings().is_empty());
        assert!(!TypeScriptQueries::console_calls().is_empty());

        let specific = TypeScriptQueries::specific_calls(&["log", "error", "warn"]);
        assert!(specific.contains("log"));
        assert!(specific.contains("error"));
        assert!(specific.contains("warn"));
    }
}

