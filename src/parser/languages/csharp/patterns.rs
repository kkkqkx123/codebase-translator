//! C#-specific patterns for method classification

use crate::parser::function_patterns::{FunctionCategory, FunctionPatternRegistry};

/// C# patterns for method classification
#[derive(Clone)]
pub struct CSharpPatterns {
    registry: FunctionPatternRegistry,
}

impl CSharpPatterns {
    /// Create a new C# patterns instance
    pub fn new() -> Self {
        let mut registry = FunctionPatternRegistry::new();
        Self::register_patterns(&mut registry);
        Self { registry }
    }

    /// Register default C# patterns
    fn register_patterns(registry: &mut FunctionPatternRegistry) {
        // Error methods
        registry.register_functions(
            "csharp",
            FunctionCategory::Error,
            &[
                "throw",
                "Exception",
                "ArgumentException",
                "InvalidOperationException",
                "NotImplementedException",
                "NotSupportedException",
                "NullReferenceException",
                "ArgumentNullException",
                "ArgumentOutOfRangeException",
                "InvalidCastException",
                "InvalidDataException",
                "IOException",
                "FileNotFoundException",
                "DirectoryNotFoundException",
                "TimeoutException",
            ],
        );

        // Format methods
        registry.register_functions(
            "csharp",
            FunctionCategory::Format,
            &[
                "string.Format",
                "Format",
                "string.Join",
                "Join",
                "string.Concat",
                "Concat",
                "StringBuilder.AppendFormat",
                "AppendFormat",
            ],
        );

        // Log methods
        registry.register_functions(
            "csharp",
            FunctionCategory::Log,
            &[
                "Console.WriteLine",
                "Console.Write",
                "Console.Error.WriteLine",
                "Console.Error.Write",
                "Debug.WriteLine",
                "Debug.Write",
                "Debug.Log",
                "Debug.LogError",
                "Debug.LogWarning",
                "Trace.WriteLine",
                "Trace.Write",
                "Trace.TraceError",
                "Trace.TraceWarning",
                "Trace.TraceInformation",
                "ILogger.Log",
                // Simple method names (for member access expressions)
                "WriteLine",
                "Write",
                "Log",
                "LogError",
                "LogWarning",
                "LogInformation",
                "LogDebug",
                "LogTrace",
                "ILogger.LogInformation",
                "ILogger.LogWarning",
                "ILogger.LogError",
                "ILogger.LogDebug",
                "ILogger.LogTrace",
                "Logger.LogInformation",
                "Logger.LogWarning",
                "Logger.LogError",
                "Logger.LogDebug",
            ],
        );

        // Debug methods
        registry.register_functions(
            "csharp",
            FunctionCategory::Debug,
            &[
                "Debug.WriteLine",
                "Debug.Write",
                "Debug.Log",
                "Debug.Assert",
                "Debugger.Log",
            ],
        );
    }

    /// Classify a method by name
    pub fn classify_method(&self, method_name: &str) -> Option<FunctionCategory> {
        self.registry.classify("csharp", method_name)
    }

    /// Check if method is an error method
    pub fn is_error_method(&self, method_name: &str) -> bool {
        self.registry.is_error_function("csharp", method_name)
    }

    /// Check if method is a format method
    pub fn is_format_method(&self, method_name: &str) -> bool {
        self.registry.is_format_function("csharp", method_name)
    }

    /// Check if method is a log method
    pub fn is_log_method(&self, method_name: &str) -> bool {
        self.registry.is_log_function("csharp", method_name)
    }

    /// Check if method is a debug method
    pub fn is_debug_method(&self, method_name: &str) -> bool {
        self.registry.is_debug_function("csharp", method_name)
    }
}

impl Default for CSharpPatterns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csharp_patterns() {
        let patterns = CSharpPatterns::new();

        assert!(patterns.is_error_method("throw"));
        assert!(patterns.is_error_method("ArgumentException"));
        assert!(patterns.is_format_method("string.Format"));
        assert!(patterns.is_format_method("Format"));
        assert!(patterns.is_log_method("Console.WriteLine"));
        assert!(patterns.is_log_method("Debug.Log"));
        assert!(patterns.is_debug_method("Debug.Assert"));

        assert!(!patterns.is_error_method("Console.WriteLine"));
        assert!(!patterns.is_format_method("Console.WriteLine"));
    }
}
