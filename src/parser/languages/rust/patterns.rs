//! Rust-specific patterns for macro and function classification

use crate::parser::core::types::{FunctionCategory, LanguageFunctionPatterns};

/// Rust patterns for macro classification
#[derive(Clone)]
pub struct RustPatterns {
    patterns: LanguageFunctionPatterns,
}

impl RustPatterns {
    /// Create a new Rust patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create Rust patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Error macros
        patterns.error_functions.extend(vec![
            "panic!".to_string(),
            "assert!".to_string(),
            "assert_eq!".to_string(),
            "assert_ne!".to_string(),
            "debug_assert!".to_string(),
            "debug_assert_eq!".to_string(),
            "debug_assert_ne!".to_string(),
            "unreachable!".to_string(),
            "unimplemented!".to_string(),
            "todo!".to_string(),
        ]);

        // Format macros
        patterns.format_functions.extend(vec![
            "format!".to_string(),
            "format_args!".to_string(),
            "print!".to_string(),
            "println!".to_string(),
            "eprint!".to_string(),
            "eprintln!".to_string(),
            "write!".to_string(),
            "writeln!".to_string(),
        ]);

        // Log macros (note: println! is also in format)
        patterns.log_functions.extend(vec![
            "println!".to_string(),
            "eprintln!".to_string(),
            "log!".to_string(),
            "trace!".to_string(),
            "debug!".to_string(),
            "info!".to_string(),
            "warn!".to_string(),
            "error!".to_string(),
        ]);

        // Debug macros
        patterns.debug_functions.push("dbg!".to_string());

        // Test macros
        patterns.test_functions.extend(vec![
            "assert!".to_string(),
            "assert_eq!".to_string(),
            "assert_ne!".to_string(),
            "debug_assert!".to_string(),
            "debug_assert_eq!".to_string(),
            "debug_assert_ne!".to_string(),
        ]);

        patterns
    }

    /// Classify a macro by name
    pub fn classify_macro(&self, macro_name: &str) -> Option<FunctionCategory> {
        self.patterns.classify(macro_name)
    }

    /// Check if macro is an error macro
    pub fn is_error_macro(&self, macro_name: &str) -> bool {
        self.patterns.is_error_function(macro_name)
    }

    /// Check if macro is a format macro
    pub fn is_format_macro(&self, macro_name: &str) -> bool {
        self.patterns.is_format_function(macro_name)
    }

    /// Check if macro is a log macro
    pub fn is_log_macro(&self, macro_name: &str) -> bool {
        self.patterns.is_log_function(macro_name)
    }

    /// Check if macro is a debug macro
    pub fn is_debug_macro(&self, macro_name: &str) -> bool {
        self.patterns.is_debug_function(macro_name)
    }

    /// Check if macro is a test macro
    pub fn is_test_macro(&self, macro_name: &str) -> bool {
        self.patterns.is_test_function(macro_name)
    }

    /// Get all error macros
    pub fn error_macros() -> &'static [&'static str] {
        &[
            "panic!",
            "assert!",
            "assert_eq!",
            "assert_ne!",
            "unreachable!",
            "unimplemented!",
            "todo!",
        ]
    }

    /// Get all format macros
    pub fn format_macros() -> &'static [&'static str] {
        &[
            "format!",
            "print!",
            "println!",
            "eprint!",
            "eprintln!",
            "write!",
            "writeln!",
        ]
    }

    /// Get all log macros
    pub fn log_macros() -> &'static [&'static str] {
        &["println!", "eprintln!"]
    }

    /// Get all debug macros
    pub fn debug_macros() -> &'static [&'static str] {
        &["dbg!"]
    }

    /// Get all test macros
    pub fn test_macros() -> &'static [&'static str] {
        &[
            "assert!",
            "assert_eq!",
            "assert_ne!",
            "debug_assert!",
            "debug_assert_eq!",
            "debug_assert_ne!",
        ]
    }

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
    }
}

impl Default for RustPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_macro() {
        let patterns = RustPatterns::new();

        assert_eq!(
            patterns.classify_macro("panic!"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            patterns.classify_macro("format!"),
            Some(FunctionCategory::Format)
        );
        assert_eq!(
            patterns.classify_macro("dbg!"),
            Some(FunctionCategory::Debug)
        );
        assert_eq!(patterns.classify_macro("unknown!"), None);
    }

    #[test]
    fn test_is_functions() {
        let patterns = RustPatterns::new();

        assert!(patterns.is_error_macro("panic!"));
        assert!(patterns.is_format_macro("println!"));
        assert!(patterns.is_log_macro("println!"));
        assert!(patterns.is_debug_macro("dbg!"));

        assert!(!patterns.is_error_macro("format!"));
        assert!(!patterns.is_format_macro("panic!"));
    }

    #[test]
    fn test_macro_lists() {
        assert!(RustPatterns::error_macros().contains(&"panic!"));
        assert!(RustPatterns::format_macros().contains(&"format!"));
        assert!(RustPatterns::log_macros().contains(&"println!"));
        assert!(RustPatterns::debug_macros().contains(&"dbg!"));
    }

    #[test]
    fn test_patterns() {
        let patterns = RustPatterns::new();
        let underlying = patterns.patterns();

        assert!(underlying.is_error_function("panic!"));
        assert!(underlying.is_format_function("format!"));
        assert!(underlying.is_log_function("println!"));
        assert!(underlying.is_debug_function("dbg!"));
    }
}
