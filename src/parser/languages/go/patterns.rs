//! Go-specific patterns for function classification

use crate::parser::function_patterns::{FunctionCategory, LanguageFunctionPatterns};

/// Go patterns for function classification
#[derive(Clone)]
pub struct GoPatterns {
    patterns: LanguageFunctionPatterns,
}

impl GoPatterns {
    /// Create a new Go patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create Go patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Error functions (panic and fatal)
        patterns.error_functions.extend(vec![
            "panic".to_string(),
            "log.Fatal".to_string(),
            "log.Fatalf".to_string(),
            "log.Panic".to_string(),
            "log.Panicf".to_string(),
            "log.Panicln".to_string(),
        ]);

        // Format functions (fmt package)
        patterns.format_functions.extend(vec![
            "fmt.Print".to_string(),
            "fmt.Printf".to_string(),
            "fmt.Println".to_string(),
            "fmt.Sprintf".to_string(),
            "fmt.Fprintf".to_string(),
            "fmt.Fprint".to_string(),
            "fmt.Fprintln".to_string(),
            "fmt.Sprint".to_string(),
            "fmt.Sprintln".to_string(),
        ]);

        // Log functions (log package)
        patterns.log_functions.extend(vec![
            "log.Print".to_string(),
            "log.Printf".to_string(),
            "log.Println".to_string(),
            "log.Fatal".to_string(),
            "log.Fatalf".to_string(),
            "log.Fatalln".to_string(),
            "log.Panic".to_string(),
            "log.Panicf".to_string(),
            "log.Panicln".to_string(),
            "t.Log".to_string(),
            "t.Logf".to_string(),
            "t.Error".to_string(),
            "t.Errorf".to_string(),
            "t.Fatal".to_string(),
            "t.Fatalf".to_string(),
            "b.Log".to_string(),
            "b.Logf".to_string(),
            "b.Error".to_string(),
            "b.Errorf".to_string(),
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

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
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

    #[test]
    fn test_patterns() {
        let patterns = GoPatterns::new();
        let underlying = patterns.patterns();

        assert!(underlying.is_error_function("panic"));
        assert!(underlying.is_format_function("fmt.Printf"));
        assert!(underlying.is_log_function("log.Println"));
    }
}
