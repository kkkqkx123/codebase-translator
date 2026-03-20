//! Tests for Chinese content handling in writer

use codebase_translate::core::models::{NodeType, Position, TranslationUnit};
use codebase_translate::writer::apply_translations;

#[test]
fn test_chinese_doc_comment_replacement() {
    // Simulate the exact scenario from E2E test
    // Original line: "    /// 获取计算器名称"
    // Translated: "    /// Get Calculator Name"
    
    let content = "    /// 获取计算器名称\n    pub fn get_name(&self) -> &str {\n        &self.name\n    }";
    
    // Create translation unit simulating parser output
    let mut units = vec![TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::DocString,
        content: "获取计算器名称".to_string(),
        // Tree-sitter returns byte positions (1-based column)
        // "    /// 获取计算器名称" breakdown:
        // - 4 spaces: bytes 0-3
        // - ///: bytes 4-6
        // - space: byte 7
        // - 获取计算器名称: bytes 8-31 (8 chars * 3 bytes each = 24 bytes)
        // Total: 32 bytes
        // Content starts at column 9 (1-based) = byte 8 (0-based)
        // Content ends at column 33 (1-based) = byte 32 (0-based)
        start_pos: Position::new(1, 9, 8),
        end_pos: Position::new(1, 33, 32),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("    /// 获取计算器名称".to_string()),
    }];
    
    units[0].set_translated("Get Calculator Name");
    
    let result = apply_translations(content, &units).unwrap();
    
    println!("Original content:\n{}", content);
    println!("\nResult:\n{}", result);
    
    // Check that translation is correct
    assert!(result.contains("/// Get Calculator Name"), 
            "Translation not found. Result: {}", result);
    
    // Check that next line is not merged
    assert!(result.contains("\n    pub fn get_name(&self) -> &str {"),
            "Next line was merged. Result: {}", result);
}

#[test]
fn test_chinese_multiline_doc_replacement() {
    // Test case for: "/// multiplicationpub fn multiply"
    let content = "/// 乘法运算\npub fn multiply(a: i32, b: i32) -> i32 {\n    a * b\n}";
    
    let mut units = vec![TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::DocString,
        content: "乘法运算".to_string(),
        // "/// 乘法运算":
        // - ///: bytes 0-2
        // - space: byte 3
        // - 乘法运算: bytes 4-15 (4 chars * 3 bytes = 12 bytes)
        // Total: 16 bytes
        start_pos: Position::new(1, 5, 4),  // Content starts after "/// "
        end_pos: Position::new(1, 17, 16),   // End of line
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("/// 乘法运算".to_string()),
    }];
    
    units[0].set_translated("multiplication");
    
    let result = apply_translations(content, &units).unwrap();
    
    println!("Original content:\n{}", content);
    println!("\nResult:\n{}", result);
    
    // Check that translation is correct and not merged with next line
    assert!(result.contains("/// multiplication\n"),
            "Translation incorrect or merged. Result: {}", result);
    assert!(result.contains("pub fn multiply(a: i32, b: i32) -> i32 {"),
            "Function definition missing. Result: {}", result);
}
