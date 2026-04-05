//! JavaScript-specific patterns for function and method classification

use crate::parser::core::types::{FunctionCategory, LanguageFunctionPatterns};

/// JavaScript patterns for function classification
#[derive(Clone)]
pub struct JavaScriptPatterns {
    patterns: LanguageFunctionPatterns,
}

impl JavaScriptPatterns {
    /// Create a new JavaScript patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create JavaScript patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Console methods (log category)
        patterns.log_functions.extend(vec![
            "console.log".to_string(),
            "console.error".to_string(),
            "console.warn".to_string(),
            "console.info".to_string(),
            "console.debug".to_string(),
            "console.trace".to_string(),
            "log".to_string(),
            "error".to_string(),
            "warn".to_string(),
            "info".to_string(),
            "debug".to_string(),
            "trace".to_string(),
            "alert".to_string(),
            "confirm".to_string(),
            "prompt".to_string(),
        ]);

        // Error methods
        patterns.error_functions.extend(vec![
            "throw".to_string(),
            "Error".to_string(),
            "TypeError".to_string(),
            "ReferenceError".to_string(),
            "SyntaxError".to_string(),
            "RangeError".to_string(),
            "URIError".to_string(),
            "EvalError".to_string(),
        ]);

        // Test functions (Jest, Mocha, Jasmine, Vitest, etc.)
        patterns.test_functions.extend(vec![
            "it".to_string(),
            "describe".to_string(),
            "test".to_string(),
            "specify".to_string(),
            "xdescribe".to_string(),
            "xit".to_string(),
            "xspecify".to_string(),
            "fdescribe".to_string(),
            "fit".to_string(),
            "fspecify".to_string(),
            "beforeEach".to_string(),
            "afterEach".to_string(),
            "beforeAll".to_string(),
            "afterAll".to_string(),
            "suite".to_string(),
            "context".to_string(),
        ]);

        patterns
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.patterns.classify(func_name)
    }

    /// Check if function is a console/log method
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.patterns.is_log_function(func_name)
    }

    /// Check if function is an error-related method
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.patterns.is_error_function(func_name)
    }

    /// Check if function is a test function
    pub fn is_test_function(&self, func_name: &str) -> bool {
        self.patterns.is_test_function(func_name)
    }

    /// Get all console/log methods
    pub fn log_functions() -> &'static [&'static str] {
        &[
            "console.log",
            "console.error",
            "console.warn",
            "console.info",
            "console.debug",
            "console.trace",
            "log",
            "error",
            "warn",
            "info",
            "debug",
            "trace",
            "alert",
            "confirm",
            "prompt",
        ]
    }

    /// Get all error functions
    pub fn error_functions() -> &'static [&'static str] {
        &[
            "throw",
            "Error",
            "TypeError",
            "ReferenceError",
            "SyntaxError",
            "RangeError",
            "URIError",
            "EvalError",
        ]
    }

    /// Get all test functions
    pub fn test_functions() -> &'static [&'static str] {
        &[
            "it",
            "describe",
            "test",
            "specify",
            "xdescribe",
            "xit",
            "xspecify",
            "fdescribe",
            "fit",
            "fspecify",
            "beforeEach",
            "afterEach",
            "beforeAll",
            "afterAll",
            "suite",
            "context",
        ]
    }

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
    }
}

impl Default for JavaScriptPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_function() {
        let patterns = JavaScriptPatterns::new();

        assert_eq!(
            patterns.classify_function("console.log"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(
            patterns.classify_function("log"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(
            patterns.classify_function("Error"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            patterns.classify_function("alert"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(patterns.classify_function("unknownFunc"), None);
    }

    #[test]
    fn test_is_functions() {
        let patterns = JavaScriptPatterns::new();

        assert!(patterns.is_log_function("console.log"));
        assert!(patterns.is_log_function("log"));
        assert!(patterns.is_error_function("Error"));
        assert!(patterns.is_error_function("throw"));

        assert!(!patterns.is_log_function("Error"));
        assert!(!patterns.is_error_function("console.log"));
    }

    #[test]
    fn test_function_lists() {
        assert!(JavaScriptPatterns::log_functions().contains(&"console.log"));
        assert!(JavaScriptPatterns::log_functions().contains(&"alert"));
        assert!(JavaScriptPatterns::error_functions().contains(&"Error"));
        assert!(JavaScriptPatterns::error_functions().contains(&"throw"));
    }

    #[test]
    fn test_patterns() {
        let patterns = JavaScriptPatterns::new();
        let underlying = patterns.patterns();

        assert!(underlying.is_log_function("console.log"));
        assert!(underlying.is_error_function("Error"));
    }
}
