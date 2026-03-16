//! Function Pattern Recognition Integration Tests
//!
//! Tests for function and macro classification across different languages.

use codebase_translate::parser::function_patterns::FunctionCategory;
use codebase_translate::parser::languages::rust::RustPatterns;
use codebase_translate::parser::languages::javascript::JavaScriptPatterns;
use codebase_translate::parser::languages::python::PythonPatterns;
use codebase_translate::parser::languages::go::GoPatterns;
use codebase_translate::parser::languages::java::JavaPatterns;

mod rust_macro_tests {
    use super::*;

    #[test]
    fn test_rust_error_macros() {
        let patterns = RustPatterns::new();
        assert!(patterns.is_error_macro("panic!"), "panic! should be error macro");
        assert!(patterns.is_error_macro("assert!"), "assert! should be error macro");
        assert!(patterns.is_error_macro("assert_eq!"), "assert_eq! should be error macro");
        assert!(patterns.is_error_macro("assert_ne!"), "assert_ne! should be error macro");
        assert!(patterns.is_error_macro("unreachable!"), "unreachable! should be error macro");
        assert!(patterns.is_error_macro("unimplemented!"), "unimplemented! should be error macro");
        assert!(patterns.is_error_macro("todo!"), "todo! should be error macro");
    }

    #[test]
    fn test_rust_format_macros() {
        let patterns = RustPatterns::new();
        assert!(patterns.is_format_macro("format!"), "format! should be format macro");
        assert!(patterns.is_format_macro("print!"), "print! should be format macro");
        assert!(patterns.is_format_macro("println!"), "println! should be format macro");
        assert!(patterns.is_format_macro("eprint!"), "eprint! should be format macro");
        assert!(patterns.is_format_macro("eprintln!"), "eprintln! should be format macro");
        assert!(patterns.is_format_macro("write!"), "write! should be format macro");
        assert!(patterns.is_format_macro("writeln!"), "writeln! should be format macro");
    }

    #[test]
    fn test_rust_log_macros() {
        let patterns = RustPatterns::new();
        assert!(patterns.is_log_macro("println!"), "println! should be log macro");
        assert!(patterns.is_log_macro("eprintln!"), "eprintln! should be log macro");
    }

    #[test]
    fn test_rust_debug_macros() {
        let patterns = RustPatterns::new();
        assert!(patterns.is_debug_macro("dbg!"), "dbg! should be debug macro");
    }

    #[test]
    fn test_rust_non_error_functions() {
        let patterns = RustPatterns::new();
        assert!(!patterns.is_error_macro("vec!"), "vec! should not be error macro");
        assert!(!patterns.is_error_macro("println!"), "println! should not be error macro");
        assert!(!patterns.is_format_macro("panic!"), "panic! should not be format macro");
    }

    #[test]
    fn test_classify_rust_macros() {
        let patterns = RustPatterns::new();
        assert_eq!(
            patterns.classify_macro("panic!"),
            Some(FunctionCategory::Error),
            "panic! should be classified as Error"
        );
        assert_eq!(
            patterns.classify_macro("format!"),
            Some(FunctionCategory::Format),
            "format! should be classified as Format"
        );
        assert_eq!(
            patterns.classify_macro("println!"),
            Some(FunctionCategory::Log),
            "println! should be classified as Log"
        );
        assert_eq!(
            patterns.classify_macro("dbg!"),
            Some(FunctionCategory::Debug),
            "dbg! should be classified as Debug"
        );
        assert_eq!(
            patterns.classify_macro("unknown!"),
            None,
            "Unknown function should return None"
        );
    }
}

mod javascript_function_tests {
    use super::*;

    #[test]
    fn test_javascript_log_functions() {
        let patterns = JavaScriptPatterns::new();
        assert!(patterns.is_log_function("console.log"));
        assert!(patterns.is_log_function("console.error"));
        assert!(patterns.is_log_function("console.warn"));
        assert!(patterns.is_log_function("console.info"));
        assert!(patterns.is_log_function("console.debug"));
    }

    #[test]
    fn test_javascript_error_functions() {
        let patterns = JavaScriptPatterns::new();
        assert!(patterns.is_error_function("throw"));
        assert!(patterns.is_error_function("Error"));
        assert!(patterns.is_error_function("TypeError"));
        assert!(patterns.is_error_function("ReferenceError"));
    }

    #[test]
    fn test_classify_javascript_functions() {
        let patterns = JavaScriptPatterns::new();
        assert_eq!(
            patterns.classify_function("console.log"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(
            patterns.classify_function("Error"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(patterns.classify_function("unknownFunc"), None);
    }
}

mod python_function_tests {
    use super::*;

    #[test]
    fn test_python_error_functions() {
        let patterns = PythonPatterns::new();
        assert!(patterns.is_error_function("raise"));
        assert!(patterns.is_error_function("assert"));
        assert!(patterns.is_error_function("sys.exit"));
    }

    #[test]
    fn test_python_format_functions() {
        let patterns = PythonPatterns::new();
        assert!(patterns.is_format_function("print"));
        assert!(patterns.is_format_function("format"));
        assert!(patterns.is_format_function("str.format"));
    }

    #[test]
    fn test_python_log_functions() {
        let patterns = PythonPatterns::new();
        assert!(patterns.is_log_function("logging.info"));
        assert!(patterns.is_log_function("logging.error"));
        assert!(patterns.is_log_function("logger.debug"));
    }

    #[test]
    fn test_classify_python_functions() {
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
}

mod go_function_tests {
    use super::*;

    #[test]
    fn test_go_error_functions() {
        let patterns = GoPatterns::new();
        assert!(patterns.is_error_function("panic"));
        assert!(patterns.is_error_function("log.Fatal"));
        assert!(patterns.is_error_function("log.Panic"));
    }

    #[test]
    fn test_go_format_functions() {
        let patterns = GoPatterns::new();
        assert!(patterns.is_format_function("fmt.Printf"));
        assert!(patterns.is_format_function("fmt.Sprintf"));
        assert!(patterns.is_format_function("fmt.Println"));
    }

    #[test]
    fn test_go_log_functions() {
        let patterns = GoPatterns::new();
        assert!(patterns.is_log_function("log.Println"));
        assert!(patterns.is_log_function("log.Printf"));
    }

    #[test]
    fn test_classify_go_functions() {
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
}

mod java_function_tests {
    use super::*;

    #[test]
    fn test_java_error_methods() {
        let patterns = JavaPatterns::new();
        assert!(patterns.is_error_method("throw"));
        assert!(patterns.is_error_method("throws"));
    }

    #[test]
    fn test_java_format_methods() {
        let patterns = JavaPatterns::new();
        assert!(patterns.is_format_method("format"));
        assert!(patterns.is_format_method("printf"));
        assert!(patterns.is_format_method("sprintf"));
    }

    #[test]
    fn test_java_log_methods() {
        let patterns = JavaPatterns::new();
        assert!(patterns.is_log_method("println"));
        assert!(patterns.is_log_method("log"));
        assert!(patterns.is_log_method("info"));
    }

    #[test]
    fn test_classify_java_methods() {
        let patterns = JavaPatterns::new();
        assert_eq!(
            patterns.classify_method("throw"),
            Some(FunctionCategory::Error)
        );
        assert_eq!(
            patterns.classify_method("format"),
            Some(FunctionCategory::Format)
        );
        assert_eq!(
            patterns.classify_method("println"),
            Some(FunctionCategory::Log)
        );
        assert_eq!(patterns.classify_method("unknownMethod"), None);
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
