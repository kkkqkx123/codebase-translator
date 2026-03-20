//! Test cases for regex/state machine raw_match preservation
//!
//! These tests verify that regex and state machine extractors
//! correctly save raw_match and use direct replacement strategy.

use codebase_translate::core::models::{NodeType, PatternType, Position, TranslationUnit};
use codebase_translate::writer::core::TranslationApplier;

#[test]
fn test_raw_match_direct_replacement() {
    // Test case: TODO pattern
    // Original: "TODO: Fix this bug"
    // Extracted: "Fix this bug"
    // Translated: "修复此问题"
    // Expected result: "TODO: 修复此问题" (preserves "TODO:" prefix)

    let content = "// TODO: Fix this bug\nfn main() {\n    println!(\"hello\");\n}";

    let mut unit = TranslationUnit::new_with_pattern(
        "test_1",
        NodeType::StringLiteral,
        "Fix this bug",
        Position::new(1, 9, 8),
        Position::new(1, 21, 20),
        PatternType::CustomRegex,
        "todo_pattern".to_string(),
    );

    // Set raw_match (this is what the coordinator should do)
    unit.raw_match = Some("TODO: Fix this bug".to_string());

    // Set translated text
    unit.set_translated("修复此问题");

    // Apply translation
    let result = TranslationApplier::apply_translations(content, &[unit]).unwrap();

    // Verify that "TODO:" prefix is preserved
    assert!(
        result.contains("TODO: 修复此问题"),
        "TODO: prefix should be preserved"
    );
    assert!(
        !result.contains("TODO: Fix this bug"),
        "Original TODO should be replaced"
    );
}

#[test]
fn test_raw_match_error_message() {
    // Test case: Error message in function call
    // Original: 'throw new Error("Invalid input")'
    // Extracted: "Invalid input"
    // Translated: "无效输入"
    // Expected result: 'throw new Error("无效输入")' (preserves function call syntax)

    let content = "function validate(input) {\n    if (!input) {\n        throw new Error(\"Invalid input\");\n    }\n}";

    let mut unit = TranslationUnit::new_with_pattern(
        "test_2",
        NodeType::StringLiteral,
        "Invalid input",
        Position::new(3, 23, 52),
        Position::new(3, 36, 65),
        PatternType::CustomRegex,
        "error_pattern".to_string(),
    );

    // Set raw_match
    unit.raw_match = Some("throw new Error(\"Invalid input\")".to_string());

    // Set translated text
    unit.set_translated("无效输入");

    // Apply translation
    let result = TranslationApplier::apply_translations(content, &[unit]).unwrap();

    // Verify that function call syntax is preserved
    assert!(
        result.contains("throw new Error(\"无效输入\")"),
        "Function call syntax should be preserved"
    );
    assert!(
        !result.contains("Invalid input"),
        "Original error message should be replaced"
    );
}

#[test]
fn test_raw_match_multiline() {
    // Test case: Multiline TODO
    // Original: Multi-line comment with TODO
    // Extracted: Multi-line text
    // Translated: Translated multi-line text
    // Expected result: Preserves comment markers and structure

    let content = "// TODO: Fix this bug\nfn main() {\n}";

    let mut unit = TranslationUnit::new_with_pattern(
        "test_3",
        NodeType::StringLiteral,
        "Fix this bug",
        Position::new(1, 9, 8),
        Position::new(1, 21, 20),
        PatternType::CustomRegex,
        "todo_pattern".to_string(),
    );

    // Set raw_match (single line for simplicity)
    unit.raw_match = Some("TODO: Fix this bug".to_string());

    // Set translated text
    unit.set_translated("修复此bug");

    // Apply translation
    let result = TranslationApplier::apply_translations(content, &[unit]).unwrap();

    // Verify that comment marker is preserved
    println!("Result: {}", result);
    assert!(
        result.contains("// TODO: 修复此bug"),
        "TODO: prefix should be preserved"
    );
}

#[test]
fn test_raw_match_strategies() {
    // Test case: Verify that raw_match strategy works correctly
    // raw_match: Used for direct replacement in the original content

    let content = "let message = \"Hello world\";\n// TODO: Fix this bug\n";

    // Unit with raw_match
    let mut unit_with_raw = TranslationUnit::new_with_pattern(
        "test_raw",
        NodeType::StringLiteral,
        "Hello world",
        Position::new(1, 15, 14),
        Position::new(1, 26, 25),
        PatternType::CustomRegex,
        "string_pattern".to_string(),
    );
    unit_with_raw.raw_match = Some("\"Hello world\"".to_string());
    unit_with_raw.set_translated("你好世界");

    // Apply translations
    let result = TranslationApplier::apply_translations(content, &[unit_with_raw]).unwrap();

    // Verify raw_match strategy works
    println!("Result: {}", result);
    assert!(
        result.contains("\"你好世界\""),
        "raw_match should preserve quotes and replace content"
    );
}

#[test]
fn test_raw_match_with_multibyte_chars() {
    // Test case: Verify that multibyte characters are handled correctly
    // Original: "TODO: 修复这个中文bug"
    // Extracted: "修复这个中文bug"
    // Translated: "Fix this Chinese bug"
    // Expected result: "TODO: Fix this Chinese bug"

    let content = "// TODO: 修复这个中文bug\nfn main() {\n}";

    let mut unit = TranslationUnit::new_with_pattern(
        "test_4",
        NodeType::StringLiteral,
        "修复这个中文bug",
        Position::new(1, 9, 8),
        Position::new(1, 23, 22),
        PatternType::CustomRegex,
        "todo_chinese_pattern".to_string(),
    );

    // Set raw_match (contains multibyte characters)
    unit.raw_match = Some("TODO: 修复这个中文bug".to_string());

    // Set translated text
    unit.set_translated("Fix this Chinese bug");

    // Apply translation
    let result = TranslationApplier::apply_translations(content, &[unit]).unwrap();

    // Verify that multibyte characters are handled correctly
    assert!(
        result.contains("TODO: Fix this Chinese bug"),
        "Multibyte characters should be handled correctly"
    );
}

#[test]
fn test_raw_match_extracted_not_found() {
    // Test case: Verify behavior when extracted text is not found in raw_match
    // This should log a warning and return raw_match as-is

    let content = "// TODO: Fix this bug\nfn main() {\n}";

    let mut unit = TranslationUnit::new_with_pattern(
        "test_5",
        NodeType::StringLiteral,
        "Different text", // This is NOT in raw_match
        Position::new(1, 9, 8),
        Position::new(1, 21, 20),
        PatternType::CustomRegex,
        "todo_pattern".to_string(),
    );

    // Set raw_match
    unit.raw_match = Some("TODO: Fix this bug".to_string());

    // Set translated text
    unit.set_translated("不同的文本");

    // Apply translation (should not crash, just log warning)
    let result = TranslationApplier::apply_translations(content, &[unit]).unwrap();

    // Verify that raw_match is preserved (since extracted text not found)
    assert!(
        result.contains("TODO: Fix this bug"),
        "raw_match should be preserved when extracted text not found"
    );
}
