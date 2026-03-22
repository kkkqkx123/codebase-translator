//! Go parser integration tests for writer

use codebase_translate::parser::filtering::{ContentFilter, FilterConfig};
use codebase_translate::parser::core::strategies::{
    ConfigBasedStrategy, ExtractionStrategyImpl,
};
use codebase_translate::parser::abstraction::strategy::ExtractionConfig;
use codebase_translate::parser::{ParserConfig, TreeSitterParserFactory};
use codebase_translate::parser::Parser;
use codebase_translate::writer::{FileWriter, WriterConfig};

use super::common::*;

fn create_test_strategy() -> std::sync::Arc<ExtractionStrategyImpl> {
    std::sync::Arc::new(ExtractionStrategyImpl::ConfigBased(
        ConfigBasedStrategy::new(ExtractionConfig::default()),
    ))
}

fn create_test_filter() -> std::sync::Arc<ContentFilter> {
    std::sync::Arc::new(ContentFilter::new(FilterConfig::default()).unwrap())
}

#[tokio::test]
async fn test_go_parser_extracts_comments_with_raw_match() {
    let config = ParserConfig::default();
    let strategy = create_test_strategy();
    let filter = create_test_filter();
    let parser = TreeSitterParserFactory::create_go_parser(config, strategy, filter)
        .expect("Failed to create Go parser");

    let content = r#"// Test file with simple comments
// This is a line comment
package main

// This is a single-line comment
const value = 42

/*
This is a multi-line comment
with multiple lines of text
*/

func test() int {
    // Another comment inside function
    return value
}

// greet returns a greeting message
// name is the person to greet
func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}
"#;

    let file = crate::writer_integration::common::create_test_file(
        &std::env::temp_dir(),
        "test_go_format.go",
        content,
    )
    .await;

    let units = parser.parse(&file).expect("Parsing should succeed");

    println!("Extracted {} units:", units.len());
    for (i, unit) in units.iter().enumerate() {
        println!("Unit {}: {:?}", i, unit);
        println!("  Content: {:?}", unit.content);
        println!(
            "  Position: line {}, col {} - line {}, col {}",
            unit.start_pos.line, unit.start_pos.column, unit.end_pos.line, unit.end_pos.column
        );
        if let Some(raw) = &unit.raw_match {
            println!("  Raw match: {:?}", raw);
        } else {
            println!("  Raw match: None");
        }
    }

    // Check that units have raw_match
    assert!(!units.is_empty(), "Should have extracted units");

    for unit in &units {
        assert!(
            unit.raw_match.is_some(),
            "Unit '{}' should have raw_match",
            unit.content
        );

        // Check that start_pos.column is valid (1-indexed, should be >= 1)
        assert!(
            unit.start_pos.column >= 1,
            "Unit '{}' should have valid start_pos.column (>= 1), got {}",
            unit.content,
            unit.start_pos.column
        );
    }
}

#[tokio::test]
async fn test_go_end_to_end_write() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    // Use non-consecutive comments to avoid merging
    let content = r#"// Test file with simple comments
package main

// This is a single-line comment
const value = 42

func test() int {
    // Another comment inside function
    return value
}

// greet returns a greeting message
func greet(name string) string {
    // name is the person to greet
    return fmt.Sprintf("Hello, %s!", name)
}
"#;

    // Step 1: Parse the file
    let config = ParserConfig::default();
    let strategy = create_test_strategy();
    let filter = create_test_filter();
    let parser = TreeSitterParserFactory::create_go_parser(config, strategy, filter)
        .expect("Failed to create Go parser");

    let file = create_test_file(&temp_path, "test_go_e2e.go", content).await;
    let mut units = parser.parse(&file).expect("Parsing should succeed");

    println!("Extracted {} units:", units.len());
    for (i, unit) in units.iter().enumerate() {
        println!("Unit {}: content={:?}", i, unit.content);
        if let Some(raw) = &unit.raw_match {
            println!("  Raw match: {:?}", raw);
        }
    }

    // Step 2: Set translations
    // Note: content field is now cleaned (without comment markers)
    for unit in &mut units {
        match unit.content.as_str() {
            "Test file with simple comments" => unit.set_translated("测试文件，包含简单注释"),
            "This is a single-line comment" => unit.set_translated("这是一个单行注释"),
            "Another comment inside function" => unit.set_translated("函数内部的另一个注释"),
            "greet returns a greeting message" => unit.set_translated("greet 返回问候信息"),
            "name is the person to greet" => unit.set_translated("name 是要问候的人"),
            _ => {}
        }
    }

    // Step 3: Write back
    let writer_config = WriterConfig::default();
    let writer = FileWriter::new(writer_config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok(), "Write should succeed");

    // Step 4: Verify output
    let written_content = read_file_content(&file.path).await;
    println!("Original content:\n{}", content);
    println!("Written content:\n{}", written_content);

    // Check that // prefix is preserved
    assert!(
        written_content.contains("// 测试文件，包含简单注释"),
        "Line 1 comment should have // prefix"
    );
    assert!(
        written_content.contains("// 这是一个单行注释"),
        "Line 5 comment should have // prefix"
    );
    assert!(
        written_content.contains("// 函数内部的另一个注释"),
        "Line 10 comment should have // prefix"
    );
    assert!(
        written_content.contains("// greet 返回问候信息"),
        "Line 13 comment should have // prefix"
    );
    assert!(
        written_content.contains("// name 是要问候的人"),
        "Line 15 comment should have // prefix"
    );

    // Write output for inspection
    write_output("test_go_end_to_end_write", &written_content);
}
