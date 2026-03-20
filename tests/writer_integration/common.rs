//! Common utilities for writer integration tests

use codebase_translate::core::models::{File, NodeType, Position, TranslationUnit};
use std::fs;
use std::path::PathBuf;

const OUTPUT_DIR: &str = "tests/writer_integration/output";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

/// Load fixture file content
pub fn load_fixture(filename: &str) -> String {
    let path = PathBuf::from("tests/writer_integration/fixtures").join(filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path.display()))
}

/// Write test result to output file
pub fn write_output(filename: &str, content: &str) {
    ensure_output_dir();
    let output_path = PathBuf::from(OUTPUT_DIR).join(format!("{}.txt", filename));
    fs::write(&output_path, content).expect("Failed to write output file");
    println!("Output written to: {}", output_path.display());
}

/// Create a test file with given content
pub async fn create_test_file(dir: &PathBuf, name: &str, content: &str) -> File {
    let file_path = dir.join(name);
    tokio::fs::write(&file_path, content)
        .await
        .expect("Failed to create test file");

    File::new(file_path, content.as_bytes().to_vec(), "UTF-8")
}

/// Create a simple translation unit for testing
pub fn create_translation_unit(
    id: &str,
    content: &str,
    line: usize,
    start_col: usize,
    end_col: usize,
) -> TranslationUnit {
    TranslationUnit {
        id: id.to_string(),
        node_type: NodeType::Comment,
        content: content.to_string(),
        start_pos: Position::new(line, start_col, 0),
        end_pos: Position::new(line, end_col, 0),
        language: None,
        should_translate: true,
        translated: None,
        format_info: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: None,
    }
}

/// Create a translation unit with format info
pub fn create_translation_unit_with_format(
    id: &str,
    content: &str,
    line: usize,
    start_col: usize,
    end_col: usize,
    format_info: codebase_translate::core::models::FormatInfo,
) -> TranslationUnit {
    TranslationUnit {
        id: id.to_string(),
        node_type: NodeType::Comment,
        content: content.to_string(),
        start_pos: Position::new(line, start_col, 0),
        end_pos: Position::new(line, end_col, 0),
        language: None,
        should_translate: true,
        translated: None,
        format_info: Some(format_info),
        pattern_type: None,
        pattern_name: None,
        raw_match: None,
    }
}

/// Read file content as string
pub async fn read_file_content(path: &PathBuf) -> String {
    tokio::fs::read_to_string(path)
        .await
        .expect("Failed to read file")
}

/// Clean up test files
pub async fn cleanup_test_files(files: Vec<PathBuf>) {
    for file in files {
        let _ = tokio::fs::remove_file(&file).await;
    }
}

/// Create a temporary directory for testing
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}
