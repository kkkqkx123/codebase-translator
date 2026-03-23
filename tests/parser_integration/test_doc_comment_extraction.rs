//! Test for doc comment extraction across languages

use std::path::PathBuf;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::ParserConfig;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

fn create_test_coordinator() -> ParserCoordinator {
    ParserCoordinator::with_defaults(ParserConfig::default()).expect("Failed to create coordinator")
}

#[test]
fn test_rust_doc_comment_extraction() {
    let coordinator = create_test_coordinator();

    let content = r#"
/// This is a doc comment with Chinese text: 你好世界
fn main() {
    // This is a regular comment with Chinese text: 你好
    let x = 5;
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    println!("Extracted {} units:", units.len());
    for unit in &units {
        println!("  Type: {:?}, Content: {:?}", unit.node_type, unit.content);
    }

    // Should extract both doc comment and regular comment
    assert!(
        units.iter().any(|u| u.content.contains("你好世界")),
        "Should extract doc comment with Chinese text"
    );
    assert!(
        units.iter().any(|u| u.content.contains("你好")),
        "Should extract regular comment with Chinese text"
    );
}

#[test]
fn test_go_doc_comment_extraction() {
    let coordinator = create_test_coordinator();

    let content = r#"
package main

// This is a doc comment with Chinese text: 你好世界
func main() {
    // This is a regular comment with Chinese text: 你好
    x := 5
}
"#;

    let file = create_test_file(content, "test.go");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    println!("Go - Extracted {} units:", units.len());
    for unit in &units {
        println!("  Type: {:?}, Content: {:?}", unit.node_type, unit.content);
    }

    // Go doc comments are syntactically identical to regular comments
    // So they should be extracted as regular comments
    assert!(
        units.iter().any(|u| u.content.contains("你好世界")),
        "Should extract Go doc comment with Chinese text"
    );
}

#[test]
fn test_csharp_doc_comment_extraction() {
    let coordinator = create_test_coordinator();

    let content = r#"
/// <summary>This is a doc comment with Chinese text: 你好世界</summary>
public class Test {
    // This is a regular comment with Chinese text: 你好
    public int x;
}
"#;

    let file = create_test_file(content, "test.cs");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    println!("C# - Extracted {} units:", units.len());
    for unit in &units {
        println!("  Type: {:?}, Content: {:?}", unit.node_type, unit.content);
    }

    // C# doc comments should be extracted
    assert!(
        units.iter().any(|u| u.content.contains("你好世界")),
        "Should extract C# doc comment with Chinese text"
    );
}

#[test]
fn test_javascript_jsdoc_extraction() {
    let coordinator = create_test_coordinator();

    let content = r#"
/**
 * This is a JSDoc comment with Chinese text: 你好世界
 */
function main() {
    // This is a regular comment with Chinese text: 你好
    var x = 5;
}
"#;

    let file = create_test_file(content, "test.js");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    println!("JavaScript - Extracted {} units:", units.len());
    for unit in &units {
        println!("  Type: {:?}, Content: {:?}", unit.node_type, unit.content);
    }

    // JSDoc comments should be extracted
    assert!(
        units.iter().any(|u| u.content.contains("你好世界")),
        "Should extract JSDoc comment with Chinese text"
    );
}

#[test]
fn test_java_javadoc_extraction() {
    let coordinator = create_test_coordinator();

    let content = r#"
/**
 * This is a Javadoc comment with Chinese text: 你好世界
 */
public class Test {
    // This is a regular comment with Chinese text: 你好
    int x;
}
"#;

    let file = create_test_file(content, "test.java");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    println!("Java - Extracted {} units:", units.len());
    for unit in &units {
        println!("  Type: {:?}, Content: {:?}", unit.node_type, unit.content);
    }

    // Javadoc comments should be extracted
    assert!(
        units.iter().any(|u| u.content.contains("你好世界")),
        "Should extract Javadoc comment with Chinese text"
    );
}

#[test]
fn test_rust_doc_comment_extraction_english_only() {
    let coordinator = create_test_coordinator();

    let content = r#"
/// This is a doc comment
fn main() {
    // This is a regular comment
    let x = 5;
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");

    println!("English Only - Extracted {} units:", units.len());
    for unit in &units {
        println!("  Type: {:?}, Content: {:?}", unit.node_type, unit.content);
    }

    // Should extract both doc comment and regular comment
    assert!(
        units.iter().any(|u| u.content.contains("doc comment")),
        "Should extract doc comment"
    );
    assert!(
        units.iter().any(|u| u.content.contains("regular comment")),
        "Should extract regular comment"
    );
}
