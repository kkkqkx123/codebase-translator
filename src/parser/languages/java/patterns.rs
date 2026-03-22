//! Java-specific patterns for method and function classification

use crate::parser::core::types::{FunctionCategory, LanguageFunctionPatterns};

/// Java patterns for method classification
#[derive(Clone)]
pub struct JavaPatterns {
    patterns: LanguageFunctionPatterns,
}

impl JavaPatterns {
    /// Create a new Java patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create Java patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Error/exception methods
        patterns
            .error_functions
            .extend(vec!["throw".to_string(), "throws".to_string()]);

        // Format methods
        patterns.format_functions.extend(vec![
            "format".to_string(),
            "printf".to_string(),
            "sprintf".to_string(),
        ]);

        // Log methods
        patterns.log_functions.extend(vec![
            "log".to_string(),
            "trace".to_string(),
            "debug".to_string(),
            "info".to_string(),
            "warn".to_string(),
            "error".to_string(),
            "fatal".to_string(),
            "print".to_string(),
            "println".to_string(),
            "write".to_string(),
        ]);

        // Debug methods
        patterns
            .debug_functions
            .extend(vec!["toString".to_string(), "dump".to_string()]);

        patterns
    }

    /// Classify a method by name
    pub fn classify_method(&self, method_name: &str) -> Option<FunctionCategory> {
        self.patterns.classify(method_name)
    }

    /// Check if method is an error-related method
    pub fn is_error_method(&self, method_name: &str) -> bool {
        self.patterns.is_error_function(method_name)
    }

    /// Check if method is a format method
    pub fn is_format_method(&self, method_name: &str) -> bool {
        self.patterns.is_format_function(method_name)
    }

    /// Check if method is a log method
    pub fn is_log_method(&self, method_name: &str) -> bool {
        self.patterns.is_log_function(method_name)
    }

    /// Check if method is a debug method
    pub fn is_debug_method(&self, method_name: &str) -> bool {
        self.patterns.is_debug_function(method_name)
    }

    /// Get all error methods
    pub fn error_methods() -> &'static [&'static str] {
        &["throw", "throws"]
    }

    /// Get all format methods
    pub fn format_methods() -> &'static [&'static str] {
        &["format", "printf", "sprintf"]
    }

    /// Get all log methods
    pub fn log_methods() -> &'static [&'static str] {
        &[
            "log", "trace", "debug", "info", "warn", "error", "fatal", "print", "println", "write",
        ]
    }

    /// Get all debug methods
    pub fn debug_methods() -> &'static [&'static str] {
        &["toString", "dump"]
    }

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
    }
}

impl Default for JavaPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_method() {
        let patterns = JavaPatterns::new();

        assert_eq!(
            patterns.classify_method("println"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(
            patterns.classify_method("format"),
            Some(FunctionCategory::Format)
        );
        assert_eq!(
            patterns.classify_method("throw"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            patterns.classify_method("toString"),
            Some(FunctionCategory::Debug)
        );
        assert_eq!(patterns.classify_method("unknownMethod"), None);
    }

    #[test]
    fn test_is_functions() {
        let patterns = JavaPatterns::new();

        assert!(patterns.is_error_method("throw"));
        assert!(patterns.is_format_method("format"));
        assert!(patterns.is_log_method("println"));
        assert!(patterns.is_debug_method("toString"));

        assert!(!patterns.is_error_method("format"));
        assert!(!patterns.is_format_method("println"));
    }

    #[test]
    fn test_method_lists() {
        assert!(JavaPatterns::error_methods().contains(&"throw"));
        assert!(JavaPatterns::format_methods().contains(&"format"));
        assert!(JavaPatterns::log_methods().contains(&"println"));
        assert!(JavaPatterns::debug_methods().contains(&"toString"));
    }

    #[test]
    fn test_patterns() {
        let patterns = JavaPatterns::new();
        let underlying = patterns.patterns();

        assert!(underlying.is_error_function("throw"));
        assert!(underlying.is_format_function("format"));
        assert!(underlying.is_log_function("println"));
        assert!(underlying.is_debug_function("toString"));
    }
}
