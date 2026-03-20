//! TranslationApplier integration tests

use codebase_translate::core::models::{NodeType, Position, TranslationUnit};
use codebase_translate::writer::apply_translations;

use super::common::*;

#[test]
fn test_translation_applier_simple_replacement() {
    let content = "Hello world\nThis is a test";
    let mut units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];
    units[0].set_translated("你好");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("你好"));
    assert!(modified.contains("world"));
}

#[test]
fn test_translation_applier_multiple_replacements() {
    let content = "First line\nSecond line\nThird line";
    let mut units = vec![
        create_translation_unit("1", "First", 1, 1, 6),
        create_translation_unit("2", "Second", 2, 1, 7),
        create_translation_unit("3", "Third", 3, 1, 6),
    ];
    units[0].set_translated("第一");
    units[1].set_translated("第二");
    units[2].set_translated("第三");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("第一"));
    assert!(modified.contains("第二"));
    assert!(modified.contains("第三"));
}

#[test]
fn test_translation_applier_with_crlf() {
    let content = "Line 1\r\nLine 2\r\nLine 3";
    let mut units = vec![create_translation_unit("1", "Line 1", 1, 1, 7)];
    units[0].set_translated("第一行");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("\r\n"), "CRLF should be preserved");
    assert!(modified.contains("第一行"));
}

#[test]
fn test_translation_applier_empty_units() {
    let content = "Hello world\nThis is a test";
    let units = vec![];

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[test]
fn test_translation_applier_missing_translation() {
    let content = "Hello world";
    let units = vec![create_translation_unit_with_raw_match(
        "1", "Hello", "Hello", 1, 1, 6,
    )];

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[test]
fn test_translation_applier_line_comment_format() {
    let content = "    // This is a comment\nint x = 5;";

    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "This is a comment",
        "    // This is a comment",
        1,
        5,
        22,
    )];
    units[0].set_translated("这是一个注释");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("    // 这是一个注释"));
}

#[test]
fn test_translation_applier_block_comment_format() {
    let content = "/* This is a comment */\nint x = 5;";

    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "/* This is a comment */",
        "/* This is a comment */",
        1,
        1,
        22,
    )];
    units[0].set_translated("/* 这是一个注释 */");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("/* 这是一个注释 */"));
}

#[test]
fn test_translation_applier_multiline_block_comment() {
    let content = "/*\n * Line 1\n * Line 2\n */\nint x = 5;";

    // content should be cleaned (without comment markers)
    // raw_match preserves the original format
    // translated is just the translation without markers
    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "Line 1\nLine 2",
        "/*\n * Line 1\n * Line 2\n */",
        1,
        1,
        22,
    )];
    units[0].set_translated("第一行\n第二行");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    println!("Expected: /*\n * 第一行\n * 第二行\n */");
    println!("Got: {}", modified);
    assert!(modified.contains("/*\n * 第一行\n * 第二行\n */"));
}

#[test]
fn test_translation_applier_doc_outer_comment() {
    let content = "/// This is a doc comment\npub fn foo() {}";

    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "/// This is a doc comment",
        "/// This is a doc comment",
        1,
        1,
        24,
    )];
    units[0].set_translated("/// 这是一个文档注释");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("/// 这是一个文档注释"));
}

#[test]
fn test_translation_applier_doc_block_comment() {
    let content = "/**\n * This is a doc comment\n */\npub fn foo() {}";

    // content should be cleaned (without comment markers)
    // raw_match preserves the original format
    // translated is just the translation without markers
    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "This is a doc comment",
        "/**\n * This is a doc comment\n */",
        1,
        1,
        4,
    )];
    units[0].set_translated("这是一个文档注释");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    println!("Expected: /**\n * 这是一个文档注释\n */");
    println!("Got: {}", modified);
    assert!(modified.contains("/**\n * 这是一个文档注释\n */"));
}

#[test]
fn test_translation_applier_multiline_translated_text() {
    let content = "/*\n * Line 1\n * Line 2\n */\nint x = 5;";

    // content should be cleaned (without comment markers)
    // raw_match preserves the original format
    // translated is just the translation without markers
    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "Line 1\nLine 2",
        "/*\n * Line 1\n * Line 2\n */",
        1,
        1,
        22,
    )];
    units[0].set_translated("第一行\n第二行");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("/*\n * 第一行\n * 第二行\n */"));
}

#[test]
fn test_translation_applier_without_raw_match() {
    let content = "Hello world\nThis is a test";
    let mut units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];
    units[0].set_translated("你好");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("你好"));
    assert!(modified.contains("world"));
}

#[test]
fn test_translation_applier_multiple_units_on_same_line() {
    let content = "Hello world and goodbye";
    let mut units = vec![
        create_translation_unit("1", "Hello", 1, 1, 6),
        create_translation_unit("2", "goodbye", 1, 17, 24),
    ];
    units[0].set_translated("你好");
    units[1].set_translated("再见");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("你好"));
    assert!(modified.contains("再见"));
    assert!(modified.contains("world and"));
}

#[test]
fn test_translation_applier_preserves_whitespace() {
    let content = "    Hello    world    ";
    let mut units = vec![create_translation_unit("1", "Hello", 1, 5, 11)];
    units[0].set_translated("你好");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.starts_with("    "));
    assert!(modified.ends_with("    "));
    assert!(modified.contains("你好"));
}

#[test]
fn test_translation_applier_unicode_characters() {
    let content = "Hello 世界\nThis is a test";
    let mut units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];
    units[0].set_translated("你好");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("你好"));
    assert!(modified.contains("世界"));
}

#[test]
fn test_translation_applier_should_translate_false() {
    let content = "Hello world\nThis is a test";
    let mut unit = create_translation_unit("1", "Hello", 1, 1, 6);
    unit.should_translate = false;
    unit.set_translated("你好");

    let result = apply_translations(content, &[unit]);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert_eq!(
        modified, content,
        "Should not translate when should_translate is false"
    );
}

#[test]
fn test_translation_applier_complex_multiline_format() {
    let content = "    /*\n     * Line 1\n     * Line 2\n     */\n    int x = 5;";

    // content should be cleaned (without comment markers)
    // raw_match preserves the original format
    // translated is just the translation without markers
    let mut units = vec![create_translation_unit_with_raw_match(
        "1",
        "Line 1\nLine 2",
        "    /*\n     * Line 1\n     * Line 2\n     */",
        1,
        1,
        37,
    )];
    units[0].set_translated("第一行\n第二行");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("/*\n     * 第一行\n     * 第二行\n     */"));
}

#[test]
fn test_translation_applier_line_ending_normalization() {
    let content = "Line 1\r\nLine 2\r\nLine 3";
    let mut units = vec![create_translation_unit("1", "Line 1", 1, 1, 7)];
    units[0].set_translated("第一行");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("\r\n"), "Should preserve CRLF");
    assert!(modified.contains("第一行"));
}

#[test]
fn test_translation_applier_empty_content() {
    let content = "";
    let units = vec![];

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_translation_applier_single_line() {
    let content = "Hello world";
    let mut units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];
    units[0].set_translated("你好");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    assert!(modified.contains("你好"));
    assert!(modified.contains("world"));
}

#[test]
fn test_translation_applier_preserves_line_structure() {
    let content = "Line 1\nLine 2\nLine 3\nLine 4";
    let mut units = vec![
        create_translation_unit("1", "Line 1", 1, 1, 7),
        create_translation_unit("3", "Line 3", 3, 1, 7),
    ];
    units[0].set_translated("第一行");
    units[1].set_translated("第三行");

    let result = apply_translations(content, &units);
    assert!(result.is_ok());
    let modified = result.unwrap();
    let lines: Vec<&str> = modified.lines().collect();
    assert_eq!(lines.len(), 4, "Should preserve line count");
    assert!(modified.contains("第一行"));
    assert!(modified.contains("Line 2"));
    assert!(modified.contains("第三行"));
    assert!(modified.contains("Line 4"));
}

#[test]
fn test_translation_applier_multiline_with_raw_match() {
    // Test multiline comment handling with raw_match
    let content = "/*\nThis is a multi-line comment\nwith multiple lines of text\n*/\nint x = 5;";

    // Create a unit for line 2 only
    let mut unit = TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::Comment,
        content: "This is a multi-line comment".to_string(),
        start_pos: Position::new(2, 1, 0),
        end_pos: Position::new(2, 25, 0),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("This is a multi-line comment".to_string()),
    };
    unit.set_translated("这是一个多行注释");

    let result = apply_translations(content, &[unit]);
    assert!(result.is_ok());
    let modified = result.unwrap();

    // The content should be correctly replaced
    assert!(modified.contains("/*\n这是一个多行注释\nwith multiple lines of text\n*/"));
    assert!(modified.contains("int x = 5;"));
}

#[test]
fn test_translation_applier_merged_multiline_doc_comment() {
    // Test merged multiline doc comment (simulating what happens after parser merges consecutive lines)
    let content = "/// Line 1\n/// Line 2\n/// Line 3\npub fn foo() {}";

    // This simulates a merged unit from the parser - but since we're using raw_match,
    // we need to create separate units for each line
    let mut unit1 = TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::DocString,
        content: "Line 1".to_string(),
        start_pos: Position::new(1, 5, 0),
        end_pos: Position::new(1, 10, 0),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("Line 1".to_string()),
    };
    unit1.set_translated("第一行");

    let mut unit2 = TranslationUnit {
        id: "2".to_string(),
        node_type: NodeType::DocString,
        content: "Line 2".to_string(),
        start_pos: Position::new(2, 5, 0),
        end_pos: Position::new(2, 10, 0),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("Line 2".to_string()),
    };
    unit2.set_translated("第二行");

    let mut unit3 = TranslationUnit {
        id: "3".to_string(),
        node_type: NodeType::DocString,
        content: "Line 3".to_string(),
        start_pos: Position::new(3, 5, 0),
        end_pos: Position::new(3, 10, 0),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("Line 3".to_string()),
    };
    unit3.set_translated("第三行");

    let result = apply_translations(content, &[unit1, unit2, unit3]);
    assert!(result.is_ok());
    let modified = result.unwrap();

    // Should correctly replace all three lines with translated content
    assert!(modified.contains("/// 第一行"));
    assert!(modified.contains("/// 第二行"));
    assert!(modified.contains("/// 第三行"));
    assert!(modified.contains("pub fn foo() {}"));

    // Should NOT have original text
    assert!(!modified.contains("Line 1"));
    assert!(!modified.contains("Line 2"));
    assert!(!modified.contains("Line 3"));
}
