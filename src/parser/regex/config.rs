//! Regex parser configuration

/// Regex-based parser configuration
#[derive(Debug, Clone)]
pub struct RegexParserConfig {
    /// File extensions this parser supports
    pub extensions: Vec<String>,
    /// Regex pattern for line comments
    pub line_comment_pattern: Option<String>,
    /// Regex pattern for block comments
    pub block_comment_pattern: Option<String>,
    /// Regex pattern for doc comments
    pub doc_comment_pattern: Option<String>,
    /// Regex pattern for string literals
    pub string_pattern: Option<String>,
    /// Minimum content length
    pub min_content_length: usize,
    /// Maximum content length
    pub max_content_length: usize,
    /// Whether to trim content
    pub trim_content: bool,
    /// State machine patterns for complex extraction
    pub state_machine_patterns: Vec<crate::config::project::StateMachinePattern>,
}

impl Default for RegexParserConfig {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            line_comment_pattern: None,
            block_comment_pattern: None,
            doc_comment_pattern: None,
            string_pattern: None,
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
            state_machine_patterns: Vec::new(),
        }
    }
}

impl RegexParserConfig {
    /// Create a generic fallback parser configuration
    pub fn fallback() -> Self {
        Self {
            extensions: vec!["txt".to_string(), "md".to_string(), "markdown".to_string()],
            line_comment_pattern: Some(r"(?m)^\s*(?://|#|--|;)\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"/\*\s*([\s\S]*?)\s*\*/".to_string()),
            doc_comment_pattern: None,
            string_pattern: Some(r#"["']([^"']{3,})["']"#.to_string()),
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
            state_machine_patterns: Vec::new(),
        }
    }

    /// Create a shell script parser configuration
    pub fn shell() -> Self {
        Self {
            extensions: vec![
                "sh".to_string(),
                "bash".to_string(),
                "zsh".to_string(),
                "fish".to_string(),
            ],
            line_comment_pattern: Some(r"(?m)^\s*#\s*(.+)$".to_string()),
            block_comment_pattern: None,
            doc_comment_pattern: None,
            string_pattern: Some(r#"["']([^"']{3,})["']"#.to_string()),
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
            state_machine_patterns: Vec::new(),
        }
    }

    /// Create an HTML parser configuration
    pub fn html() -> Self {
        Self {
            extensions: vec![
                "html".to_string(),
                "htm".to_string(),
                "xml".to_string(),
                "svg".to_string(),
            ],
            line_comment_pattern: None,
            block_comment_pattern: Some(r"<!--\s*([\s\S]*?)\s*-->".to_string()),
            doc_comment_pattern: None,
            string_pattern: None,
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
            state_machine_patterns: Vec::new(),
        }
    }

    /// Create a SQL parser configuration
    pub fn sql() -> Self {
        Self {
            extensions: vec!["sql".to_string(), "mysql".to_string(), "pgsql".to_string()],
            line_comment_pattern: Some(r"(?m)^\s*--\s*(.+)$".to_string()),
            block_comment_pattern: Some(r"/\*\s*([\s\S]*?)\s*\*/".to_string()),
            doc_comment_pattern: None,
            string_pattern: Some(r#"'([^']{3,})'"#.to_string()),
            min_content_length: 2,
            max_content_length: 10000,
            trim_content: true,
            state_machine_patterns: Vec::new(),
        }
    }

    /// Add a state machine pattern
    pub fn with_state_machine_pattern(
        mut self,
        pattern: crate::config::project::StateMachinePattern,
    ) -> Self {
        self.state_machine_patterns.push(pattern);
        self
    }
}
