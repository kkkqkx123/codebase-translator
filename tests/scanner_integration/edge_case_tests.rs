//! Edge Case Tests
//!
//! Tests for edge cases and special scenarios.

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

    output.push_str(&format!(
        "Found {} files\n",
        entries.len()
    ));
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
fn test_empty_include_patterns() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(!result.is_empty(), "Empty include patterns should match all files");

    write_scan_result("empty_include_patterns", &result);
}

#[test]
fn test_empty_exclude_patterns() {
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
    assert!(!result.is_empty(), "Empty exclude patterns should not exclude anything");

    write_scan_result("empty_exclude_patterns", &result);
}

#[test]
fn test_no_matching_include_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*.nonexistent".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(result.is_empty(), "No files should match non-existent patterns");

    write_scan_result("no_matching_include_patterns", &result);
}

#[test]
fn test_all_files_excluded() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["*".to_string()],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(result.is_empty(), "All files should be excluded");

    write_scan_result("all_files_excluded", &result);
}

#[test]
fn test_special_characters_in_filename() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    let has_special_chars = result
        .iter()
        .any(|entry| {
            let filename = entry.relative_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            filename.contains(' ') || filename.contains('-') || filename.contains('_')
        });

    assert!(has_special_chars, "Should find files with special characters");

    write_scan_result("special_characters_in_filename", &result);
}

#[test]
fn test_hidden_files() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    let has_hidden = result
        .iter()
        .any(|entry| {
            let filename = entry.relative_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            filename.starts_with('.')
        });

    assert!(has_hidden, "Should find hidden files");

    write_scan_result("hidden_files", &result);
}

#[test]
fn test_empty_filename() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        let filename = entry.relative_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        assert!(!filename.is_empty(), "No file should have empty filename");
    }

    write_scan_result("empty_filename", &result);
}

#[test]
fn test_case_sensitive_patterns() {
    let scanner = FSScanner::new();

    let opts = ScanOptions {
        root_path: FIXTURES_DIR.to_string(),
        include_patterns: vec!["*.RS".to_string()],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        let ext = entry.relative_path.extension().unwrap_or_default().to_str().unwrap_or("");
        assert_eq!(ext, "RS", "Extension should match case-sensitively");
    }

    write_scan_result("case_sensitive_patterns", &result);
}

#[test]
fn test_duplicate_files() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    let mut paths: Vec<_> = result.iter().map(|e| e.relative_path.clone()).collect();
    paths.sort();
    paths.dedup();

    assert_eq!(
        paths.len(),
        result.len(),
        "No duplicate files should be returned"
    );

    write_scan_result("duplicate_files", &result);
}

#[test]
fn test_absolute_path_handling() {
    let scanner = FSScanner::new();

    let absolute_path = fs::canonicalize(FIXTURES_DIR).expect("Failed to get absolute path");

    let opts = ScanOptions {
        root_path: absolute_path.to_string_lossy().to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        follow_symlinks: false,
        respect_gitignore: false,
        gitignore_patterns: vec![],
        gitignore_path: None,
    };

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(!result.is_empty(), "Should handle absolute paths correctly");

    write_scan_result("absolute_path_handling", &result);
}

#[test]
fn test_relative_path_handling() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");
    assert!(!result.is_empty(), "Should handle relative paths correctly");

    for entry in &result {
        let relative_str = entry.relative_path.to_string_lossy().to_string();
        assert!(
            !relative_str.contains(':') || relative_str.starts_with("\\\\"),
            "Relative paths should not contain drive letters"
        );
    }

    write_scan_result("relative_path_handling", &result);
}

#[test]
fn test_symlink_handling() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            !entry.path.is_symlink(),
            "Symlinks should not be included when follow_symlinks is false"
        );
    }

    write_scan_result("symlink_handling", &result);
}

#[test]
fn test_file_size_info() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(entry.size > 0, "File size should be positive");
    }

    write_scan_result("file_size_info", &result);
}

#[test]
fn test_modified_time_info() {
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

    let result = scanner.scan(opts).expect("Scan should succeed");

    for entry in &result {
        assert!(
            entry.modified.elapsed().unwrap().as_secs() > 0,
            "Modified time should be available"
        );
    }

    write_scan_result("modified_time_info", &result);
}
