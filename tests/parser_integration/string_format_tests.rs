//! String Format Preservation Tests
//!
//! These tests verify that the parser correctly extracts string literals
//! while preserving their format information (quotes, raw string markers, etc.)

use codebase_translate::parser::core::string_processor::StringProcessor;
use codebase_translate::core::models::{StringStyle, FormatPlaceholder};

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
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    // Verify content is extracted correctly
    assert_eq!(cleaned.text, "Hello, world!");
    
    // Verify format info is preserved
    let format_info = cleaned.format_info;
    assert!(format_info.string_style.is_some());
    assert_eq!(format_info.string_style.unwrap(), StringStyle::DoubleQuoted);
    assert_eq!(format_info.quote_char, Some('\"'));
}

#[test]
fn test_rust_raw_string_format() {
    let processor = StringProcessor::new();
    
    // Test r"..."
    let input1 = "r\"Hello, world!\"";
    let cleaned1 = processor.clean_string_literal_with_format(input1);
    assert_eq!(cleaned1.text, "Hello, world!");
    assert_eq!(cleaned1.format_info.string_style.unwrap(), StringStyle::Raw { hash_count: 0 });
    
    // Test r#"..."#
    let input2 = "r#\"Hello, \"world\"!\"#";
    let cleaned2 = processor.clean_string_literal_with_format(input2);
    assert_eq!(cleaned2.text, "Hello, \"world\"!");
    assert_eq!(cleaned2.format_info.string_style.unwrap(), StringStyle::Raw { hash_count: 1 });
    
    // Test r##"..."##
    let input3 = "r##\"Hello, #\"world\"#!\"##";
    let cleaned3 = processor.clean_string_literal_with_format(input3);
    assert_eq!(cleaned3.text, "Hello, #\"world\"#!");
    assert_eq!(cleaned3.format_info.string_style.unwrap(), StringStyle::Raw { hash_count: 2 });
}

#[test]
fn test_rust_byte_string_format() {
    let processor = StringProcessor::new();
    let input = "b\"Hello, world!\"";
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    assert_eq!(cleaned.text, "Hello, world!");
    assert_eq!(cleaned.format_info.string_style.unwrap(), StringStyle::ByteString);
}

#[test]
fn test_python_regular_string_format() {
    let processor = StringProcessor::new();
    
    // Test double-quoted
    let input1 = "\"Hello, world!\"";
    let cleaned1 = processor.clean_string_literal_with_format(input1);
    assert_eq!(cleaned1.text, "Hello, world!");
    assert_eq!(cleaned1.format_info.string_style.unwrap(), StringStyle::DoubleQuoted);
    
    // Test single-quoted - use hex escape for single quote
    let input2 = "\x27Hello, world!\x27";
    let cleaned2 = processor.clean_string_literal_with_format(input2);
    assert_eq!(cleaned2.text, "Hello, world!");
    assert_eq!(cleaned2.format_info.string_style.unwrap(), StringStyle::SingleQuoted);
}

#[test]
fn test_python_f_string_format() {
    let processor = StringProcessor::new();
    let input = "f\"Hello, {name}!\"";
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    assert_eq!(cleaned.text, "Hello, {name}!");
    assert_eq!(cleaned.format_info.string_style.unwrap(), StringStyle::Formatted);
    
    // Check placeholders are extracted
    assert!(!cleaned.placeholders.is_empty());
    assert!(matches!(&cleaned.placeholders[0], FormatPlaceholder::FString(s) if s == "name"));
}

#[test]
fn test_go_raw_string_format() {
    let processor = StringProcessor::new();
    // Use hex escape for backtick
    let input = "\x60Hello, world!\x60";
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    assert_eq!(cleaned.text, "Hello, world!");
    assert_eq!(cleaned.format_info.string_style.unwrap(), StringStyle::Backtick);
    assert_eq!(cleaned.format_info.quote_char, Some('\x60'));
}

#[test]
fn test_js_template_string_format() {
    let processor = StringProcessor::new();
    // Use hex escape for backtick
    let input = "\x60Hello, ${name}!\x60";
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    assert_eq!(cleaned.text, "Hello, ${name}!");
    assert_eq!(cleaned.format_info.string_style.unwrap(), StringStyle::Template);
    
    // Check placeholders are extracted
    assert!(!cleaned.placeholders.is_empty());
    assert!(matches!(&cleaned.placeholders[0], FormatPlaceholder::JSTemplate(s) if s == "name"));
}

#[test]
fn test_string_with_escapes() {
    let processor = StringProcessor::new();
    let input = "\"Hello\\nWorld\\t!\"";
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    // Escapes should be processed
    assert_eq!(cleaned.text, "Hello\nWorld\t!");
}

#[test]
fn test_multiline_string() {
    let processor = StringProcessor::new();
    let input = "\"Line 1\nLine 2\nLine 3\"";
    
    let cleaned = processor.clean_string_literal_with_format(input);
    
    assert!(cleaned.text.contains('\n'));
    assert!(cleaned.format_info.is_multiline);
}

#[test]
fn test_rust_fixture_parsing() {
    let _content = load_fixture("rust/string_format_preservation.rs");
    let processor = StringProcessor::new();
    
    // Test cases from the fixture
    let test_cases: Vec<(&str, &str, StringStyle)> = vec![
        ("\"Hello, world!\"", "Hello, world!", StringStyle::DoubleQuoted),
        ("r\"Hello, world!\"", "Hello, world!", StringStyle::Raw { hash_count: 0 }),
        ("r#\"Hello, \"world\"!\"#", "Hello, \"world\"!", StringStyle::Raw { hash_count: 1 }),
        ("b\"Hello, world!\"", "Hello, world!", StringStyle::ByteString),
    ];
    
    let mut results: Vec<String> = Vec::new();
    results.push("Rust String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());
    
    for (input, expected, expected_style) in test_cases {
        let cleaned = processor.clean_string_literal_with_format(input);
        let passed = cleaned.text == expected && cleaned.format_info.string_style == Some(expected_style);
        
        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned.text));
        results.push(format!("Style: {:?}", cleaned.format_info.string_style));
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
    
    let test_cases: Vec<(&str, &str, StringStyle)> = vec![
        ("\"Hello, world!\"", "Hello, world!", StringStyle::DoubleQuoted),
        ("\x27Simple text\x27", "Simple text", StringStyle::SingleQuoted),
        ("f\"Hello, {name}!\"", "Hello, {name}!", StringStyle::Formatted),
        ("r\"Hello\\nWorld\"", "Hello\\nWorld", StringStyle::Raw { hash_count: 0 }),
    ];
    
    let mut results: Vec<String> = Vec::new();
    results.push("Python String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());
    
    for (input, expected, expected_style) in test_cases {
        let cleaned = processor.clean_string_literal_with_format(input);
        let passed = cleaned.text == expected && cleaned.format_info.string_style == Some(expected_style);
        
        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned.text));
        results.push(format!("Style: {:?}", cleaned.format_info.string_style));
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
    
    let test_cases: Vec<(&str, &str, StringStyle)> = vec![
        ("\"Hello, world!\"", "Hello, world!", StringStyle::DoubleQuoted),
        ("\x60Hello, world!\x60", "Hello, world!", StringStyle::Backtick),
    ];
    
    let mut results: Vec<String> = Vec::new();
    results.push("Go String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());
    
    for (input, expected, expected_style) in test_cases {
        let cleaned = processor.clean_string_literal_with_format(input);
        let passed = cleaned.text == expected && cleaned.format_info.string_style == Some(expected_style);
        
        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned.text));
        results.push(format!("Style: {:?}", cleaned.format_info.string_style));
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
    
    let test_cases: Vec<(&str, &str, StringStyle)> = vec![
        ("\"Hello, world!\"", "Hello, world!", StringStyle::DoubleQuoted),
        ("\x27Simple text\x27", "Simple text", StringStyle::SingleQuoted),
        ("\x60Hello, ${name}!\x60", "Hello, ${name}!", StringStyle::Template),
    ];
    
    let mut results: Vec<String> = Vec::new();
    results.push("JavaScript String Format Preservation Test Results".to_string());
    results.push("=".repeat(50));
    results.push(String::new());
    
    for (input, expected, expected_style) in test_cases {
        let cleaned = processor.clean_string_literal_with_format(input);
        let passed = cleaned.text == expected && cleaned.format_info.string_style == Some(expected_style);
        
        results.push(format!("Input: {}", input));
        results.push(format!("Expected: {}", expected));
        results.push(format!("Got: {}", cleaned.text));
        results.push(format!("Style: {:?}", cleaned.format_info.string_style));
        results.push(format!("Status: {}", if passed { "PASS" } else { "FAIL" }));
        results.push(String::new());
        
        assert!(passed, "Test failed for input: {}", input);
    }
    
    save_output("javascript_string_format_preservation.txt", &results.join("\n"));
}

#[test]
fn test_placeholder_extraction() {
    let processor = StringProcessor::new();
    
    // Test Python f-string placeholders
    let python_input = "f\"Hello, {name}! You are {age} years old.\"";
    let python_cleaned = processor.clean_string_literal_with_format(python_input);
    
    assert_eq!(python_cleaned.placeholders.len(), 2);
    assert!(matches!(&python_cleaned.placeholders[0], FormatPlaceholder::FString(s) if s == "name"));
    assert!(matches!(&python_cleaned.placeholders[1], FormatPlaceholder::FString(s) if s == "age"));
    
    // Test JS template placeholders
    let js_input = "\x60Hello, ${name}!\x60";
    let js_cleaned = processor.clean_string_literal_with_format(js_input);
    
    assert_eq!(js_cleaned.placeholders.len(), 1);
    assert!(matches!(&js_cleaned.placeholders[0], FormatPlaceholder::JSTemplate(s) if s == "name"));
    
    // Test C-style placeholders
    let c_input = "\"Hello, %s! You have %d messages.\"";
    let c_cleaned = processor.clean_string_literal_with_format(c_input);
    
    assert_eq!(c_cleaned.placeholders.len(), 2);
    assert!(matches!(&c_cleaned.placeholders[0], FormatPlaceholder::CStyle(s) if s == "%s"));
    assert!(matches!(&c_cleaned.placeholders[1], FormatPlaceholder::CStyle(s) if s == "%d"));
}

#[test]
fn test_format_info_serialization() {
    use codebase_translate::core::models::FormatInfo;
    use codebase_translate::core::models::CommentStyle;
    
    let format_info = FormatInfo {
        style: CommentStyle::Line,
        base_indent: "    ".to_string(),
        line_prefix: Some("// ".to_string()),
        ends_with_newline: false,
        is_multiline: false,
        string_style: Some(StringStyle::DoubleQuoted),
        placeholders: Some(vec![FormatPlaceholder::FString("name".to_string())]),
        quote_char: Some('\"'),
    };
    
    // Test serialization
    let json = serde_json::to_string(&format_info).expect("Failed to serialize");
    
    // Test deserialization
    let deserialized: FormatInfo = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(deserialized.string_style, Some(StringStyle::DoubleQuoted));
    assert_eq!(deserialized.quote_char, Some('\"'));
    assert!(deserialized.placeholders.is_some());
    assert_eq!(deserialized.placeholders.as_ref().unwrap().len(), 1);
}
