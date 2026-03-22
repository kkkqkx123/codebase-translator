//! End-to-End Integration Tests
//!
//! Tests that verify the complete workflow from parsing to output.

use std::fs;
use std::path::PathBuf;

use codebase_translate::commands::verify::{
    MatchCollector, OutputFormat, OutputFormatter, StatisticsGenerator,
};
use codebase_translate::core::models::{File, PatternType};
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::ParserConfig;

const FIXTURE_DIR: &str = "tests/verify_integration/fixtures";
const OUTPUT_DIR: &str = "tests/verify_integration/output";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn load_fixture(filename: &str) -> String {
    let path = PathBuf::from(FIXTURE_DIR).join(filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path.display()))
}

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

fn create_test_coordinator() -> ParserCoordinator {
    let config = ParserConfig {
        extract_strings: true,
        ..Default::default()
    };

    ParserCoordinator::with_unified_config(config).expect("Failed to create coordinator")
}

fn write_output(filename: &str, content: &str) {
    ensure_output_dir();
    let output_path = PathBuf::from(OUTPUT_DIR).join(filename);
    fs::write(&output_path, content).expect("Failed to write output file");
    println!("Output written to: {}", output_path.display());
}

#[test]
fn test_end_to_end_simple_rust_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("simple_rust.rs");
    let file = create_test_file(&content, "simple_rust.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    println!("Parsed {} translation units", units.len());
    for (i, unit) in units.iter().enumerate() {
        println!("  Unit {}: {} ({})", i + 1, unit.node_type, unit.content);
    }

    let matches =
        MatchCollector::collect_from_units(PathBuf::from("simple_rust.rs"), units, &content);

    println!("Collected {} verification matches", matches.len());
    for (i, match_item) in matches.iter().enumerate() {
        println!(
            "  Match {}: {} - {}",
            i + 1,
            match_item.pattern_name,
            match_item.extracted_text
        );
    }

    let summary = StatisticsGenerator::generate(&matches, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, true, true)
        .expect("Failed to format output");

    write_output("simple_rust_verification.txt", &output);

    assert!(!matches.is_empty(), "Should collect matches from the file");
    assert!(matches.len() >= 5, "Should collect at least 5 matches");
}

#[test]
fn test_end_to_end_pattern_classification() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("simple_rust.rs");
    let file = create_test_file(&content, "simple_rust.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    let matches =
        MatchCollector::collect_from_units(PathBuf::from("simple_rust.rs"), units, &content);

    let summary = StatisticsGenerator::generate(&matches, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, false, false)
        .expect("Failed to format output");

    write_output("simple_rust_classification.txt", &output);

    let pattern_types: Vec<String> = matches
        .iter()
        .map(|m| format!("{}", m.pattern_type))
        .collect();
    println!("Pattern types found: {:?}", pattern_types);

    assert!(
        pattern_types.contains(&format!("{}", PatternType::Builtin)),
        "Should have builtin patterns"
    );
}

#[test]
fn test_end_to_end_category_distribution() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("simple_rust.rs");
    let file = create_test_file(&content, "simple_rust.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    let matches =
        MatchCollector::collect_from_units(PathBuf::from("simple_rust.rs"), units, &content);

    let summary = StatisticsGenerator::generate(&matches, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Json, false, true)
        .expect("Failed to format output");

    write_output("simple_rust_categories.json", &output);

    println!("Categories: {:?}", summary.by_category);

    assert!(
        !summary.by_category.is_empty(),
        "Should have category distribution"
    );
}

#[test]
fn test_end_to_end_position_tracking() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("simple_rust.rs");
    let file = create_test_file(&content, "simple_rust.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    let matches =
        MatchCollector::collect_from_units(PathBuf::from("simple_rust.rs"), units, &content);

    let output = matches
        .iter()
        .map(|m| {
            format!(
                "Line {}: {} - {}",
                m.position.line, m.pattern_name, m.extracted_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    write_output("simple_rust_positions.txt", &output);

    let all_have_positions = matches.iter().all(|m| m.position.line > 0);
    assert!(
        all_have_positions,
        "All matches should have valid line numbers"
    );
}

#[test]
fn test_end_to_end_complex_rust_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("complex_rust.rs");
    let file = create_test_file(&content, "complex_rust.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    println!("Parsed {} translation units from complex file", units.len());

    let matches =
        MatchCollector::collect_from_units(PathBuf::from("complex_rust.rs"), units, &content);

    println!(
        "Collected {} verification matches from complex file",
        matches.len()
    );

    let summary = StatisticsGenerator::generate(&matches, 1);

    let output = OutputFormatter::format(&matches, &summary, OutputFormat::Table, true, true)
        .expect("Failed to format output");

    write_output("complex_rust_verification.txt", &output);

    assert!(
        !matches.is_empty(),
        "Should collect matches from the complex file"
    );
    assert!(
        matches.len() >= 20,
        "Should collect at least 20 matches from complex file"
    );

    let json_output = OutputFormatter::format(&matches, &summary, OutputFormat::Json, false, true)
        .expect("Failed to format JSON output");

    write_output("complex_rust_verification.json", &json_output);

    let csv_output = OutputFormatter::format(&matches, &summary, OutputFormat::Csv, false, false)
        .expect("Failed to format CSV output");

    write_output("complex_rust_verification.csv", &csv_output);
}
