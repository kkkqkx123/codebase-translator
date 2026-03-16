//! C++-specific patterns for function classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// C++ patterns for function classification
#[derive(Clone)]
pub struct CppPatterns {
    registry: FunctionPatternRegistry,
}

impl CppPatterns {
    /// Create a new C++ patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default C++ patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error functions
        registry.register_functions(
            "cpp",
            FunctionCategory::Error,
            &["perror", "strerror", "assert", "static_assert", "throw"],
        );

        // Format functions
        registry.register_functions(
            "cpp",
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
                "std::format",
                "std::vformat",
                "std::sprintf",
                "std::snprintf",
            ],
        );

        // Log functions
        registry.register_functions(
            "cpp",
            FunctionCategory::Log,
            &["std::cout", "std::cerr", "std::clog", "syslog"],
        );

        // Debug functions
        registry.register_functions("cpp", FunctionCategory::Debug, &["std::cout", "std::cerr"]);
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("cpp", func_name)
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.registry.is_error_function("cpp", func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.registry.is_format_function("cpp", func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.registry.is_log_function("cpp", func_name)
    }

    /// Check if function is a debug function
    pub fn is_debug_function(&self, func_name: &str) -> bool {
        self.registry.is_debug_function("cpp", func_name)
    }
}

impl Default for CppPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_patterns() {
        let patterns = CppPatterns::new();

        assert!(patterns.is_error_function("perror"));
        assert!(patterns.is_error_function("throw"));
        assert!(patterns.is_format_function("printf"));
        assert!(patterns.is_format_function("std::format"));
        assert!(patterns.is_log_function("std::cout"));

        assert!(!patterns.is_error_function("printf"));
        assert!(!patterns.is_format_function("perror"));
    }
}
