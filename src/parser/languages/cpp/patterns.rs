//! C++-specific patterns for function classification

use crate::parser::function_patterns::{FunctionCategory, LanguageFunctionPatterns};

/// C++ patterns for function classification
#[derive(Clone)]
pub struct CppPatterns {
    patterns: LanguageFunctionPatterns,
}

impl CppPatterns {
    /// Create a new C++ patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create C++ patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Error functions
        patterns.error_functions.extend(vec![
            "perror".to_string(),
            "strerror".to_string(),
            "assert".to_string(),
            "static_assert".to_string(),
            "throw".to_string(),
        ]);

        // Format functions
        patterns.format_functions.extend(vec![
            "printf".to_string(),
            "fprintf".to_string(),
            "sprintf".to_string(),
            "snprintf".to_string(),
            "vprintf".to_string(),
            "vfprintf".to_string(),
            "vsprintf".to_string(),
            "vsnprintf".to_string(),
            "scanf".to_string(),
            "fscanf".to_string(),
            "sscanf".to_string(),
            "std::format".to_string(),
            "std::vformat".to_string(),
            "std::sprintf".to_string(),
            "std::snprintf".to_string(),
        ]);

        // Log functions
        patterns.log_functions.extend(vec![
            "std::cout".to_string(),
            "std::cerr".to_string(),
            "std::clog".to_string(),
            "syslog".to_string(),
        ]);

        // Debug functions
        patterns
            .debug_functions
            .extend(vec!["std::cout".to_string(), "std::cerr".to_string()]);

        patterns
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.patterns.classify(func_name)
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.patterns.is_error_function(func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.patterns.is_format_function(func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.patterns.is_log_function(func_name)
    }

    /// Check if function is a debug function
    pub fn is_debug_function(&self, func_name: &str) -> bool {
        self.patterns.is_debug_function(func_name)
    }

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
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
