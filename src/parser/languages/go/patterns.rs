//! Go-specific patterns for function classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// Go patterns for function classification
#[derive(Clone)]
pub struct GoPatterns {
    registry: FunctionPatternRegistry,
}

impl GoPatterns {
    /// Create a new Go patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default Go patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error functions (panic and fatal)
        registry.register_functions(
            "go",
            FunctionCategory::Error,
            &[
                "panic",
                "log.Fatal",
                "log.Fatalf",
                "log.Panic",
                "log.Panicf",
                "log.Panicln",
            ],
        );

        // Format functions (fmt package)
        registry.register_functions(
            "go",
            FunctionCategory::Format,
            &[
                "fmt.Print",
                "fmt.Printf",
                "fmt.Println",
                "fmt.Sprintf",
                "fmt.Fprintf",
                "fmt.Fprint",
                "fmt.Fprintln",
                "fmt.Sprint",
                "fmt.Sprintln",
            ],
        );

        // Log functions (log package)
        registry.register_functions(
            "go",
            FunctionCategory::Log,
            &[
                "log.Print",
                "log.Printf",
                "log.Println",
                "log.Fatal",
                "log.Fatalf",
                "log.Fatalln",
                "log.Panic",
                "log.Panicf",
                "log.Panicln",
            ],
        );

        // Testing functions
        registry.register_functions(
            "go",
            FunctionCategory::Log,
            &[
                "t.Log", "t.Logf", "t.Error", "t.Errorf", "t.Fatal", "t.Fatalf", "b.Log", "b.Logf",
                "b.Error", "b.Errorf",
            ],
        );
    }

    /// Classify a function by name
    pub fn classify_function(&self, func_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("go", func_name)
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.registry.is_error_function("go", func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.registry.is_format_function("go", func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.registry.is_log_function("go", func_name)
    }

    /// Get all error functions
    pub fn error_functions() -> &'static [&'static str] {
        &[
            "panic",
            "log.Fatal",
            "log.Fatalf",
            "log.Panic",
            "log.Panicf",
            "log.Panicln",
        ]
    }

    /// Get all format functions
    pub fn format_functions() -> &'static [&'static str] {
        &[
            "fmt.Print",
            "fmt.Printf",
            "fmt.Println",
            "fmt.Sprintf",
            "fmt.Fprintf",
            "fmt.Fprint",
            "fmt.Fprintln",
            "fmt.Sprint",
            "fmt.Sprintln",
        ]
    }

    /// Get all log functions
    pub fn log_functions() -> &'static [&'static str] {
        &[
            "log.Print",
            "log.Printf",
            "log.Println",
            "log.Fatal",
            "log.Fatalf",
            "log.Fatalln",
            "log.Panic",
            "log.Panicf",
            "log.Panicln",
        ]
    }

    /// Get all testing functions
    pub fn testing_functions() -> &'static [&'static str] {
        &[
            "t.Log", "t.Logf", "t.Error", "t.Errorf", "t.Fatal", "t.Fatalf", "b.Log", "b.Logf",
            "b.Error", "b.Errorf",
        ]
    }
}

impl Default for GoPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_function() {
        let patterns = GoPatterns::new();

        assert_eq!(
            patterns.classify_function("panic"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            patterns.classify_function("fmt.Printf"),
            Some(FunctionCategory::Format)
        );
        assert_eq!(
            patterns.classify_function("log.Println"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(patterns.classify_function("unknown"), None);
    }

    #[test]
    fn test_is_functions() {
        let patterns = GoPatterns::new();

        assert!(patterns.is_error_function("panic"));
        assert!(patterns.is_format_function("fmt.Printf"));
        assert!(patterns.is_log_function("log.Println"));

        assert!(!patterns.is_error_function("fmt.Printf"));
        assert!(!patterns.is_format_function("panic"));
    }

    #[test]
    fn test_function_lists() {
        assert!(GoPatterns::error_functions().contains(&"panic"));
        assert!(GoPatterns::format_functions().contains(&"fmt.Printf"));
        assert!(GoPatterns::log_functions().contains(&"log.Println"));
        assert!(GoPatterns::testing_functions().contains(&"t.Errorf"));
    }
}
