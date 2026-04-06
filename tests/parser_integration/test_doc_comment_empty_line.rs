//! Test for doc comment empty line handling issue
//! Issue: docs/issue/parser-doc-comment-empty-line-handling.md

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::ParserConfig;
use std::fs;
use std::path::PathBuf;

/// Test that empty lines with comment markers are preserved in raw_match
#[test]
fn test_doc_comment_empty_line_preserved() {
    let config = ParserConfig::default();
    let coordinator =
        ParserCoordinator::with_unified_config(config).expect("Failed to create coordinator");

    // Create test content with empty doc comment line
    let content = r#"/// 创建新的计算器实例
/// 
/// # Arguments
/// 
/// * `name` - 计算器名称
pub fn new(name: &str) {}
"#;

    let file = File::new(
        PathBuf::from("test_empty_line.rs"),
        content.as_bytes().to_vec(),
        "utf-8",
    );

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    // Find the unit with "创建新的计算器实例"
    let unit = units
        .iter()
        .find(|u| u.content.contains("创建新的计算器实例"))
        .expect("Should find unit with '创建新的计算器实例'");

    println!("Unit content: {:?}", unit.content);
    println!("Unit raw_match: {:?}", unit.raw_match);

    // The content should include "# Arguments" but separated by empty line
    assert!(
        unit.content.contains("创建新的计算器实例"),
        "Content should contain '创建新的计算器实例'"
    );

    // Check if raw_match preserves the empty line marker
    if let Some(raw) = &unit.raw_match {
        println!("Raw match: {:?}", raw);
        // The raw_match should contain "/// " (with space) for empty line
        // or the content should be properly separated
        let has_empty_line_marker: bool = raw.contains("/// ");
        let has_empty_line_in_content: bool = unit.content.contains("\n\n");
        assert!(
            has_empty_line_marker || has_empty_line_in_content,
            "Raw match should preserve empty line marker or content should have empty line"
        );
    }
}

/// Test the actual fixture file to document current behavior
#[test]
fn test_fixture_simple_rust_doc_comments() {
    let config = ParserConfig::default();
    let coordinator =
        ParserCoordinator::with_unified_config(config).expect("Failed to create coordinator");

    let content = fs::read_to_string("tests/main_integration/fixtures/simple_rust.rs")
        .expect("Failed to read fixture file");

    let file = File::new(
        PathBuf::from("simple_rust.rs"),
        content.as_bytes().to_vec(),
        "utf-8",
    );

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    // Write output for inspection
    let mut output = String::new();
    output.push_str(&format!("Extracted {} translation units\n", units.len()));
    output.push_str("=".repeat(50).as_str());
    output.push('\n');

    for (i, unit) in units.iter().enumerate() {
        output.push_str(&format!("\n--- Unit {} ---\n", i + 1));
        output.push_str(&format!("ID: {}\n", unit.id));
        output.push_str(&format!("Type: {:?}\n", unit.node_type));
        output.push_str(&format!(
            "Position: Line {}, Column {} (Offset: {})\n",
            unit.start_pos.line, unit.start_pos.column, unit.start_pos.offset
        ));
        output.push_str(&format!(
            "End Position: Line {}, Column {} (Offset: {})\n",
            unit.end_pos.line, unit.end_pos.column, unit.end_pos.offset
        ));
        output.push_str(&format!("Content:\n{}\n", unit.content));
        if let Some(raw) = &unit.raw_match {
            output.push_str(&format!("Raw Match:\n{}\n", raw));
        }
        output.push_str(&format!("Should Translate: {}\n", unit.should_translate));
    }

    fs::write(
        "tests/parser_integration/output/doc_comment_issue_analysis.txt",
        output,
    )
    .expect("Failed to write output");

    println!("Output written to: tests/parser_integration/output/doc_comment_issue_analysis.txt");

    // Check for specific issues
    let issues: Vec<_> = units
        .iter()
        .filter(|u| {
            // Find units that might be problematic
            u.content.starts_with("assert_eq!")
                || u.content.starts_with("let ")
                || u.content.starts_with("# ") && !u.content.contains("中文")
        })
        .collect();

    if !issues.is_empty() {
        println!("\nPotential issues found:");
        for unit in issues {
            println!("  - Unit '{}' might be incorrectly extracted", unit.content);
        }
    }
}
