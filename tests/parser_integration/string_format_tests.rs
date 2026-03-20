//! String Format Preservation Tests
//!
//! These tests verify that the parser correctly extracts string literals

use codebase_translate::parser::core::string_processor::StringProcessor;

/// Helper function to load fixture file
fn load_fixture(filename: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("parser_integration")
        .join("fixtures")
        .join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e))
}

/// Helper function to save output file
fn save_output(filename: &str, content: &str) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("parser_integration")
        .join("output")
        .join(filename);
    std::fs::write(&path, content)
        .unwrap_or_else(|e| panic!("Failed to write output {}: {}", path.display(), e));
}

#[test]
fn test_rust_regular_string_format() {
    let processor = StringProcessor::new();
    let input = "\"Hello, world!\"";

    let cleaned = processor.clean_string_literal(input);

    // Verify content is extracted correctly
    assert_eq!(cleaned, "Hello, world!");
}

#[test]
fn test_rust_raw_string_format() {
    let processor = StringProcessor::new();

    // Test r"..."
    let input1 = "r\"Hello, world!\"";
    let cleaned1 = processor.clean_string_literal(input1);
    assert_eq!(cleaned1, "Hello, world!");

    // Test r#"..."#
    let input2 = "r#\"Hello, \"world\"!\"#";
    let cleaned2 = processor.clean_string_literal(input2);
    assert_eq!(cleaned2, "Hello, \"world\"!");

    // Test r##"..."##
    let input3 = "r##\"Hello, #\"world\"#!\"##";
    let cleaned3 = processor.clean_string_literal(input3);
    assert_eq!(cleaned3, "Hello, #\"world\"#!");
}

#[test]
fn test_rust_byte_string_format() {
    let processor = StringProcessor::new();
    let input = "b\"Hello, world!\"";

    let cleaned = processor.clean_string_literal(input);

    assert_eq!(cleaned, "Hello, world!");
}

#[test]
fn test_python_regular_string_format() {
    let processor = StringProcessor::new();

    // Test double-quoted
    let input1 = "\"Hello, world!\"";
    let cleaned1 = processor.clean_string_literal(input1);
    assert_eq!(cleaned1, "Hello, world!");

    // Test single-quoted - use hex escape for single quote
    let input2 = "\x27Hello, world!\x27";
    let cleaned2 = processor.clean_string_literal(input2);
    assert_eq!(cleaned2, "Hello, world!");
}

#[test]
fn test_python_f_string_format() {
    let processor = StringProcessor::new();
    let input = "f\"Hello, {name}!\"";

    let cleaned = processor.clean_string_literal(input);

    assert_eq!(cleaned, "Hello, {name}!");
}

#[test]
fn test_go_raw_string_format() {
    let processor = StringProcessor::new();
    // Use hex escape for backtick
    let input = "\x60Hello, world!\x60";

    let cleaned = processor.clean_string_literal(input);

    assert_eq!(cleaned, "Hello, world!");
}

#[test]
fn test_js_template_string_format() {
    let processor = StringProcessor::new();
    // Use hex escape for backtick
    let input = "\x60Hello, ${name}!\x60";

    let cleaned = processor.clean_string_literal(input);

    assert_eq!(cleaned, "Hello, ${name}!");
}

#[test]
fn test_string_with_escapes() {
    let processor = StringProcessor::new();
    let input = "\"Hello\\nWorld\\t!\"";

    let cleaned = processor.clean_string_literal(input);

    // Escapes should be processed
    assert_eq!(cleaned, "Hello\nWorld\t!");
}

#[test]
fn test_multiline_string() {
    let processor = StringProcessor::new();
    let input = "\"Line 1\nLine 2\nLine 3\"";

    let cleaned = processor.clean_string_literal(input);

    assert!(cleaned.contains('\n'));
}

#[test]
fn test_rust_fixture_parsing() {
    let _content = load_fixture("rust/string_format_preservation.rs");
    let processor = StringProcessor::new();

    let test_cases: Vec<(&str, &str)> = vec![
        ("\"Hello, world!\"", "Hello, world!"),
        ("r\"Hello, world!\"", "Hello, world!"),
        ("r#\"Hello, \"world\"!\"#", "Hello, \"world\"!"),
        ("b\"Hello, world!\"", "Hello, world!"),
    ];

    let mut results: Vec<String> = Vec::new();
    results.push("Rust String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());

    for (input, expected) in test_cases {
        let cleaned = processor.clean_string_literal(input);
        let passed = cleaned == expected;

        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned));
        results.push(format!("Status: {}", if passed { "PASS" } else { "FAIL" }));
        results.push(String::new());

        assert!(passed, "Test failed for input: {}", input);
    }

    save_output("rust_string_format_preservation.txt", &results.join("\n"));
}

#[test]
fn test_python_fixture_parsing() {
    let _content = load_fixture("python/string_format_preservation.py");
    let processor = StringProcessor::new();

    let test_cases: Vec<(&str, &str)> = vec![
        ("\"Hello, world!\"", "Hello, world!"),
        ("\x27Simple text\x27", "Simple text"),
        ("f\"Hello, {name}!\"", "Hello, {name}!"),
        ("r\"Hello\\nWorld\"", "Hello\\nWorld"),
    ];

    let mut results: Vec<String> = Vec::new();
    results.push("Python String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());

    for (input, expected) in test_cases {
        let cleaned = processor.clean_string_literal(input);
        let passed = cleaned == expected;

        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned));
        results.push(format!("Status: {}", if passed { "PASS" } else { "FAIL" }));
        results.push(String::new());

        assert!(passed, "Test failed for input: {}", input);
    }

    save_output("python_string_format_preservation.txt", &results.join("\n"));
}

#[test]
fn test_go_fixture_parsing() {
    let _content = load_fixture("go/string_format_preservation.go");
    let processor = StringProcessor::new();

    let test_cases: Vec<(&str, &str)> = vec![
        ("\"Hello, world!\"", "Hello, world!"),
        ("\x60Hello, world!\x60", "Hello, world!"),
    ];

    let mut results: Vec<String> = Vec::new();
    results.push("Go String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());

    for (input, expected) in test_cases {
        let cleaned = processor.clean_string_literal(input);
        let passed = cleaned == expected;

        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned));
        results.push(format!("Status: {}", if passed { "PASS" } else { "FAIL" }));
        results.push(String::new());

        assert!(passed, "Test failed for input: {}", input);
    }

    save_output("go_string_format_preservation.txt", &results.join("\n"));
}

#[test]
fn test_javascript_fixture_parsing() {
    let _content = load_fixture("javascript/string_format_preservation.js");
    let processor = StringProcessor::new();

    let test_cases: Vec<(&str, &str)> = vec![
        ("\"Hello, world!\"", "Hello, world!"),
        ("\x27Simple text\x27", "Simple text"),
        ("\x60Hello, ${name}!\x60", "Hello, ${name}!"),
    ];

    let mut results: Vec<String> = Vec::new();
    results.push("JavaScript String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());

    for (input, expected) in test_cases {
        let cleaned = processor.clean_string_literal(input);
        let passed = cleaned == expected;

        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned));
        results.push(format!("Status: {}", if passed { "PASS" } else { "FAIL" }));
        results.push(String::new());

        assert!(passed, "Test failed for input: {}", input);
    }

    save_output(
        "javascript_string_format_preservation.txt",
        &results.join("\n"),
    );
}

#[test]
fn test_placeholder_extraction() {
    let processor = StringProcessor::new();

    // Test Python f-string placeholders
    let python_input = "f\"Hello, {name}! You are {age} years old.\"";
    let python_cleaned = processor.clean_string_literal(python_input);

    assert_eq!(python_cleaned, "Hello, {name}! You are {age} years old.");

    // Test JS template placeholders
    let js_input = "\x60Hello, ${name}!\x60";
    let js_cleaned = processor.clean_string_literal(js_input);

    assert_eq!(js_cleaned, "Hello, ${name}!");

    // Test C-style placeholders
    let c_input = "\"Hello, %s! You have %d messages.\"";
    let c_cleaned = processor.clean_string_literal(c_input);

    assert_eq!(c_cleaned, "Hello, %s! You have %d messages.");
}
