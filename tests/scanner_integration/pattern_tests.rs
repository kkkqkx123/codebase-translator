//! Pattern Matching Tests
//!
//! Tests for include and exclude pattern matching.

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
        output.push_str(&format!("Path: {}\n", entry.relative_path.display()));
        output.push_str(&format!("Size: {} bytes\n", entry.size));
        output.push('\n');
    }

    fs::write(&output_path, output).expect("Failed to write scan result");
}

#[test]
fn test_include_single_extension() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(!result.is_empty(), "Should find .rs files");

    for entry in &result {
        assert!(
            entry
                .relative_path
                .extension()
                .is_some_and(|ext| ext == "rs"),
            "All files should have .rs extension"
        );
    }

    write_scan_result("include_single_extension", &result);
}

#[test]
fn test_include_multiple_extensions() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*.rs".to_string(), "*.py".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(!result.is_empty(), "Should find .rs or .py files");

    for entry in &result {
        let ext = entry
            .relative_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("");
        assert!(
            ext == "rs" || ext == "py",
            "All files should have .rs or .py extension"
        );
    }

    write_scan_result("include_multiple_extensions", &result);
}

#[test]
fn test_exclude_extension() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["*.log".to_string()],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        let ext = entry
            .relative_path
            .extension()
            .map_or(true, |ext| ext != "log");
        assert!(ext, "No files should have .log extension");
    }

    write_scan_result("exclude_extension", &result);
}

#[test]
fn test_include_and_exclude() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["*.log".to_string(), "*.tmp".to_string()],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        let ext = entry
            .relative_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("");
        assert!(
            ext != "log" && ext != "tmp",
            "No files should have .log or .tmp extension"
        );
    }

    write_scan_result("include_and_exclude", &result);
}

#[test]
fn test_wildcard_pattern() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(
        !result.is_empty(),
        "Should find all files with wildcard pattern"
    );

    write_scan_result("wildcard_pattern", &result);
}

#[test]
fn test_recursive_pattern() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["**/*.rs".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(!result.is_empty(), "Should find .rs files recursively");

    for entry in &result {
        assert!(
            entry
                .relative_path
                .extension()
                .is_some_and(|ext| ext == "rs"),
            "All files should have .rs extension"
        );
    }

    write_scan_result("recursive_pattern", &result);
}

#[test]
fn test_directory_pattern() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["nested_dir/*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            entry.relative_path.starts_with("nested_dir"),
            "All files should be in nested_dir"
        );
    }

    write_scan_result("directory_pattern", &result);
}

#[test]
fn test_exclude_directory() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["nested_dir".to_string()],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            !entry.relative_path.starts_with("nested_dir"),
            "No files should be in nested_dir"
        );
    }

    write_scan_result("exclude_directory", &result);
}
