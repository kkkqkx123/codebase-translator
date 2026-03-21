//! Tests for Python docstring format preservation
//!
//! This test file reproduces and verifies the fix for the Python docstring
//! indentation and formatting issues.

use codebase_translate::core::models::{File, NodeType, Position, TranslationUnit};
use codebase_translate::writer::{FileWriter, WriterConfig};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const OUTPUT_DIR: &str = "tests/writer_integration/output";

fn ensure_output_dir() {
    std::fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn write_output(filename: &str, content: &str) {
    ensure_output_dir();
    let output_path = PathBuf::from(OUTPUT_DIR).join(format!("{}.txt", filename));
    std::fs::write(&output_path, content).expect("Failed to write output file");
    println!("Output written to: {}", output_path.display());
}

async fn create_test_file(dir: &Path, name: &str, content: &str) -> File {
    let file_path = dir.join(name);
    tokio::fs::write(&file_path, content)
        .await
        .expect("Failed to create test file");

    File::new(file_path, content.as_bytes().to_vec(), "UTF-8")
}

async fn read_file_content(path: &PathBuf) -> String {
    tokio::fs::read_to_string(path)
        .await
        .expect("Failed to read file")
}

/// Create a multiline translation unit with proper offsets
fn create_multiline_unit(
    id: &str,
    content: &str,
    raw_match: &str,
    start_line: usize,
    end_line: usize,
    start_offset: usize,
    end_offset: usize,
) -> TranslationUnit {
    TranslationUnit {
        id: id.to_string(),
        node_type: NodeType::DocString,
        content: content.to_string(),
        start_pos: Position::new(start_line, 1, start_offset),
        end_pos: Position::new(end_line, 1, end_offset),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: Some(codebase_translate::core::models::PatternType::Builtin),
        pattern_name: Some("python".to_string()),
        raw_match: Some(raw_match.to_string()),
    }
}

#[tokio::test]
async fn test_python_docstring_format_preservation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Original Python content with docstring
    let content = r#"def add(a, b):
    """
    计算两个数的和
    
    Args:
        a: 第一个数字
        b: 第二个数字
    
    Returns:
        两个数的和
    """
    return a + b
"#;

    let file = create_test_file(&temp_path, "test_docstring.py", content).await;

    // The raw_match is the complete docstring as it appears in the source
    // including triple quotes and indentation
    let raw_match = r#"    """
    计算两个数的和
    
    Args:
        a: 第一个数字
        b: 第二个数字
    
    Returns:
        两个数的和
    """"#;

    // The extracted_content is what the parser extracts after cleaning
    // (removing triple quotes and common indentation)
    let extracted_content = r#"计算两个数的和

Args:
    a: 第一个数字
    b: 第二个数字

Returns:
    两个数的和"#;

    let translated_content = r#"Compute the sum of two numbers

Args:
    a: first number
    b: second number

Returns:
    The sum of the two numbers"#;

    // Calculate the correct byte offsets
    // "def add(a, b):\n" = 15 bytes
    let start_offset = 15;
    // The docstring length in bytes
    let end_offset = start_offset + raw_match.len();

    // Create the translation unit
    let mut unit = create_multiline_unit(
        "1",
        extracted_content,
        raw_match,
        2,   // start line
        11,  // end line
        start_offset,
        end_offset,
    );
    unit.set_translated(translated_content);

    let units = vec![unit];

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok(), "Write should succeed: {:?}", result.err());

    let written_content = read_file_content(&file.path).await;

    // Write output for debugging
    let output = format!(
        "Test: Python Docstring Format Preservation\n\
         ===========================================\n\n\
         Original content:\n{}\n\n\
         Written content:\n{}\n\n\
         Raw match used:\n{}\n\n\
         Extracted content:\n{}\n\n\
         Translated content:\n{}\n",
        content, written_content, raw_match, extracted_content, translated_content
    );
    write_output("python_docstring_format_test", &output);

    // Verify the docstring format is preserved
    assert!(
        written_content.contains(r#"    """"#),
        "Opening triple quotes should be preserved with proper indentation"
    );
    assert!(
        written_content.contains(r#"    """"#),
        "Closing triple quotes should be preserved with proper indentation"
    );
    assert!(
        written_content.contains("Compute the sum of two numbers"),
        "Translated content should be present"
    );
    assert!(
        written_content.contains("    Args:"),
        "Args section should have proper indentation"
    );
    assert!(
        written_content.contains("        a: first number"),
        "Parameter descriptions should have proper indentation (8 spaces)"
    );
    assert!(
        written_content.contains("    Returns:"),
        "Returns section should have proper indentation"
    );

    // Verify the function structure is intact
    assert!(
        written_content.contains("def add(a, b):"),
        "Function definition should be preserved"
    );
    assert!(
        written_content.contains("    return a + b"),
        "Function body should be preserved"
    );
}

#[tokio::test]
async fn test_python_single_line_docstring() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"class Calculator:
    """简单的计算器类"""
    
    def __init__(self, name):
        pass
"#;

    let file = create_test_file(&temp_path, "test_single_line_docstring.py", content).await;

    // For single-line docstrings, we use a single-line translation unit
    // The raw_match should be the complete docstring with triple quotes
    let raw_match = r#""""简单的计算器类""")"#;
    let extracted_content = "简单的计算器类";
    let translated_content = "Simple Calculator Class";

    // Single-line docstrings are handled differently - they're not multiline
    // So we use a regular single-line unit
    // The content is the full line, and raw_match is the docstring portion
    let mut unit = TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::DocString,
        content: extracted_content.to_string(),
        start_pos: Position::new(2, 5, 20),  // column 5 (0-indexed: 4) is where """ starts
        end_pos: Position::new(2, 21, 36),   // column 21 is after """
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: Some(codebase_translate::core::models::PatternType::Builtin),
        pattern_name: Some("python".to_string()),
        raw_match: Some(raw_match.to_string()),
    };
    unit.set_translated(translated_content);

    let units = vec![unit];

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    let output = format!(
        "Test: Python Single Line Docstring\n\
         ===================================\n\n\
         Original content:\n{}\n\n\
         Written content:\n{}\n",
        content, written_content
    );
    write_output("python_single_line_docstring_test", &output);

    // For single-line docstrings, the LineApplier should handle it
    // The raw_match contains """简单的计算器类""" and we replace "简单的计算器类" with "Simple Calculator Class"
    assert!(
        written_content.contains(r#"""Simple Calculator Class""")"#),
        "Single line docstring should preserve triple quotes and indentation"
    );
}

#[tokio::test]
async fn test_python_multiline_comment() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"# 这是一个简单的Python文件，用于测试翻译功能
# 包含中文注释和文档字符串

def hello():
    pass
"#;

    let file = create_test_file(&temp_path, "test_python_comment.py", content).await;

    // First comment line
    let mut unit1 = TranslationUnit {
        id: "1".to_string(),
        node_type: NodeType::Comment,
        content: "这是一个简单的Python文件，用于测试翻译功能".to_string(),
        start_pos: Position::new(1, 1, 0),
        end_pos: Position::new(1, 30, 30),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("# 这是一个简单的Python文件，用于测试翻译功能".to_string()),
    };
    unit1.set_translated("This is a simple Python file for testing translation");

    // Second comment line
    let mut unit2 = TranslationUnit {
        id: "2".to_string(),
        node_type: NodeType::Comment,
        content: "包含中文注释和文档字符串".to_string(),
        start_pos: Position::new(2, 1, 31),
        end_pos: Position::new(2, 20, 51),
        language: None,
        should_translate: true,
        translated: None,
        pattern_type: None,
        pattern_name: None,
        raw_match: Some("# 包含中文注释和文档字符串".to_string()),
    };
    unit2.set_translated("Contains Chinese comments and docstrings");

    let units = vec![unit1, unit2];

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    let output = format!(
        "Test: Python Multiline Comment\n\
         ===============================\n\n\
         Original content:\n{}\n\n\
         Written content:\n{}\n",
        content, written_content
    );
    write_output("python_multiline_comment_test", &output);

    assert!(
        written_content.contains("# This is a simple Python file for testing translation"),
        "First comment should be translated with # prefix"
    );
    assert!(
        written_content.contains("# Contains Chinese comments and docstrings"),
        "Second comment should be translated with # prefix"
    );
}
