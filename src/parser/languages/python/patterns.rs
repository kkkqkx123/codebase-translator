//! Python-specific patterns for function classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// Python patterns for function classification
#[derive(Clone)]
pub struct PythonPatterns {
    registry: FunctionPatternRegistry,
}

impl PythonPatterns {
    /// Create a new Python patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default Python patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error functions
        registry.register_functions(
            "python",
            FunctionCategory::Error,
            &[
                "raise",
                "raise Exception",
                "raise ValueError",
                "raise TypeError",
                "raise RuntimeError",
                "raise AssertionError",
                "sys.exit",
                "exit",
                "quit",
            ],
        );

        // Format functions
        registry.register_functions(
            "python",
            FunctionCategory::Format,
            &[
                "print",
                "format",
                "str.format",
                "f-string",
                "logging.Formatter",
                "logging.basicConfig",
            ],
        );

        // Log functions
        registry.register_functions(
            "python",
            FunctionCategory::Log,
            &[
                "logging.info",
                "logging.debug",
                "logging.warning",
                "logging.error",
                "logging.critical",
                "logging.exception",
                "logger.info",
                "logger.debug",
                "logger.warning",
                "logger.error",
                "logger.critical",
                "logger.exception",
                "log.info",
                "log.debug",
                "log.warning",
                "log.error",
                "log.critical",
            ],
        );

        // Exception handling
        registry.register_functions(
            "python",
            FunctionCategory::Error,
            &[
                "assert",
                "assertEqual",
                "assertTrue",
                "assertFalse",
                "assertRaises",
            ],
        );
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("python", func_name)
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.registry.is_error_function("python", func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.registry.is_format_function("python", func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.registry.is_log_function("python", func_name)
    }

    /// Get all error functions
    pub fn error_functions() -> &'static [&'static str] {
        &[
            "raise",
            "raise Exception",
            "raise ValueError",
            "raise TypeError",
            "raise RuntimeError",
            "raise AssertionError",
            "sys.exit",
            "exit",
            "quit",
            "assert",
            "assertEqual",
            "assertTrue",
            "assertFalse",
            "assertRaises",
        ]
    }

    /// Get all format functions
    pub fn format_functions() -> &'static [&'static str] {
        &[
            "print",
            "format",
            "str.format",
            "f-string",
            "logging.Formatter",
            "logging.basicConfig",
        ]
    }

    /// Get all log functions
    pub fn log_functions() -> &'static [&'static str] {
        &[
            "logging.info",
            "logging.debug",
            "logging.warning",
            "logging.error",
            "logging.critical",
            "logging.exception",
            "logger.info",
            "logger.debug",
            "logger.warning",
            "logger.error",
            "logger.critical",
            "logger.exception",
            "log.info",
            "log.debug",
            "log.warning",
            "log.error",
            "log.critical",
        ]
    }
}

impl Default for PythonPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_function() {
        let patterns = PythonPatterns::new();

        assert_eq!(
            patterns.classify_function("raise"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            patterns.classify_function("print"),
            Some(FunctionCategory::Format)
        );
        assert_eq!(
            patterns.classify_function("logging.info"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(patterns.classify_function("unknown"), None);
    }

    #[test]
    fn test_is_functions() {
        let patterns = PythonPatterns::new();

        assert!(patterns.is_error_function("raise"));
        assert!(patterns.is_format_function("print"));
        assert!(patterns.is_log_function("logging.info"));

        assert!(!patterns.is_error_function("print"));
        assert!(!patterns.is_format_function("raise"));
    }

    #[test]
    fn test_function_lists() {
        assert!(PythonPatterns::error_functions().contains(&"raise"));
        assert!(PythonPatterns::format_functions().contains(&"print"));
        assert!(PythonPatterns::log_functions().contains(&"logging.info"));
    }
}
