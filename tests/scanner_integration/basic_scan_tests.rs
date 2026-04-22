//! Basic Scanner Integration Tests
//!
//! Tests for basic file system scanning functionality.

use std::fs;
use std::path::PathBuf;

use codebase_translate::core::models::FileEntry;
use codebase_translate::scanner::{FSScanner, ScanOptions, Scanner};

const FIXTURES_DIR: &str = "tests/scanner_integration/fixtures";
const OUTPUT_DIR: &str = "tests/scanner_integration/output";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn write_scan_result(filename: &str, entries: &[FileEntry]) {
    ensure_output_dir();

    let output_path = PathBuf::from(OUTPUT_DIR).join(format!("{}.txt", filename));
    let mut output = String::new();

    output.push_str(&format!("Found {} files\n", entries.len()));
    output.push_str("==================================================\n\n");

    for (i, entry) in entries.iter().enumerate() {
        output.push_str(&format!("--- File {} ---\n", i + 1));
        output.push_str(&format!("Path: {}\n", entry.path.display()));
        output.push_str(&format!("Relative: {}\n", entry.relative_path.display()));
        output.push_str(&format!("Size: {} bytes\n", entry.size));
        output.push_str(&format!("Modified: {:?}\n", entry.modified));
        output.push('\n');
    }

    fs::write(&output_path, output).expect("Failed to write scan result");
}

#[test]
fn test_basic_scan() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts);
    assert!(result.is_ok(), "Scan should succeed");

    let entries = result.unwrap();
    assert!(!entries.is_empty(), "Should find some .rs files");

    write_scan_result("basic_scan", &entries);
}

#[test]
fn test_scan_multiple_extensions() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*.rs".to_string(), "*.py".to_string(), "*.js".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts);
    assert!(result.is_ok(), "Scan should succeed");

    let entries = result.unwrap();
    assert!(
        !entries.is_empty(),
        "Should find files with multiple extensions"
    );

    write_scan_result("scan_multiple_extensions", &entries);
}

#[test]
fn test_scan_all_files() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts);
    assert!(result.is_ok(), "Scan should succeed");

    let entries = result.unwrap();
    assert!(!entries.is_empty(), "Should find all files");

    write_scan_result("scan_all_files", &entries);
}

#[test]
fn test_scan_nonexistent_directory() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: "nonexistent_directory".to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts);
    assert!(
        result.is_err(),
        "Scan should fail for nonexistent directory"
    );
}

#[test]
fn test_scan_single_file() {
    let scanner = FSScanner::new();

    let file_path = PathBuf::from(FIXTURES_DIR).join("simple_rust.rs");
    assert!(file_path.exists(), "Test file should exist");

    let opts = ScanOptions {
        root_path: file_path.to_string_lossy().to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts);
    assert!(
        result.is_ok(),
        "Scan should succeed when path is a single file"
    );

    let entries = result.expect("Should get scan results");
    assert_eq!(entries.len(), 1, "Should find exactly one file");
    assert_eq!(
        entries[0].relative_path.to_string_lossy(),
        "simple_rust.rs",
        "Should find the correct file"
    );
}

#[test]
fn test_scan_empty_directory() {
    let scanner = FSScanner::new();

    let empty_dir = PathBuf::from(FIXTURES_DIR).join("empty_dir");
    fs::create_dir_all(&empty_dir).expect("Failed to create empty test directory");

    let opts = ScanOptions {
        root_path: empty_dir.to_string_lossy().to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts);
    assert!(result.is_ok(), "Scan should succeed for empty directory");

    let entries = result.unwrap();
    assert!(
        entries.is_empty(),
        "Should find no files in empty directory"
    );

    write_scan_result("scan_empty_directory", &entries);
}
