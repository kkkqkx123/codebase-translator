//! Test parser with Chinese content

use std::fs;
use std::path::PathBuf;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::ParserConfig;

#[test]
fn test_parse_chinese_rust_file() {
    let config = ParserConfig {
        extract_strings: true,
        ..Default::default()
    };

    let coordinator =
        ParserCoordinator::with_unified_config(config).expect("Failed to create coordinator");

    // Load the Chinese Rust file from main_integration fixtures
    let content = fs::read_to_string("tests/main_integration/fixtures/simple_rust.rs")
        .expect("Failed to read fixture file");

    let file = File::new(
        PathBuf::from("simple_rust.rs"),
        content.as_bytes().to_vec(),
        "utf-8",
    );

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    // Write output for inspection
    fs::create_dir_all("tests/parser_integration/output").expect("Failed to create output dir");
    let output_path = PathBuf::from("tests/parser_integration/output/chinese_rust_parse.txt");

    let mut output = String::new();
    output.push_str(&format!("Extracted {} translation units\n", units.len()));
    output.push_str("==================================================\n\n");

    for (i, unit) in units.iter().enumerate() {
        output.push_str(&format!("--- Unit {} ---\n", i + 1));
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
        output.push('\n');
    }

    fs::write(&output_path, output).expect("Failed to write output file");
    println!("Output written to: {}", output_path.display());

    // Print summary
    println!("\n=== Summary ===");
    println!("Total units: {}", units.len());
    for (i, unit) in units.iter().enumerate() {
        println!(
            "Unit {}: {:?} - '{}'",
            i + 1,
            unit.node_type,
            unit.content.chars().take(30).collect::<String>()
        );
    }
}
