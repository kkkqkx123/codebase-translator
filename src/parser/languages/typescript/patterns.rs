//! TypeScript-specific patterns for function and method classification

use crate::parser::function_patterns::{FunctionCategory, LanguageFunctionPatterns};

/// TypeScript patterns for function classification
#[derive(Clone)]
pub struct TypeScriptPatterns {
    patterns: LanguageFunctionPatterns,
}

impl TypeScriptPatterns {
    /// Create a new TypeScript patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create TypeScript patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Console methods
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

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
    }
}

impl Default for TypeScriptPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_function() {
        let patterns = TypeScriptPatterns::new();

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
        let patterns = TypeScriptPatterns::new();

        assert!(patterns.is_log_function("console.log"));
        assert!(patterns.is_log_function("log"));
        assert!(patterns.is_error_function("Error"));
        assert!(patterns.is_error_function("throw"));

        assert!(!patterns.is_log_function("Error"));
        assert!(!patterns.is_error_function("console.log"));
    }

    #[test]
    fn test_function_lists() {
        assert!(TypeScriptPatterns::log_functions().contains(&"console.log"));
        assert!(TypeScriptPatterns::log_functions().contains(&"alert"));
        assert!(TypeScriptPatterns::error_functions().contains(&"Error"));
        assert!(TypeScriptPatterns::error_functions().contains(&"throw"));
    }
}
