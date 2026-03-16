//! Function patterns module
//!
//! This module provides function and macro classification for different programming languages.
//! It categorizes functions into error, format, and log categories to help determine
//! how to handle string arguments.

use std::collections::HashMap;
use std::sync::Arc;

/// Function category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    /// Error-related functions/macros
    Error,
    /// Formatting functions/macros
    Format,
    /// Logging functions/macros
    Log,
    /// Debug functions/macros
    Debug,
}

impl FunctionCategory {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Format => "format",
            Self::Log => "log",
            Self::Debug => "debug",
        }
    }
}

impl std::fmt::Display for FunctionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Function patterns for a specific language
#[derive(Debug, Clone)]
pub struct LanguageFunctionPatterns {
    /// Error functions
    pub error_functions: Vec<String>,
    /// Format functions
    pub format_functions: Vec<String>,
    /// Log functions
    pub log_functions: Vec<String>,
    /// Debug functions
    pub debug_functions: Vec<String>,
}

impl LanguageFunctionPatterns {
    /// Create new language function patterns
    pub fn new(
        error: Vec<String>,
        format: Vec<String>,
        log: Vec<String>,
        debug: Vec<String>,
    ) -> Self {
        Self {
            error_functions: error,
            format_functions: format,
            log_functions: log,
            debug_functions: debug,
        }
    }

    /// Create an empty pattern set
    pub fn empty() -> Self {
        Self {
            error_functions: Vec::new(),
            format_functions: Vec::new(),
            log_functions: Vec::new(),
            debug_functions: Vec::new(),
        }
    }

    /// Classify a function by name
    pub fn classify(&self, func_name: &str) -> Option<FunctionCategory> {
        if self.error_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Error)
        } else if self.format_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Format)
        } else if self.log_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Log)
        } else if self.debug_functions.iter().any(|f| f == func_name) {
            Some(FunctionCategory::Debug)
        } else {
            None
        }
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, func_name: &str) -> bool {
        self.error_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, func_name: &str) -> bool {
        self.format_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, func_name: &str) -> bool {
        self.log_functions.iter().any(|f| f == func_name)
    }

    /// Check if function is a debug function
    pub fn is_debug_function(&self, func_name: &str) -> bool {
        self.debug_functions.iter().any(|f| f == func_name)
    }

    /// Add a function to a category
    pub fn add_function(&mut self, category: FunctionCategory, func_name: String) {
        match category {
            FunctionCategory::Error => self.error_functions.push(func_name),
            FunctionCategory::Format => self.format_functions.push(func_name),
            FunctionCategory::Log => self.log_functions.push(func_name),
            FunctionCategory::Debug => self.debug_functions.push(func_name),
        }
    }
}

/// Function pattern registry
#[derive(Clone)]
pub struct FunctionPatternRegistry {
    patterns: HashMap<String, LanguageFunctionPatterns>,
}

impl FunctionPatternRegistry {
    /// Create a new function pattern registry with default patterns
    pub fn new() -> Self {
        let mut registry = Self {
            patterns: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Create an empty registry without defaults
    pub fn empty() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Register multiple functions for a category
    pub fn register_functions(
        &mut self,
        language: &str,
        category: FunctionCategory,
        functions: &[&str],
    ) {
        let patterns = self
            .patterns
            .entry(language.to_lowercase())
            .or_insert_with(LanguageFunctionPatterns::empty);

        for func in functions {
            match category {
                FunctionCategory::Error => patterns.error_functions.push(func.to_string()),
                FunctionCategory::Format => patterns.format_functions.push(func.to_string()),
                FunctionCategory::Log => patterns.log_functions.push(func.to_string()),
                FunctionCategory::Debug => patterns.debug_functions.push(func.to_string()),
            }
        }
    }

    /// Register default patterns for all supported languages
    fn register_defaults(&mut self) {
        // Rust
        self.register(
            "rust",
            LanguageFunctionPatterns::new(
                vec![
                    "panic!".to_string(),
                    "assert!".to_string(),
                    "assert_eq!".to_string(),
                    "assert_ne!".to_string(),
                    "unreachable!".to_string(),
                    "unimplemented!".to_string(),
                    "todo!".to_string(),
                ],
                vec![
                    "format!".to_string(),
                    "print!".to_string(),
                    "println!".to_string(),
                    "eprint!".to_string(),
                    "eprintln!".to_string(),
                    "write!".to_string(),
                    "writeln!".to_string(),
                ],
                vec!["println!".to_string(), "eprintln!".to_string()],
                vec!["dbg!".to_string()],
            ),
        );

        // Go
        self.register(
            "go",
            LanguageFunctionPatterns::new(
                vec![
                    "fmt.Errorf".to_string(),
                    "errors.New".to_string(),
                    "log.Fatal".to_string(),
                    "log.Panic".to_string(),
                    "log.Fatalf".to_string(),
                    "log.Panicf".to_string(),
                    "panic".to_string(),
                ],
                vec![
                    "fmt.Sprintf".to_string(),
                    "fmt.Printf".to_string(),
                    "fmt.Fprintf".to_string(),
                    "fmt.Errorf".to_string(),
                    "log.Printf".to_string(),
                    "log.Fatalf".to_string(),
                    "log.Panicf".to_string(),
                ],
                vec![
                    "log.Print".to_string(),
                    "log.Println".to_string(),
                    "log.Printf".to_string(),
                ],
                vec![],
            ),
        );

        // Python
        self.register(
            "python",
            LanguageFunctionPatterns::new(
                vec![
                    "raise".to_string(),
                    "Exception".to_string(),
                    "ValueError".to_string(),
                    "TypeError".to_string(),
                    "RuntimeError".to_string(),
                    "AssertionError".to_string(),
                ],
                vec![
                    "str.format".to_string(),
                    "format".to_string(),
                    "print".to_string(),
                ],
                vec![
                    "logging.debug".to_string(),
                    "logging.info".to_string(),
                    "logging.warning".to_string(),
                    "logging.error".to_string(),
                    "logging.critical".to_string(),
                    "print".to_string(),
                ],
                vec![],
            ),
        );

        // JavaScript/TypeScript
        self.register(
            "javascript",
            LanguageFunctionPatterns::new(
                vec![
                    "throw".to_string(),
                    "Error".to_string(),
                    "TypeError".to_string(),
                    "ReferenceError".to_string(),
                    "RangeError".to_string(),
                    "SyntaxError".to_string(),
                ],
                vec!["console.log".to_string(), "console.info".to_string()],
                vec![
                    "console.log".to_string(),
                    "console.info".to_string(),
                    "console.warn".to_string(),
                    "console.error".to_string(),
                    "console.debug".to_string(),
                ],
                vec!["console.debug".to_string()],
            ),
        );

        // Java
        self.register(
            "java",
            LanguageFunctionPatterns::new(
                vec![
                    "throw".to_string(),
                    "IllegalArgumentException".to_string(),
                    "IllegalStateException".to_string(),
                    "RuntimeException".to_string(),
                    "Exception".to_string(),
                ],
                vec![
                    "String.format".to_string(),
                    "System.out.printf".to_string(),
                    "System.out.format".to_string(),
                ],
                vec![
                    "System.out.print".to_string(),
                    "System.out.println".to_string(),
                    "System.err.print".to_string(),
                    "System.err.println".to_string(),
                    "Logger.info".to_string(),
                    "Logger.debug".to_string(),
                    "Logger.warn".to_string(),
                    "Logger.error".to_string(),
                ],
                vec![],
            ),
        );

        // C/C++
        self.register(
            "cpp",
            LanguageFunctionPatterns::new(
                vec![
                    "std::runtime_error".to_string(),
                    "std::logic_error".to_string(),
                    "std::exception".to_string(),
                    "throw".to_string(),
                    "exit".to_string(),
                    "abort".to_string(),
                    "perror".to_string(),
                    "strerror".to_string(),
                ],
                vec![
                    "printf".to_string(),
                    "fprintf".to_string(),
                    "sprintf".to_string(),
                    "snprintf".to_string(),
                    "scanf".to_string(),
                    "fscanf".to_string(),
                    "sscanf".to_string(),
                    "std::format".to_string(),
                    "fmt::format".to_string(),
                ],
                vec![
                    "syslog".to_string(),
                    "std::clog".to_string(),
                    "std::cerr".to_string(),
                    "std::cout".to_string(),
                ],
                vec![],
            ),
        );
    }

    /// Register patterns for a language
    pub fn register(&mut self, language: &str, patterns: LanguageFunctionPatterns) {
        self.patterns.insert(language.to_lowercase(), patterns);
    }

    /// Get patterns for a language
    pub fn get(&self, language: &str) -> Option<&LanguageFunctionPatterns> {
        self.patterns.get(&language.to_lowercase())
    }

    /// Classify a function for a specific language
    pub fn classify(&self, language: &str, func_name: &str) -> Option<FunctionCategory> {
        self.get(language)?.classify(func_name)
    }

    /// Check if function is an error function
    pub fn is_error_function(&self, language: &str, func_name: &str) -> bool {
        self.get(language)
            .map(|p| p.is_error_function(func_name))
            .unwrap_or(false)
    }

    /// Check if function is a format function
    pub fn is_format_function(&self, language: &str, func_name: &str) -> bool {
        self.get(language)
            .map(|p| p.is_format_function(func_name))
            .unwrap_or(false)
    }

    /// Check if function is a log function
    pub fn is_log_function(&self, language: &str, func_name: &str) -> bool {
        self.get(language)
            .map(|p| p.is_log_function(func_name))
            .unwrap_or(false)
    }

    /// Check if function is a debug function
    pub fn is_debug_function(&self, language: &str, func_name: &str) -> bool {
        self.get(language)
            .map(|p| p.is_debug_function(func_name))
            .unwrap_or(false)
    }

    /// Add a custom function pattern
    pub fn add_function_pattern(
        &mut self,
        language: &str,
        category: FunctionCategory,
        func_name: String,
    ) {
        let language = language.to_lowercase();
        if let Some(patterns) = self.patterns.get_mut(&language) {
            patterns.add_function(category, func_name);
        } else {
            let mut patterns = LanguageFunctionPatterns::new(vec![], vec![], vec![], vec![]);
            patterns.add_function(category, func_name);
            self.patterns.insert(language, patterns);
        }
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<&String> {
        self.patterns.keys().collect()
    }

    /// Check if language is supported
    pub fn is_supported(&self, language: &str) -> bool {
        self.patterns.contains_key(&language.to_lowercase())
    }
}

impl Default for FunctionPatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global function pattern registry instance
pub fn get_global_registry() -> Arc<FunctionPatternRegistry> {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<Arc<FunctionPatternRegistry>> = OnceLock::new();
    INSTANCE
        .get_or_init(|| Arc::new(FunctionPatternRegistry::new()))
        .clone()
}

/// Create a new function pattern registry
pub fn create_registry() -> FunctionPatternRegistry {
    FunctionPatternRegistry::new()
}

/// Classify a function using the global registry
pub fn classify_function(language: &str, func_name: &str) -> Option<FunctionCategory> {
    get_global_registry().classify(language, func_name)
}

/// Check if function is an error function using the global registry
pub fn is_error_function(language: &str, func_name: &str) -> bool {
    get_global_registry().is_error_function(language, func_name)
}

/// Check if function is a format function using the global registry
pub fn is_format_function(language: &str, func_name: &str) -> bool {
    get_global_registry().is_format_function(language, func_name)
}

/// Check if function is a log function using the global registry
pub fn is_log_function(language: &str, func_name: &str) -> bool {
    get_global_registry().is_log_function(language, func_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_category_display() {
        assert_eq!(FunctionCategory::Error.to_string(), "error");
        assert_eq!(FunctionCategory::Format.to_string(), "format");
        assert_eq!(FunctionCategory::Log.to_string(), "log");
        assert_eq!(FunctionCategory::Debug.to_string(), "debug");
    }

    #[test]
    fn test_language_function_patterns() {
        let patterns = LanguageFunctionPatterns::new(
            vec!["error1".to_string()],
            vec!["format1".to_string()],
            vec!["log1".to_string()],
            vec!["debug1".to_string()],
        );

        assert_eq!(patterns.classify("error1"), Some(FunctionCategory::Error));
        assert_eq!(patterns.classify("format1"), Some(FunctionCategory::Format));
        assert_eq!(patterns.classify("log1"), Some(FunctionCategory::Log));
        assert_eq!(patterns.classify("debug1"), Some(FunctionCategory::Debug));
        assert_eq!(patterns.classify("unknown"), None);

        assert!(patterns.is_error_function("error1"));
        assert!(!patterns.is_error_function("format1"));
    }

    #[test]
    fn test_function_pattern_registry() {
        let registry = FunctionPatternRegistry::new();

        // Test Rust patterns
        assert_eq!(
            registry.classify("rust", "panic!"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            registry.classify("rust", "format!"),
            Some(FunctionCategory::Format)
        );
        // println! is both format and log, classify returns first match (Format)
        assert!(
            registry.classify("rust", "println!") == Some(FunctionCategory::Format)
                || registry.classify("rust", "println!") == Some(FunctionCategory::Log)
        );
        assert_eq!(
            registry.classify("rust", "dbg!"),
            Some(FunctionCategory::Debug)
        );

        // Test Go patterns
        assert_eq!(
            registry.classify("go", "fmt.Errorf"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            registry.classify("go", "fmt.Sprintf"),
            Some(FunctionCategory::Format)
        );

        // Test unsupported language
        assert_eq!(registry.classify("unknown", "func"), None);
    }

    #[test]
    fn test_is_functions() {
        let registry = FunctionPatternRegistry::new();

        assert!(registry.is_error_function("rust", "panic!"));
        assert!(registry.is_format_function("rust", "format!"));
        assert!(registry.is_log_function("rust", "println!"));
        assert!(registry.is_debug_function("rust", "dbg!"));

        assert!(!registry.is_error_function("rust", "format!"));
        assert!(!registry.is_error_function("unknown", "panic!"));
    }

    #[test]
    fn test_add_function_pattern() {
        let mut registry = FunctionPatternRegistry::new();

        registry.add_function_pattern("rust", FunctionCategory::Error, "my_error!".to_string());

        assert!(registry.is_error_function("rust", "my_error!"));
        assert_eq!(
            registry.classify("rust", "my_error!"),
            Some(FunctionCategory::Error)
        );
    }

    #[test]
    fn test_add_to_new_language() {
        let mut registry = FunctionPatternRegistry::new();

        registry.add_function_pattern("kotlin", FunctionCategory::Log, "println".to_string());

        assert!(registry.is_log_function("kotlin", "println"));
        assert!(registry.is_supported("kotlin"));
    }

    #[test]
    fn test_supported_languages() {
        let registry = FunctionPatternRegistry::new();
        let languages = registry.supported_languages();

        assert!(languages.iter().any(|l| l.as_str() == "rust"));
        assert!(languages.iter().any(|l| l.as_str() == "go"));
        assert!(languages.iter().any(|l| l.as_str() == "python"));
        assert!(languages.iter().any(|l| l.as_str() == "javascript"));
        assert!(languages.iter().any(|l| l.as_str() == "java"));
        assert!(languages.iter().any(|l| l.as_str() == "cpp"));
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_global_registry();
        let registry2 = get_global_registry();

        // Should be the same instance
        assert!(Arc::ptr_eq(&registry1, &registry2));

        // Should have default patterns
        assert!(registry1.is_error_function("rust", "panic!"));
    }

    #[test]
    fn test_classify_function_helper() {
        assert_eq!(
            classify_function("rust", "panic!"),
            Some(FunctionCategory::Error)
        );
        assert!(is_error_function("rust", "panic!"));
        assert!(is_format_function("rust", "format!"));
        assert!(is_log_function("rust", "println!"));
    }

    #[test]
    fn test_rust_patterns() {
        let registry = FunctionPatternRegistry::new();

        // Error macros
        assert!(registry.is_error_function("rust", "panic!"));
        assert!(registry.is_error_function("rust", "assert!"));
        assert!(registry.is_error_function("rust", "assert_eq!"));
        assert!(registry.is_error_function("rust", "assert_ne!"));
        assert!(registry.is_error_function("rust", "unreachable!"));
        assert!(registry.is_error_function("rust", "unimplemented!"));
        assert!(registry.is_error_function("rust", "todo!"));

        // Format macros
        assert!(registry.is_format_function("rust", "format!"));
        assert!(registry.is_format_function("rust", "print!"));
        assert!(registry.is_format_function("rust", "println!"));
        assert!(registry.is_format_function("rust", "eprint!"));
        assert!(registry.is_format_function("rust", "eprintln!"));
        assert!(registry.is_format_function("rust", "write!"));
        assert!(registry.is_format_function("rust", "writeln!"));

        // Log macros
        assert!(registry.is_log_function("rust", "println!"));
        assert!(registry.is_log_function("rust", "eprintln!"));

        // Debug macros
        assert!(registry.is_debug_function("rust", "dbg!"));
    }

    #[test]
    fn test_go_patterns() {
        let registry = FunctionPatternRegistry::new();

        // Error functions
        assert!(registry.is_error_function("go", "fmt.Errorf"));
        assert!(registry.is_error_function("go", "errors.New"));
        assert!(registry.is_error_function("go", "log.Fatal"));
        assert!(registry.is_error_function("go", "panic"));

        // Format functions
        assert!(registry.is_format_function("go", "fmt.Sprintf"));
        assert!(registry.is_format_function("go", "fmt.Printf"));
        assert!(registry.is_format_function("go", "fmt.Fprintf"));

        // Log functions
        assert!(registry.is_log_function("go", "log.Print"));
        assert!(registry.is_log_function("go", "log.Println"));
        assert!(registry.is_log_function("go", "log.Printf"));
    }
}
