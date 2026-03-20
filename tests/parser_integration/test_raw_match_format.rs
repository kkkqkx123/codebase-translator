//! Test to check raw_match format from parser

use std::fs;
use std::path::PathBuf;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::tree_sitter::ParserConfig;

#[test]
fn test_raw_match_format() {
    let mut config = ParserConfig::default();
    config.extract_strings = true;

    let coordinator =
        ParserCoordinator::with_unified_config(config).expect("Failed to create coordinator");

    let content = fs::read_to_string("tests/main_integration/fixtures/simple_rust.rs")
        .expect("Failed to read fixture file");

    println!("Original file content:\n{}", content);
    println!("\n=== File content bytes around line 4-7 ===");
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate().take(10).skip(3) {
        println!("Line {}: {:?}", i + 1, line);
    }

    let file = File::new(
        PathBuf::from("simple_rust.rs"),
        content.as_bytes().to_vec(),
        "utf-8",
    );
    let units = coordinator.parse_file(&file).expect("Parsing failed");

    println!("\n=== Translation Units ===\n");
    for (i, unit) in units.iter().enumerate() {
        println!("Unit {}: {:?}", i + 1, unit.node_type);
        println!("  Content: {:?}", unit.content);
        if let Some(raw) = &unit.raw_match {
            println!("  Raw match: {:?}", raw);
            // Check if raw_match ends with newline
            println!("  Raw match ends with '\\n': {}", raw.ends_with('\n'));
            // Show byte representation
            println!("  Raw match bytes: {:?}", raw.as_bytes());
        }
        println!(
            "  Position: Line {}, Col {} (Offset {})",
            unit.start_pos.line, unit.start_pos.column, unit.start_pos.offset
        );
        println!(
            "  End Position: Line {}, Col {} (Offset {})",
            unit.end_pos.line, unit.end_pos.column, unit.end_pos.offset
        );
        println!();
    }
}
