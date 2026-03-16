//! Function Pattern Recognition Integration Tests
//!
//! Tests for function and macro classification across different languages.

use codebase_translate::parser::function_patterns::{
    classify_function, is_debug_function, is_error_function, is_format_function, is_log_function,
    FunctionCategory, FunctionPatternRegistry, LanguageFunctionPatterns,
};

mod rust_macro_tests {
    use super::*;

    #[test]
    fn test_rust_error_macros() {
        assert!(is_error_function("panic"), "panic! should be error macro");
        assert!(is_error_function("assert"), "assert! should be error macro");
        assert!(is_error_function("assert_eq"), "assert_eq! should be error macro");
        assert!(is_error_function("assert_ne"), "assert_ne! should be error macro");
        assert!(is_error_function("unreachable"), "unreachable! should be error macro");
        assert!(is_error_function("unimplemented"), "unimplemented! should be error macro");
        assert!(is_error_function("todo"), "todo! should be error macro");
    }

    #[test]
    fn test_rust_format_macros() {
        assert!(is_format_function("format"), "format! should be format macro");
        assert!(is_format_function("print"), "print! should be format macro");
        assert!(is_format_function("println"), "println! should be format macro");
        assert!(is_format_function("eprint"), "eprint! should be format macro");
        assert!(is_format_function("eprintln"), "eprintln! should be format macro");
        assert!(is_format_function("write"), "write! should be format macro");
        assert!(is_format_function("writeln"), "writeln! should be format macro");
    }

    #[test]
    fn test_rust_log_macros() {
        assert!(is_log_function("println"), "println! should be log macro");
        assert!(is_log_function("eprintln"), "eprintln! should be log macro");
    }

    #[test]
    fn test_rust_debug_macros() {
        assert!(is_debug_function("dbg"), "dbg! should be debug macro");
    }

    #[test]
    fn test_rust_non_error_functions() {
        assert!(!is_error_function("vec"), "vec! should not be error macro");
        assert!(!is_error_function("println"), "println! should not be error macro");
        assert!(!is_format_function("panic"), "panic! should not be format macro");
    }

    #[test]
    fn test_classify_rust_macros() {
        assert_eq!(
            classify_function("panic"),
            Some(FunctionCategory::Error),
            "panic! should be classified as Error"
        );
        assert_eq!(
            classify_function("format"),
            Some(FunctionCategory::Format),
            "format! should be classified as Format"
        );
        assert_eq!(
            classify_function("println"),
            Some(FunctionCategory::Log),
            "println! should be classified as Log"
        );
        assert_eq!(
            classify_function("dbg"),
            Some(FunctionCategory::Debug),
            "dbg! should be classified as Debug"
        );
        assert_eq!(
            classify_function("unknown"),
            None,
            "Unknown function should return None"
        );
    }
}

mod pattern_registry_tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = FunctionPatternRegistry::new();
        
        assert!(
            registry.get_patterns("rust").is_some(),
            "Should have Rust patterns"
        );
    }

    #[test]
    fn test_registry_empty_creation() {
        let registry = FunctionPatternRegistry::empty();
        
        assert!(
            registry.get_patterns("rust").is_none(),
            "Empty registry should not have patterns"
        );
    }

    #[test]
    fn test_registry_classify() {
        let registry = FunctionPatternRegistry::new();
        
        assert_eq!(
            registry.classify("rust", "panic"),
            Some(FunctionCategory::Error),
            "Should classify Rust panic macro"
        );
        assert_eq!(
            registry.classify("rust", "format"),
            Some(FunctionCategory::Format),
            "Should classify Rust format macro"
        );
        assert_eq!(
            registry.classify("unknown_lang", "panic"),
            None,
            "Should return None for unknown language"
        );
    }

    #[test]
    fn test_registry_is_error_function() {
        let registry = FunctionPatternRegistry::new();
        
        assert!(registry.is_error_function("rust", "panic"));
        assert!(!registry.is_error_function("rust", "format"));
        assert!(!registry.is_error_function("unknown", "panic"));
    }

    #[test]
    fn test_registry_is_format_function() {
        let registry = FunctionPatternRegistry::new();
        
        assert!(registry.is_format_function("rust", "format"));
        assert!(!registry.is_format_function("rust", "panic"));
    }

    #[test]
    fn test_registry_is_log_function() {
        let registry = FunctionPatternRegistry::new();
        
        assert!(registry.is_log_function("rust", "println"));
        assert!(!registry.is_log_function("rust", "panic"));
    }

    #[test]
    fn test_registry_is_debug_function() {
        let registry = FunctionPatternRegistry::new();
        
        assert!(registry.is_debug_function("rust", "dbg"));
        assert!(!registry.is_debug_function("rust", "panic"));
    }

    #[test]
    fn test_registry_register_patterns() {
        let mut registry = FunctionPatternRegistry::empty();
        
        let patterns = LanguageFunctionPatterns::new(
            vec!["error1".to_string(), "error2".to_string()],
            vec!["format1".to_string()],
            vec!["log1".to_string(), "log2".to_string()],
            vec!["debug1".to_string()],
        );
        
        registry.register("test_lang", patterns);
        
        assert!(registry.is_error_function("test_lang", "error1"));
        assert!(registry.is_error_function("test_lang", "error2"));
        assert!(registry.is_format_function("test_lang", "format1"));
        assert!(registry.is_log_function("test_lang", "log1"));
        assert!(registry.is_debug_function("test_lang", "debug1"));
    }

    #[test]
    fn test_registry_register_functions() {
        let mut registry = FunctionPatternRegistry::empty();
        
        registry.register_functions(
            "test_lang",
            FunctionCategory::Error,
            vec!["err1".to_string(), "err2".to_string()],
        );
        
        assert!(registry.is_error_function("test_lang", "err1"));
        assert!(registry.is_error_function("test_lang", "err2"));
    }
}

mod language_function_patterns_tests {
    use super::*;

    #[test]
    fn test_language_patterns_creation() {
        let patterns = LanguageFunctionPatterns::new(
            vec!["error1".to_string()],
            vec!["format1".to_string()],
            vec!["log1".to_string()],
            vec!["debug1".to_string()],
        );
        
        assert!(patterns.is_error_function("error1"));
        assert!(patterns.is_format_function("format1"));
        assert!(patterns.is_log_function("log1"));
        assert!(patterns.is_debug_function("debug1"));
    }

    #[test]
    fn test_language_patterns_empty() {
        let patterns = LanguageFunctionPatterns::empty();
        
        assert!(!patterns.is_error_function("anything"));
        assert!(!patterns.is_format_function("anything"));
        assert!(!patterns.is_log_function("anything"));
        assert!(!patterns.is_debug_function("anything"));
        assert_eq!(patterns.classify("anything"), None);
    }

    #[test]
    fn test_language_patterns_classify() {
        let patterns = LanguageFunctionPatterns::new(
            vec!["panic".to_string()],
            vec!["format".to_string()],
            vec!["log".to_string()],
            vec!["debug".to_string()],
        );
        
        assert_eq!(patterns.classify("panic"), Some(FunctionCategory::Error));
        assert_eq!(patterns.classify("format"), Some(FunctionCategory::Format));
        assert_eq!(patterns.classify("log"), Some(FunctionCategory::Log));
        assert_eq!(patterns.classify("debug"), Some(FunctionCategory::Debug));
        assert_eq!(patterns.classify("unknown"), None);
    }

    #[test]
    fn test_language_patterns_add_function() {
        let mut patterns = LanguageFunctionPatterns::empty();
        
        patterns.add_function(FunctionCategory::Error, "new_error".to_string());
        patterns.add_function(FunctionCategory::Format, "new_format".to_string());
        patterns.add_function(FunctionCategory::Log, "new_log".to_string());
        patterns.add_function(FunctionCategory::Debug, "new_debug".to_string());
        
        assert!(patterns.is_error_function("new_error"));
        assert!(patterns.is_format_function("new_format"));
        assert!(patterns.is_log_function("new_log"));
        assert!(patterns.is_debug_function("new_debug"));
    }

    #[test]
    fn test_language_patterns_case_sensitive() {
        let patterns = LanguageFunctionPatterns::new(
            vec!["Error".to_string()],
            vec![],
            vec![],
            vec![],
        );
        
        assert!(patterns.is_error_function("Error"));
        assert!(!patterns.is_error_function("error"));
        assert!(!patterns.is_error_function("ERROR"));
    }
}

mod function_category_tests {
    use super::*;

    #[test]
    fn test_function_category_as_str() {
        assert_eq!(FunctionCategory::Error.as_str(), "error");
        assert_eq!(FunctionCategory::Format.as_str(), "format");
        assert_eq!(FunctionCategory::Log.as_str(), "log");
        assert_eq!(FunctionCategory::Debug.as_str(), "debug");
    }

    #[test]
    fn test_function_category_display() {
        assert_eq!(format!("{}", FunctionCategory::Error), "error");
        assert_eq!(format!("{}", FunctionCategory::Format), "format");
        assert_eq!(format!("{}", FunctionCategory::Log), "log");
        assert_eq!(format!("{}", FunctionCategory::Debug), "debug");
    }

    #[test]
    fn test_function_category_equality() {
        assert_eq!(FunctionCategory::Error, FunctionCategory::Error);
        assert_ne!(FunctionCategory::Error, FunctionCategory::Format);
    }

    #[test]
    fn test_function_category_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(FunctionCategory::Error);
        set.insert(FunctionCategory::Format);
        set.insert(FunctionCategory::Log);
        set.insert(FunctionCategory::Debug);
        
        assert_eq!(set.len(), 4);
        assert!(set.contains(&FunctionCategory::Error));
    }
}

mod cross_language_tests {
    use super::*;

    #[test]
    fn test_go_error_functions() {
        let registry = FunctionPatternRegistry::new();
        
        assert!(
            registry.is_error_function("go", "Errorf") || 
            registry.get_patterns("go").is_none(),
            "Go should have Errorf or no patterns defined"
        );
    }

    #[test]
    fn test_python_error_functions() {
        let registry = FunctionPatternRegistry::new();
        
        if registry.get_patterns("python").is_some() {
            assert!(
                registry.is_error_function("python", "ValueError") ||
                registry.is_error_function("python", "raise"),
                "Python should have error functions"
            );
        }
    }

    #[test]
    fn test_java_error_functions() {
        let registry = FunctionPatternRegistry::new();
        
        if registry.get_patterns("java").is_some() {
            assert!(
                registry.is_error_function("java", "throw") ||
                registry.is_error_function("java", "Exception"),
                "Java should have error functions"
            );
        }
    }

    #[test]
    fn test_javascript_error_functions() {
        let registry = FunctionPatternRegistry::new();
        
        if registry.get_patterns("javascript").is_some() {
            assert!(
                registry.is_error_function("javascript", "Error") ||
                registry.is_error_function("javascript", "throw"),
                "JavaScript should have error functions"
            );
        }
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_function_name() {
        assert_eq!(classify_function(""), None);
        assert!(!is_error_function(""));
        assert!(!is_format_function(""));
        assert!(!is_log_function(""));
        assert!(!is_debug_function(""));
    }

    #[test]
    fn test_whitespace_function_name() {
        assert_eq!(classify_function(" "), None);
        assert_eq!(classify_function("  "), None);
        assert_eq!(classify_function("\t"), None);
    }

    #[test]
    fn test_special_characters_in_function_name() {
        assert_eq!(classify_function("panic!"), None);
        assert_eq!(classify_function("format!"), None);
        assert_eq!(classify_function("error@"), None);
    }

    #[test]
    fn test_very_long_function_name() {
        let long_name = "a".repeat(1000);
        assert_eq!(classify_function(&long_name), None);
    }

    #[test]
    fn test_unicode_function_name() {
        assert_eq!(classify_function("错误"), None);
        assert_eq!(classify_function("エラー"), None);
    }

    #[test]
    fn test_function_name_with_numbers() {
        assert_eq!(classify_function("error123"), None);
        assert_eq!(classify_function("123error"), None);
    }

    #[test]
    fn test_case_variations() {
        assert!(!is_error_function("Panic"));
        assert!(!is_error_function("PANIC"));
        assert!(!is_error_function("panic"));
        
        assert!(!is_format_function("Format"));
        assert!(!is_format_function("FORMAT"));
    }
}
