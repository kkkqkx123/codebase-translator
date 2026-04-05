//! Function patterns module
//!
//! This module provides common types for function and macro classification
//! across different programming languages. It categorizes functions into
//! error, format, log, and debug categories to help determine how to handle
//! string arguments.
//!
//! Language-specific patterns should be defined in the corresponding
//! `src/parser/languages/*/patterns.rs` files.

/// Function category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    /// Error-related functions/macros
    Error,
    /// Formatting functions/macros
    Format,
    /// Logging functions/macros
    Log,
    /// Debug functions/macros
    Debug,
    /// Test functions (it, describe, test, etc.)
    Test,
}

impl FunctionCategory {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Format => "format",
            Self::Log => "log",
            Self::Debug => "debug",
            Self::Test => "test",
        }
    }
}

impl std::fmt::Display for FunctionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Function patterns for a specific language
#[derive(Debug, Clone)]
pub struct LanguageFunctionPatterns {
    /// Error functions
    pub error_functions: Vec<String>,
    /// Format functions
    pub format_functions: Vec<String>,
    /// Log functions
    pub log_functions: Vec<String>,
    /// Debug functions
    pub debug_functions: Vec<String>,
    /// Test functions (it, describe, test, etc.)
    pub test_functions: Vec<String>,
}

impl LanguageFunctionPatterns {
    /// Create new language function patterns
    pub fn new(
        error: Vec<String>,
        format: Vec<String>,
        log: Vec<String>,
        debug: Vec<String>,
    ) -> Self {
        Self {
            error_functions: error,
            format_functions: format,
            log_functions: log,
            debug_functions: debug,
            test_functions: Vec::new(),
        }
    }

    /// Create an empty pattern set
    pub fn empty() -> Self {
        Self {
            error_functions: Vec::new(),
            format_functions: Vec::new(),
            log_functions: Vec::new(),
            debug_functions: Vec::new(),
            test_functions: Vec::new(),
        }
    }

    /// Classify a function by name
    pub fn classify(&self, func_name: &str) -> Option<FunctionCategory> {
        if self.error_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Error)
        } else if self.format_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Format)
        } else if self.log_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Log)
        } else if self.debug_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Debug)
        } else if self.test_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Test)
        } else {
            None
        }
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.error_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.format_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.log_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a debug function
    pub fn is_debug_function(&self, func_name: &str) -> bool {
        self.debug_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a test function
    pub fn is_test_function(&self, func_name: &str) -> bool {
        self.test_functions.iter().any(|f| f == func_name)
    }

    /// Add a function to a category
    pub fn add_function(&mut self, category: FunctionCategory, func_name: String) {
        match category {
            FunctionCategory::Error => self.error_functions.push(func_name),
            FunctionCategory::Format => self.format_functions.push(func_name),
            FunctionCategory::Log => self.log_functions.push(func_name),
            FunctionCategory::Debug => self.debug_functions.push(func_name),
            FunctionCategory::Test => self.test_functions.push(func_name),
        }
    }
}

impl Default for LanguageFunctionPatterns {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_category_display() {
        assert_eq!(FunctionCategory::Error.to_string(), "error");
        assert_eq!(FunctionCategory::Format.to_string(), "format");
        assert_eq!(FunctionCategory::Log.to_string(), "log");
        assert_eq!(FunctionCategory::Debug.to_string(), "debug");
        assert_eq!(FunctionCategory::Test.to_string(), "test");
    }

    #[test]
    fn test_language_function_patterns() {
        let patterns = LanguageFunctionPatterns::new(
            vec!["error1".to_string()],
            vec!["format1".to_string()],
            vec!["log1".to_string()],
            vec!["debug1".to_string()],
        );

        assert_eq!(patterns.classify("error1"), Some(FunctionCategory::Error));
        assert_eq!(patterns.classify("format1"), Some(FunctionCategory::Format));
        assert_eq!(patterns.classify("log1"), Some(FunctionCategory::Log));
        assert_eq!(patterns.classify("debug1"), Some(FunctionCategory::Debug));
        assert_eq!(patterns.classify("unknown"), None);

        assert!(patterns.is_error_function("error1"));
        assert!(!patterns.is_error_function("format1"));
    }

    #[test]
    fn test_test_functions() {
        let mut patterns = LanguageFunctionPatterns::empty();
        patterns.test_functions =
            vec!["it".to_string(), "describe".to_string(), "test".to_string()];

        assert_eq!(patterns.classify("it"), Some(FunctionCategory::Test));
        assert_eq!(patterns.classify("describe"), Some(FunctionCategory::Test));
        assert_eq!(patterns.classify("test"), Some(FunctionCategory::Test));
        assert!(patterns.is_test_function("it"));
        assert!(!patterns.is_test_function("unknown"));
    }

    #[test]
    fn test_add_function() {
        let mut patterns = LanguageFunctionPatterns::empty();
        patterns.add_function(FunctionCategory::Error, "panic!".to_string());

        assert!(patterns.is_error_function("panic!"));
        assert_eq!(patterns.classify("panic!"), Some(FunctionCategory::Error));
    }

    #[test]
    fn test_default() {
        let patterns = LanguageFunctionPatterns::default();

        assert!(!patterns.is_error_function("any"));
        assert!(!patterns.is_format_function("any"));
        assert!(!patterns.is_log_function("any"));
        assert!(!patterns.is_debug_function("any"));
        assert!(!patterns.is_test_function("any"));
    }
}
