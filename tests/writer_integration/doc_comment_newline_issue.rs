//! Test to reproduce the missing newline issue in doc comments

use std::fs;

use codebase_translate::core::models::{NodeType, Position, TranslationUnit};
use codebase_translate::writer::core::TranslationApplier;

#[test]
fn test_doc_comment_missing_newline_issue() {
    // This test reproduces the issue where the newline between doc comment
    // and function declaration is lost after translation

    let content = r#"/// English documentation comment
/// @param value - The value to process
fn process_english(value: i32) -> i32 {
    value * 2
}

/// 中文文档注释
/// @param value - 要处理的值
fn process_chinese(value: i32) -> i32 {
    value * 2
}
"#;

    println!("Original content:\n{}", content);
    println!("=== Content length: {} bytes ===\n", content.len());

    // Simulate translation units from parser
    // Note: These units should represent the doc comments as extracted by the parser
    // Correct positions calculated from actual content:
    // Unit 1: Lines 1-2, byte offset 0-73 (ends before newline after line 2)
    // Unit 2: Lines 7-8, byte offset 131-188 (ends before newline after line 8)
    let mut units = vec![
        // Unit 1: English doc comment (lines 1-2)
        TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::DocString,
            content: "English documentation comment\n@param value - The value to process"
                .to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(2, 40, 73), // Line 2, col 40, byte 73 (end of line content, before newline)
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some(
                "/// English documentation comment\n/// @param value - The value to process"
                    .to_string(),
            ),
        },
        // Unit 2: Chinese doc comment (lines 7-8)
        TranslationUnit {
            id: "2".to_string(),
            node_type: NodeType::DocString,
            content: "中文文档注释\n@param value - 要处理的值".to_string(),
            start_pos: Position::new(7, 1, 131),
            end_pos: Position::new(8, 35, 188), // Line 8, col 35, byte 188 (end of line content, before newline)
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 中文文档注释\n/// @param value - 要处理的值".to_string()),
        },
    ];

    // Set translations (Chinese to English)
    units[0].set_translated("英文文档注释\n@param value - 要处理的值");
    units[1].set_translated("Chinese documentation comments\n@param value - the value to be processed");

    let result = TranslationApplier::apply_translations(content, &units);
    assert!(result.is_ok(), "apply_translations failed: {:?}", result.err());

    let modified = result.unwrap();
    println!("\n=== Modified content ===\n{}", modified);

    // Write output for inspection
    fs::create_dir_all("tests/writer_integration/output").expect("Failed to create output dir");
    fs::write(
        "tests/writer_integration/output/doc_comment_newline_issue.rs",
        &modified,
    )
    .expect("Failed to write output");

    // Check that newlines are preserved
    // The function declaration should be on a separate line after the doc comment
    assert!(
        modified.contains("/// @param value - 要处理的值\nfn process_english"),
        "Newline should be preserved between doc comment and function declaration"
    );

    assert!(
        modified.contains("/// @param value - the value to be processed\nfn process_chinese"),
        "Newline should be preserved between doc comment and function declaration"
    );

    // Check that the function declarations are not concatenated with doc comments
    assert!(
        !modified.contains("processfn"),
        "Function declaration should not be concatenated with doc comment"
    );

    println!("\n=== Test passed ===");
}

#[test]
fn test_doc_comment_with_correct_positions() {
    // Test with correct position information that includes the trailing newline

    let content = r#"/// English documentation comment
/// @param value - The value to process
fn process_english(value: i32) -> i32 {
    value * 2
}
"#;

    println!("Original content:\n{}", content);

    // Simulate translation units with correct positions
    // The end_pos should be at the end of the last doc comment line
    let mut units = vec![TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::DocString,
        content: "English documentation comment\n@param value - The value to process"
            .to_string(),
        // Note: end_pos.offset should point to the end of the last doc comment line
        // In the original content, line 2 ends at byte 85 (including the newline)
        start_pos: Position::new(1, 1, 0),
        end_pos: Position::new(2, 42, 85),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some(
            "/// English documentation comment\n/// @param value - The value to process"
                .to_string(),
        ),
    }];

    units[0].set_translated("英文文档注释\n@param value - 要处理的值");

    let result = TranslationApplier::apply_translations(content, &units);
    assert!(result.is_ok(), "apply_translations failed: {:?}", result.err());

    let modified = result.unwrap();
    println!("\n=== Modified content ===\n{}", modified);

    // Check that newline is preserved
    assert!(
        modified.contains("/// @param value - 要处理的值\nfn process_english"),
        "Newline should be preserved"
    );

    assert!(!modified.contains("processfn"), "Should not have concatenated text");
}
