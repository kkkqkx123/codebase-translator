//! Python-specific patterns for function classification

use crate::parser::abstraction::function_patterns::{FunctionCategory, LanguageFunctionPatterns};

/// Python patterns for function classification
#[derive(Clone)]
pub struct PythonPatterns {
    patterns: LanguageFunctionPatterns,
}

impl PythonPatterns {
    /// Create a new Python patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create Python patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Error functions
        patterns.error_functions.extend(vec![
            "raise".to_string(),
            "raise Exception".to_string(),
            "raise ValueError".to_string(),
            "raise TypeError".to_string(),
            "raise RuntimeError".to_string(),
            "raise AssertionError".to_string(),
            "sys.exit".to_string(),
            "exit".to_string(),
            "quit".to_string(),
            "assert".to_string(),
            "assertEqual".to_string(),
            "assertTrue".to_string(),
            "assertFalse".to_string(),
            "assertRaises".to_string(),
        ]);

        // Format functions
        patterns.format_functions.extend(vec![
            "print".to_string(),
            "format".to_string(),
            "str.format".to_string(),
            "f-string".to_string(),
            "logging.Formatter".to_string(),
            "logging.basicConfig".to_string(),
        ]);

        // Log functions
        patterns.log_functions.extend(vec![
            "logging.info".to_string(),
            "logging.debug".to_string(),
            "logging.warning".to_string(),
            "logging.error".to_string(),
            "logging.critical".to_string(),
            "logging.exception".to_string(),
            "logger.info".to_string(),
            "logger.debug".to_string(),
            "logger.warning".to_string(),
            "logger.error".to_string(),
            "logger.critical".to_string(),
            "logger.exception".to_string(),
            "log.info".to_string(),
            "log.debug".to_string(),
            "log.warning".to_string(),
            "log.error".to_string(),
            "log.critical".to_string(),
        ]);

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

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
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

    #[test]
    fn test_patterns() {
        let patterns = PythonPatterns::new();
        let underlying = patterns.patterns();

        assert!(underlying.is_error_function("raise"));
        assert!(underlying.is_format_function("print"));
        assert!(underlying.is_log_function("logging.info"));
    }
}

