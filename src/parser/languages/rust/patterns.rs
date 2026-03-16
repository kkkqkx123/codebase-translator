//! Rust-specific patterns for macro and function classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// Rust patterns for macro classification
#[derive(Clone)]
pub struct RustPatterns {
    registry: FunctionPatternRegistry,
}

impl RustPatterns {
    /// Create a new Rust patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default Rust patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error macros
        registry.register_functions(
            "rust",
            FunctionCategory::Error,
            &[
                "panic!",
                "assert!",
                "assert_eq!",
                "assert_ne!",
                "debug_assert!",
                "debug_assert_eq!",
                "debug_assert_ne!",
                "unreachable!",
                "unimplemented!",
                "todo!",
            ],
        );

        // Format macros
        registry.register_functions(
            "rust",
            FunctionCategory::Format,
            &[
                "format!",
                "format_args!",
                "print!",
                "println!",
                "eprint!",
                "eprintln!",
                "write!",
                "writeln!",
            ],
        );

        // Log macros (note: println! is also in format)
        registry.register_functions(
            "rust",
            FunctionCategory::Log,
            &[
                "println!",
                "eprintln!",
                "log!",
                "trace!",
                "debug!",
                "info!",
                "warn!",
                "error!",
            ],
        );

        // Debug macros
        registry.register_functions("rust", FunctionCategory::Debug, &["dbg!"]);
    }

    /// Classify a macro by name
    pub fn classify_macro(&self, macro_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("rust", macro_name)
    }

    /// Check if macro is an error macro
    pub fn is_error_macro(&self, macro_name: &str) -> bool {
        self.registry.is_error_function("rust", macro_name)
    }

    /// Check if macro is a format macro
    pub fn is_format_macro(&self, macro_name: &str) -> bool {
        self.registry.is_format_function("rust", macro_name)
    }

    /// Check if macro is a log macro
    pub fn is_log_macro(&self, macro_name: &str) -> bool {
        self.registry.is_log_function("rust", macro_name)
    }

    /// Check if macro is a debug macro
    pub fn is_debug_macro(&self, macro_name: &str) -> bool {
        self.registry.is_debug_function("rust", macro_name)
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
}
