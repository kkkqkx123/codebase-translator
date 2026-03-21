//! Test writer with Chinese content replacement

use std::fs;

use codebase_translate::core::models::{NodeType, Position, TranslationUnit};
use codebase_translate::writer::apply_translations;

#[test]
fn test_chinese_doc_comment_replacement_full() {
    // Test the full content from simple_rust.rs
    let content = fs::read_to_string("tests/main_integration/fixtures/simple_rust.rs")
        .expect("Failed to read fixture");

    println!("Original content:\n{}", content);
    println!("\n=== Content length: {} bytes ===\n", content.len());

    // Simulate translation units from parser (based on actual parser output)
    let mut units = vec![
        // Unit 1: Line comments
        TranslationUnit {
            id: "1".to_string(),
            node_type: NodeType::Comment,
            content: "这是一个简单的Rust文件，用于测试翻译功能\n包含中文注释和文档字符串"
                .to_string(),
            start_pos: Position::new(1, 1, 0),
            end_pos: Position::new(2, 40, 101),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some(
                "// 这是一个简单的Rust文件，用于测试翻译功能\n// 包含中文注释和文档字符串"
                    .to_string(),
            ),
        },
        // Unit 2: Doc comment for add function
        TranslationUnit {
            id: "2".to_string(),
            node_type: NodeType::DocString,
            content: "计算两个数的和\n# Arguments".to_string(),
            start_pos: Position::new(4, 1, 103),
            end_pos: Position::new(7, 1, 150),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 计算两个数的和\n/// \n/// # Arguments\n".to_string()),
        },
        // Unit 6: multiply function
        TranslationUnit {
            id: "6".to_string(),
            node_type: NodeType::DocString,
            content: "乘法运算".to_string(),
            start_pos: Position::new(31, 1, 503),
            end_pos: Position::new(32, 1, 520),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("/// 乘法运算\n".to_string()),
        },
        // Unit 10: get_name function
        TranslationUnit {
            id: "10".to_string(),
            node_type: NodeType::DocString,
            content: "获取计算器名称".to_string(),
            start_pos: Position::new(54, 5, 918),
            end_pos: Position::new(55, 1, 944),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("    /// 获取计算器名称\n".to_string()),
        },
        // Unit 11: String literal
        TranslationUnit {
            id: "11".to_string(),
            node_type: NodeType::StringLiteral,
            content: "\"测试翻译功能\"".to_string(),
            start_pos: Position::new(61, 14, 1034),
            end_pos: Position::new(61, 34, 1054),
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: Some("\"测试翻译功能\"".to_string()),
        },
    ];

    // Set translations
    units[0].set_translated("This is a simple Rust file for testing translation functionality\nContains Chinese comments and doc strings");
    units[1].set_translated("Calculate the sum of two numbers\n# Arguments");
    units[2].set_translated("multiplication");
    units[3].set_translated("Get Calculator Name");
    units[4].set_translated("\"Test translation function\"");

    let result = apply_translations(&content, &units);
    assert!(
        result.is_ok(),
        "apply_translations failed: {:?}",
        result.err()
    );

    let modified = result.unwrap();
    println!("\n=== Modified content ===\n{}", modified);

    // Write output for inspection
    fs::create_dir_all("tests/writer_integration/output").expect("Failed to create output dir");
    fs::write(
        "tests/writer_integration/output/chinese_replacement_result.rs",
        &modified,
    )
    .expect("Failed to write output");

    // Check specific replacements
    assert!(
        modified.contains("// This is a simple Rust file"),
        "First comment not translated"
    );
    assert!(
        modified.contains("/// Calculate the sum of two numbers"),
        "add doc comment not translated"
    );
    assert!(
        modified.contains("/// multiplication"),
        "multiply doc comment not translated correctly"
    );
    assert!(
        modified.contains("/// Get Calculator Name"),
        "get_name doc comment not translated"
    );
    assert!(
        modified.contains("\"Test translation function\""),
        "String literal not translated"
    );
}
