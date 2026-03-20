//! Debug test to check parser output

use std::fs;
use std::path::PathBuf;

const FIXTURES_DIR: &str = "tests/main_integration/fixtures";

#[test]
fn debug_parse_rust_file_multiply() {
    let fixture_path = PathBuf::from(FIXTURES_DIR).join("simple_rust.rs");
    let content = fs::read_to_string(&fixture_path).expect("Failed to read fixture");

    // Print lines around the multiply function
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate().skip(28).take(10) {
        println!("Line {}: {:?}", i + 1, line);
    }
}
