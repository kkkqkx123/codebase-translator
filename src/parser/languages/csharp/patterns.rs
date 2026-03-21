//! C#-specific patterns for method classification

use crate::parser::patterns::{FunctionCategory, LanguageFunctionPatterns};

/// C# patterns for method classification
#[derive(Clone)]
pub struct CSharpPatterns {
    patterns: LanguageFunctionPatterns,
}

impl CSharpPatterns {
    /// Create a new C# patterns instance
    pub fn new() -> Self {
        Self {
            patterns: Self::create_patterns(),
        }
    }

    /// Create C# patterns
    fn create_patterns() -> LanguageFunctionPatterns {
        let mut patterns = LanguageFunctionPatterns::empty();

        // Error methods
        patterns.error_functions.extend(vec![
            "throw".to_string(),
            "Exception".to_string(),
            "ArgumentException".to_string(),
            "InvalidOperationException".to_string(),
            "NotImplementedException".to_string(),
            "NotSupportedException".to_string(),
            "NullReferenceException".to_string(),
            "ArgumentNullException".to_string(),
            "ArgumentOutOfRangeException".to_string(),
            "InvalidCastException".to_string(),
            "InvalidDataException".to_string(),
            "IOException".to_string(),
            "FileNotFoundException".to_string(),
            "DirectoryNotFoundException".to_string(),
            "TimeoutException".to_string(),
        ]);

        // Format methods
        patterns.format_functions.extend(vec![
            "string.Format".to_string(),
            "Format".to_string(),
            "string.Join".to_string(),
            "Join".to_string(),
            "string.Concat".to_string(),
            "Concat".to_string(),
            "StringBuilder.AppendFormat".to_string(),
            "AppendFormat".to_string(),
        ]);

        // Log methods
        patterns.log_functions.extend(vec![
            "Console.WriteLine".to_string(),
            "Console.Write".to_string(),
            "Console.Error.WriteLine".to_string(),
            "Console.Error.Write".to_string(),
            "Debug.WriteLine".to_string(),
            "Debug.Write".to_string(),
            "Debug.Log".to_string(),
            "Debug.LogError".to_string(),
            "Debug.LogWarning".to_string(),
            "Trace.WriteLine".to_string(),
            "Trace.Write".to_string(),
            "Trace.TraceError".to_string(),
            "Trace.TraceWarning".to_string(),
            "Trace.TraceInformation".to_string(),
            "ILogger.Log".to_string(),
            "WriteLine".to_string(),
            "Write".to_string(),
            "Log".to_string(),
            "LogError".to_string(),
            "LogWarning".to_string(),
            "LogInformation".to_string(),
            "LogDebug".to_string(),
            "LogTrace".to_string(),
            "ILogger.LogInformation".to_string(),
            "ILogger.LogWarning".to_string(),
            "ILogger.LogError".to_string(),
            "ILogger.LogDebug".to_string(),
            "ILogger.LogTrace".to_string(),
            "Logger.LogInformation".to_string(),
            "Logger.LogWarning".to_string(),
            "Logger.LogError".to_string(),
            "Logger.LogDebug".to_string(),
        ]);

        // Debug methods
        patterns.debug_functions.extend(vec![
            "Debug.WriteLine".to_string(),
            "Debug.Write".to_string(),
            "Debug.Log".to_string(),
            "Debug.Assert".to_string(),
            "Debugger.Log".to_string(),
        ]);

        patterns
    }

    /// Classify a method by name
    pub fn classify_method(&self, method_name: &str) -> Option<FunctionCategory> {
        self.patterns.classify(method_name)
    }

    /// Check if method is an error method
    pub fn is_error_method(&self, method_name: &str) -> bool {
        self.patterns.is_error_function(method_name)
    }

    /// Check if method is a format method
    pub fn is_format_method(&self, method_name: &str) -> bool {
        self.patterns.is_format_function(method_name)
    }

    /// Check if method is a log method
    pub fn is_log_method(&self, method_name: &str) -> bool {
        self.patterns.is_log_function(method_name)
    }

    /// Check if method is a debug method
    pub fn is_debug_method(&self, method_name: &str) -> bool {
        self.patterns.is_debug_function(method_name)
    }

    /// Get the underlying patterns
    pub fn patterns(&self) -> &LanguageFunctionPatterns {
        &self.patterns
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

