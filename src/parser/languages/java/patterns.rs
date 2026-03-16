//! Java-specific patterns for method and function classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// Java patterns for method classification
#[derive(Clone)]
pub struct JavaPatterns {
    registry: FunctionPatternRegistry,
}

impl JavaPatterns {
    /// Create a new Java patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default Java patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error/exception methods
        registry.register_functions("java", FunctionCategory::Error, &["throw", "throws"]);

        // Format methods
        registry.register_functions(
            "java",
            FunctionCategory::Format,
            &["format", "printf", "sprintf"],
        );

        // Log methods
        registry.register_functions(
            "java",
            FunctionCategory::Log,
            &["log", "trace", "debug", "info", "warn", "error", "fatal"],
        );

        // Print methods (System.out.println, etc.)
        registry.register_functions(
            "java",
            FunctionCategory::Log,
            &["print", "println", "write"],
        );

        // Debug methods
        registry.register_functions("java", FunctionCategory::Debug, &["toString", "dump"]);
    }

    /// Classify a method by name
    pub fn classify_method(&self, method_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("java", method_name)
    }

    /// Check if method is an error-related method
    pub fn is_error_method(&self, method_name: &str) -> bool {
        self.registry.is_error_function("java", method_name)
    }

    /// Check if method is a format method
    pub fn is_format_method(&self, method_name: &str) -> bool {
        self.registry.is_format_function("java", method_name)
    }

    /// Check if method is a log method
    pub fn is_log_method(&self, method_name: &str) -> bool {
        self.registry.is_log_function("java", method_name)
    }

    /// Check if method is a debug method
    pub fn is_debug_method(&self, method_name: &str) -> bool {
        self.registry.is_debug_function("java", method_name)
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
}
