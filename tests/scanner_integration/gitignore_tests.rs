//! Gitignore Tests
//!
//! Tests for .gitignore pattern matching functionality.

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
fn test_respect_gitignore() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: true,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            !entry.relative_path.ends_with("ignored.log"),
            "Files matching .gitignore patterns should be excluded"
        );
    }

    write_scan_result("respect_gitignore", &result);
}

#[test]
fn test_custom_gitignore_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["*.tmp".to_string(), "temp_*".to_string()],
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
        let filename = entry
            .relative_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert!(
            ext != "tmp" && !filename.starts_with("temp_"),
            "Files matching custom gitignore patterns should be excluded"
        );
    }

    write_scan_result("custom_gitignore_patterns", &result);
}

#[test]
fn test_custom_gitignore_file() {
    let scanner = FSScanner::new();

    let custom_gitignore_path = PathBuf::from(FIXTURES_DIR).join("custom.gitignore");

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: true,
        gitignore_patterns: vec![],
        gitignore_path: Some(custom_gitignore_path),
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        let filename = entry
            .relative_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert!(
            !filename.ends_with(".custom"),
            "Files matching custom .gitignore should be excluded"
        );
    }

    write_scan_result("custom_gitignore_file", &result);
}

#[test]
fn test_gitignore_with_include_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: true,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            !entry.relative_path.ends_with("ignored.log"),
            "Files matching .gitignore should be excluded even with include patterns"
        );
    }

    write_scan_result("gitignore_with_include_patterns", &result);
}

#[test]
fn test_gitignore_with_exclude_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["*.log".to_string()],
        follow_symlinks: false,
        respect_gitignore: true,
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
            ext != "log",
            "Files matching exclude patterns should be excluded"
        );
    }

    write_scan_result("gitignore_with_exclude_patterns", &result);
}

#[test]
fn test_gitignore_negation() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: true,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    let has_important = result
        .iter()
        .any(|entry| entry.relative_path.ends_with("important.log"));

    assert!(
        has_important,
        "Files negated in .gitignore should be included"
    );

    write_scan_result("gitignore_negation", &result);
}

#[test]
fn test_gitignore_directory_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: true,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            !entry.relative_path.starts_with("ignored_dir"),
            "Files in ignored directories should be excluded"
        );
    }

    write_scan_result("gitignore_directory_patterns", &result);
}

#[test]
fn test_gitignore_globstar_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: true,
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
            ext != "cache",
            "Files matching **/*.cache pattern should be excluded"
        );
    }

    write_scan_result("gitignore_globstar_patterns", &result);
}
