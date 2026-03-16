//! C-specific patterns for function classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// C patterns for function classification
#[derive(Clone)]
pub struct CPatterns {
    registry: FunctionPatternRegistry,
}

impl CPatterns {
    /// Create a new C patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default C patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error functions
        registry.register_functions(
            "c",
            FunctionCategory::Error,
            &[
                "perror",
                "strerror",
                "assert",
                "assert_fail",
                "__assert_fail",
            ],
        );

        // Format functions
        registry.register_functions(
            "c",
            FunctionCategory::Format,
            &[
                "printf",
                "fprintf",
                "sprintf",
                "snprintf",
                "vprintf",
                "vfprintf",
                "vsprintf",
                "vsnprintf",
                "scanf",
                "fscanf",
                "sscanf",
            ],
        );

        // Log functions
        registry.register_functions(
            "c",
            FunctionCategory::Log,
            &["syslog", "openlog", "closelog"],
        );
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("c", func_name)
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.registry.is_error_function("c", func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.registry.is_format_function("c", func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.registry.is_log_function("c", func_name)
    }
}

impl Default for CPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_patterns() {
        let patterns = CPatterns::new();

        assert!(patterns.is_error_function("perror"));
        assert!(patterns.is_format_function("printf"));
        assert!(patterns.is_format_function("sprintf"));
        assert!(patterns.is_log_function("syslog"));

        assert!(!patterns.is_error_function("printf"));
        assert!(!patterns.is_format_function("perror"));
    }
}
