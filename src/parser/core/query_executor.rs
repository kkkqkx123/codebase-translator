//! Tree-sitter query execution utilities

use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor};

use crate::core::error::{Result, TranslateError};
use crate::core::models::Position;
use tracing::{debug, error};

/// Query match result
#[derive(Debug, Clone)]
pub struct QueryMatch<'a> {
    /// Capture name
    pub capture_name: String,
    /// Matched text
    pub text: &'a str,
    /// Start position
    pub start_pos: Position,
    /// End position
    pub end_pos: Position,
    /// Tree-sitter node (for advanced use)
    pub node: Node<'a>,
}

/// Filter for query captures
#[derive(Debug, Clone, Default)]
pub struct CaptureFilter {
    /// Include only these capture names (empty = include all)
    include_names: Vec<String>,
    /// Exclude these capture names
    exclude_names: Vec<String>,
}

impl CaptureFilter {
    /// Create a new capture filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Include only specific capture names
    pub fn include_only(mut self, names: &[&str]) -> Self {
        self.include_names = names.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Exclude specific capture names
    pub fn exclude(mut self, names: &[&str]) -> Self {
        self.exclude_names = names.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Check if a capture name should be included
    pub fn should_include(&self, name: &str) -> bool {
        // If include list is not empty, name must be in it
        if !self.include_names.is_empty() && !self.include_names.contains(&name.to_string()) {
            return false;
        }

        // Name must not be in exclude list
        !self.exclude_names.contains(&name.to_string())
    }
}

/// Tree-sitter query executor
pub struct QueryExecutor {
    query: Query,
    filter: CaptureFilter,
}

impl QueryExecutor {
    /// Create a new query executor
    pub fn new(query: Query) -> Self {
        Self {
            query,
            filter: CaptureFilter::default(),
        }
    }

    /// Create a new query executor from query string
    pub fn from_string(language: &tree_sitter::Language, query_str: &str) -> Result<Self> {
        let query = Query::new(language, query_str)
            .map_err(|e| TranslateError::Parse(format!("Invalid query: {}", e)))?;
        Ok(Self::new(query))
    }

    /// Set capture filter
    pub fn with_filter(mut self, filter: CaptureFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Execute query and return matches
    pub fn execute<'a>(
        &'a self,
        root_node: &'a Node,
        content: &'a str,
    ) -> Result<Vec<QueryMatch<'a>>> {
        let mut cursor = QueryCursor::new();
        let capture_names = self.query.capture_names();
        let text_provider: &[u8] = content.as_bytes();

        debug!(
            query_pattern_count = self.query.pattern_count(),
            "Executing query"
        );

        let mut matches = Vec::new();
        let mut query_matches = cursor.matches(&self.query, *root_node, text_provider);

        while let Some(m) = query_matches.next() {
            for capture in m.captures {
                let capture_name = &capture_names[capture.index as usize];

                // Apply filter
                if !self.filter.should_include(capture_name) {
                    debug!(
                        capture_name = %capture_name,
                        "Capture filtered"
                    );
                    continue;
                }

                let node = capture.node;
                let text = node.utf8_text(content.as_bytes()).map_err(|e| {
                    error!(error = %e, "Failed to get node text");
                    TranslateError::Parse(format!("Failed to get node text: {}", e))
                })?;

                let start_pos = Position::new(
                    node.start_position().row + 1,
                    node.start_position().column + 1,
                    node.start_byte(),
                );

                // Fix: Adjust end_pos to not include trailing newlines
                // If the node text ends with a newline, the end_position would point to the next line
                // We need to keep end_pos on the same line as the actual content
                let end_row = node.end_position().row;
                let end_col = node.end_position().column;
                let end_byte = node.end_byte();

                // Check if the text ends with newline and adjust accordingly
                // When end_col == 0, it means the end_position points to the start of the next line
                // (after the newline character), so we need to adjust it to the previous line
                let (adjusted_end_row, adjusted_end_col, adjusted_end_byte) =
                    if end_col == 0 && end_row > node.start_position().row {
                        // end_position points to the start of next line (after newline)
                        // Adjust to the end of the current line
                        let prev_line_end_byte = end_byte.saturating_sub(1);
                        (end_row, end_col, prev_line_end_byte)
                    } else {
                        (end_row, end_col, end_byte)
                    };

                let end_pos = Position::new(
                    adjusted_end_row + 1,
                    adjusted_end_col + 1,
                    adjusted_end_byte,
                );

                debug!(
                    capture_name = %capture_name,
                    node_kind = node.kind(),
                    text_len = text.len(),
                    "Found capture"
                );

                matches.push(QueryMatch {
                    capture_name: capture_name.to_string(),
                    text,
                    start_pos,
                    end_pos,
                    node,
                });
            }
        }

        debug!(total_matches = matches.len(), "Query execution completed");

        Ok(matches)
    }

    /// Execute query with a custom processor for each match
    pub fn execute_with_processor<'a, F, T>(
        &'a self,
        root_node: &'a Node,
        content: &'a str,
        processor: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&QueryMatch<'a>) -> Option<T>,
    {
        let matches = self.execute(root_node, content)?;
        Ok(matches.iter().filter_map(processor).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn create_test_tree(content: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        parser.parse(content, None).unwrap()
    }

    #[test]
    fn test_capture_filter() {
        let filter = CaptureFilter::new().include_only(&["comment", "docstring"]);

        assert!(filter.should_include("comment"));
        assert!(filter.should_include("docstring"));
        assert!(!filter.should_include("string"));
        assert!(!filter.should_include("other"));
    }

    #[test]
    fn test_capture_filter_exclude() {
        let filter = CaptureFilter::new().exclude(&["string", "number"]);

        assert!(filter.should_include("comment"));
        assert!(!filter.should_include("string"));
        assert!(!filter.should_include("number"));
    }

    #[test]
    fn test_query_executor() {
        let content = r#"
/// Doc comment
fn test() {
    // Regular comment
}
"#;
        let tree = create_test_tree(content);
        let query_str = r#"
        (line_comment) @comment
        "#;

        let executor =
            QueryExecutor::from_string(&tree_sitter_rust::LANGUAGE.into(), query_str).unwrap();

        let root_node = tree.root_node();
        let matches = executor.execute(&root_node, content).unwrap();
        assert!(!matches.is_empty());
    }
}
