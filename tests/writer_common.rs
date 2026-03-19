//! Common utilities for writer integration tests

use codebase_translate::core::models::{File, NodeType, Position, TranslationUnit};
use std::fs;
use std::path::PathBuf;

const OUTPUT_DIR: &str = "tests/writer_integration/output";

/// Ensure output directory exists
pub fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

/// Write test result to file for comparison
pub fn write_test_result(test_name: &str, original: &str, result: &str, units: &[TranslationUnit]) {
    ensure_output_dir();

    let output_path = PathBuf::from(OUTPUT_DIR).join(format!("{}.txt", test_name));
    let mut output = String::new();

    output.push_str(&format!("Test: {}\n", test_name));
    output.push_str("==================================================\n\n");

    output.push_str("--- Translation Units ---\n");
    for (i, unit) in units.iter().enumerate() {
        output.push_str(&format!("Unit {}:\n", i + 1));
        output.push_str(&format!("  ID: {}\n", unit.id));
        output.push_str(&format!("  Content: {:?}\n", unit.content));
        output.push_str(&format!(
            "  Position: Line {}, Col {}-{}\n",
            unit.start_pos.line, unit.start_pos.column, unit.end_pos.column
        ));
        if let Some(translated) = &unit.translated {
            output.push_str(&format!("  Translated: {:?}\n", translated));
        }
        output.push_str("\n");
    }

    output.push_str("--- Original Content ---\n");
    output.push_str(original);
    output.push_str("\n\n");

    output.push_str("--- Written Content ---\n");
    output.push_str(result);
    output.push_str("\n");

    fs::write(&output_path, output).expect("Failed to write output file");
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
