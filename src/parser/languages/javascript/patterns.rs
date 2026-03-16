//! JavaScript-specific patterns for function and method classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// JavaScript patterns for function classification
#[derive(Clone)]
pub struct JavaScriptPatterns {
    registry: FunctionPatternRegistry,
}

impl JavaScriptPatterns {
    /// Create a new JavaScript patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::empty();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default JavaScript patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Console methods
        registry.register_functions(
            "javascript",
            FunctionCategory::Log,
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
            ],
        );

        // Error methods
        registry.register_functions(
            "javascript",
            FunctionCategory::Error,
            &[
                "throw",
                "Error",
                "TypeError",
                "ReferenceError",
                "SyntaxError",
                "RangeError",
                "URIError",
                "EvalError",
            ],
        );

        // Alert/confirm/prompt
        registry.register_functions(
            "javascript",
            FunctionCategory::Log,
            &["alert", "confirm", "prompt"],
        );
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("javascript", func_name)
    }

    /// Check if function is a console/log method
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.registry.is_log_function("javascript", func_name)
    }

    /// Check if function is an error-related method
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.registry.is_error_function("javascript", func_name)
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
}
