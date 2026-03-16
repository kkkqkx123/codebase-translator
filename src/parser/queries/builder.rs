//! Query builder for constructing tree-sitter queries

use tree_sitter::Language;

use crate::core::error::{Result, TranslateError};

/// Query pattern types
#[derive(Debug, Clone)]
pub enum QueryPattern {
    /// Line comments
    LineComments,
    /// Block comments
    BlockComments,
    /// All comments
    AllComments,
    /// Doc comments (language-specific)
    DocComments,
    /// String literals
    StringLiterals,
    /// Raw string literals
    RawStringLiterals,
    /// All strings
    AllStrings,
    /// Function calls with specific names
    FunctionCalls(Vec<String>),
    /// Macro invocations
    MacroInvocations,
    /// Custom query pattern
    Custom(String),
}

/// Query builder for constructing tree-sitter queries
pub struct QueryBuilder {
    language: String,
    patterns: Vec<QueryPattern>,
}

impl QueryBuilder {
    /// Create a new query builder for a language
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
            patterns: Vec::new(),
        }
    }

    /// Add line comments pattern
    pub fn with_line_comments(mut self) -> Self {
        self.patterns.push(QueryPattern::LineComments);
        self
    }

    /// Add block comments pattern
    pub fn with_block_comments(mut self) -> Self {
        self.patterns.push(QueryPattern::BlockComments);
        self
    }

    /// Add all comments pattern
    pub fn with_all_comments(mut self) -> Self {
        self.patterns.push(QueryPattern::AllComments);
        self
    }

    /// Add doc comments pattern
    pub fn with_doc_comments(mut self) -> Self {
        self.patterns.push(QueryPattern::DocComments);
        self
    }

    /// Add string literals pattern
    pub fn with_string_literals(mut self) -> Self {
        self.patterns.push(QueryPattern::StringLiterals);
        self
    }

    /// Add raw string literals pattern
    pub fn with_raw_strings(mut self) -> Self {
        self.patterns.push(QueryPattern::RawStringLiterals);
        self
    }

    /// Add all strings pattern
    pub fn with_all_strings(mut self) -> Self {
        self.patterns.push(QueryPattern::AllStrings);
        self
    }

    /// Add function calls pattern
    pub fn with_function_calls(mut self, functions: &[&str]) -> Self {
        self.patterns.push(QueryPattern::FunctionCalls(
            functions.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Add macro invocations pattern
    pub fn with_macros(mut self) -> Self {
        self.patterns.push(QueryPattern::MacroInvocations);
        self
    }

    /// Add custom pattern
    pub fn with_custom(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(QueryPattern::Custom(pattern.into()));
        self
    }

    /// Build the query string
    pub fn build(&self) -> String {
        let mut queries = Vec::new();

        for pattern in &self.patterns {
            match pattern {
                QueryPattern::LineComments => {
                    queries.push(self.line_comment_query());
                }
                QueryPattern::BlockComments => {
                    queries.push(self.block_comment_query());
                }
                QueryPattern::AllComments => {
                    queries.push(self.line_comment_query());
                    queries.push(self.block_comment_query());
                }
                QueryPattern::DocComments => {
                    if let Some(query) = self.doc_comment_query() {
                        queries.push(query);
                    }
                }
                QueryPattern::StringLiterals => {
                    queries.push(self.string_literal_query());
                }
                QueryPattern::RawStringLiterals => {
                    queries.push(self.raw_string_query());
                }
                QueryPattern::AllStrings => {
                    queries.push(self.string_literal_query());
                    queries.push(self.raw_string_query());
                }
                QueryPattern::FunctionCalls(names) => {
                    queries.push(self.function_call_query(names));
                }
                QueryPattern::MacroInvocations => {
                    queries.push(self.macro_invocation_query());
                }
                QueryPattern::Custom(q) => {
                    queries.push(q.clone());
                }
            }
        }

        queries.join("\n")
    }

    /// Build and parse the query
    pub fn build_query(&self, language: &Language) -> Result<tree_sitter::Query> {
        let query_str = self.build();
        tree_sitter::Query::new(language, &query_str)
            .map_err(|e| TranslateError::Parse(format!("Invalid query: {}", e)))
    }

    // Language-specific query generators

    fn line_comment_query(&self) -> String {
        match self.language.as_str() {
            "rust" => "(line_comment) @comment".to_string(),
            "go" => "(comment) @comment".to_string(),
            "python" => "(comment) @comment".to_string(),
            _ => "(comment) @comment".to_string(),
        }
    }

    fn block_comment_query(&self) -> String {
        match self.language.as_str() {
            "rust" => "(block_comment) @comment".to_string(),
            "go" => "(block_comment) @comment".to_string(),
            _ => "(block_comment) @comment".to_string(),
        }
    }

    fn doc_comment_query(&self) -> Option<String> {
        match self.language.as_str() {
            "rust" => Some(
                r#"
((line_comment) @docstring
  (#match? @docstring "^///"))

((line_comment) @docstring
  (#match? @docstring "^//!"))

((block_comment) @docstring
  (#match? @docstring "^/\\*\\*"))
"#
                .to_string(),
            ),
            "go" => Some(
                r#"
((comment) @docstring
  (#match? @docstring "^// "))
"#
                .to_string(),
            ),
            "python" => Some(
                r#"
((expression_statement (string)) @docstring)
"#
                .to_string(),
            ),
            _ => None,
        }
    }

    fn string_literal_query(&self) -> String {
        match self.language.as_str() {
            "rust" => "(string_literal) @string".to_string(),
            "go" => "(interpreted_string_literal) @string".to_string(),
            "python" => "(string) @string".to_string(),
            _ => "(string) @string".to_string(),
        }
    }

    fn raw_string_query(&self) -> String {
        match self.language.as_str() {
            "rust" => "(raw_string_literal) @string".to_string(),
            "go" => "(raw_string_literal) @string".to_string(),
            "python" => "(string) @string".to_string(),
            _ => "(raw_string) @string".to_string(),
        }
    }

    fn function_call_query(&self, names: &[String]) -> String {
        if names.is_empty() {
            return String::new();
        }

        let name_pattern = names.join("|");
        format!(
            r#"
(call_expression
  function: (identifier) @func_name
  (#match? @func_name "^({})$")
  arguments: (argument_list) @args)
"#,
            name_pattern
        )
    }

    fn macro_invocation_query(&self) -> String {
        match self.language.as_str() {
            "rust" => r#"
(macro_invocation
  macro: (identifier) @macro_name
  (token_tree) @macro_body)
"#
            .to_string(),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_rust() {
        let builder = QueryBuilder::new("rust")
            .with_line_comments()
            .with_doc_comments();

        let query = builder.build();
        assert!(query.contains("line_comment"));
        assert!(query.contains("docstring"));
    }

    #[test]
    fn test_query_builder_go() {
        let builder = QueryBuilder::new("go")
            .with_all_comments()
            .with_string_literals();

        let query = builder.build();
        assert!(query.contains("comment"));
        assert!(query.contains("interpreted_string_literal"));
    }

    #[test]
    fn test_query_builder_function_calls() {
        let builder = QueryBuilder::new("rust").with_function_calls(&["println", "format"]);

        let query = builder.build();
        assert!(query.contains("println"));
        assert!(query.contains("format"));
    }
}
