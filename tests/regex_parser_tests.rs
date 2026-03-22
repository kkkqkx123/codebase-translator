//! Tests for regex parsers with StringProcessor integration
//!
//! This test file verifies that regex parsers (FallbackParser, HtmlParser,
//! ShellParser, SqlParser) correctly use StringProcessor for cleaning
//! comments and strings, ensuring consistency with tree-sitter parsers.

use std::path::PathBuf;

use codebase_translate::core::models::File;
use codebase_translate::parser::regex_parsers::{
    FallbackParser, HtmlParser, ShellParser, SqlParser,
};
use codebase_translate::parser::ParserConfig;
use codebase_translate::Parser;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

#[test]
fn test_fallback_parser_block_comment_with_asterisks() {
    let parser = FallbackParser::new(ParserConfig::default());

    let content = r#"/*
 * This is a block comment
 * with multiple lines
 * and asterisk prefixes
 */
key = value"#;

    let file = create_test_file(content, "test.txt");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    assert!(
        !comment.content.contains('*'),
        "Should not contain asterisk prefixes"
    );
    assert!(comment.content.contains("This is a block comment"));
    assert!(comment.content.contains("with multiple lines"));
}

#[test]
fn test_fallback_parser_multiline_block_comment() {
    let parser = FallbackParser::new(ParserConfig::default());

    let content = r#"/* This is a multiline
block comment without
asterisk prefixes */"#;

    let file = create_test_file(content, "test.txt");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    assert!(comment.content.contains("This is a multiline"));
    assert!(comment.content.contains("block comment without"));
}

#[test]
fn test_html_parser_multiline_comment() {
    let parser = HtmlParser::new(ParserConfig::default());

    let content = r#"<!--
This is a multiline
HTML comment
-->
<div>Hello</div>"#;

    let file = create_test_file(content, "test.html");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    assert!(comment.content.contains("This is a multiline"));
    assert!(comment.content.contains("HTML comment"));
}

#[test]
fn test_shell_parser_powershell_block_comment() {
    let parser = ShellParser::new(ParserConfig::default());

    let content = r#"<#
This is a multi-line
PowerShell comment
#>
Write-Host "Hello""#;

    let file = create_test_file(content, "test.ps1");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    assert!(comment.content.contains("This is a multi-line"));
    assert!(comment.content.contains("PowerShell comment"));
}

#[test]
fn test_sql_parser_block_comment() {
    let parser = SqlParser::new(ParserConfig::default());

    let content = r#"/*
 * This is a SQL block comment
 * with multiple lines
 */
SELECT * FROM users;"#;

    let file = create_test_file(content, "test.sql");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    assert!(
        !comment.content.contains('*'),
        "Should not contain asterisk prefixes"
    );
    assert!(comment.content.contains("This is a SQL block comment"));
    assert!(comment.content.contains("with multiple lines"));
}

#[test]
fn test_fallback_parser_line_comment() {
    let parser = FallbackParser::new(ParserConfig::default());

    let content = r#"# This is a line comment
key = value  # inline comment
// Another line comment
-- Yet another comment"#;

    let file = create_test_file(content, "test.txt");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    for unit in &units {
        assert!(!unit.content.starts_with('#'), "Should not start with #");
        assert!(!unit.content.starts_with("//"), "Should not start with //");
        assert!(!unit.content.starts_with("--"), "Should not start with --");
    }
}

#[test]
fn test_shell_parser_bash_line_comment() {
    let parser = ShellParser::new(ParserConfig::default());

    let content = r#"#!/bin/bash
# This is a bash comment
echo "hello world"  # inline comment"#;

    let file = create_test_file(content, "test.sh");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    for unit in &units {
        assert!(!unit.content.starts_with('#'), "Should not start with #");
    }
}

#[test]
fn test_sql_parser_line_comment() {
    let parser = SqlParser::new(ParserConfig::default());

    let content = r#"-- This is a SQL comment
SELECT * FROM users;  -- inline comment"#;

    let file = create_test_file(content, "test.sql");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    for unit in &units {
        assert!(!unit.content.starts_with("--"), "Should not start with --");
    }
}

#[test]
fn test_fallback_parser_string_literal() {
    let parser = FallbackParser::new(ParserConfig::default());

    let content = r#"key = "This is a string literal"
another = 'Another string'"#;

    let file = create_test_file(content, "test.txt");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    for unit in &units {
        assert!(
            !unit.content.starts_with('"'),
            "Should not start with quote"
        );
        assert!(!unit.content.ends_with('"'), "Should not end with quote");
        assert!(
            !unit.content.starts_with('\''),
            "Should not start with quote"
        );
        assert!(!unit.content.ends_with('\''), "Should not end with quote");
    }
}

#[test]
fn test_sql_parser_string_literal() {
    let parser = SqlParser::new(ParserConfig::default());

    let content = r#"SELECT 'This is a string' FROM users;
SELECT "Another string" FROM products;"#;

    let file = create_test_file(content, "test.sql");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    for unit in &units {
        assert!(
            !unit.content.starts_with('\''),
            "Should not start with quote"
        );
        assert!(!unit.content.ends_with('\''), "Should not end with quote");
    }
}

#[test]
fn test_fallback_parser_javadoc_style_comment() {
    let parser = FallbackParser::new(ParserConfig::default());

    let content = r#"/**
 * Javadoc-style comment
 * @param name The name parameter
 * @return The result
 */
function test() {}"#;

    let file = create_test_file(content, "test.txt");
    let units = parser.parse(&file).expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    assert!(
        !comment.content.contains('*'),
        "Should not contain asterisk prefixes"
    );
    assert!(comment.content.contains("Javadoc-style comment"));
    assert!(comment.content.contains("@param name The name parameter"));
}

#[test]
fn test_consistency_with_tree_sitter_block_comments() {
    let fallback_parser = FallbackParser::new(ParserConfig::default());

    let content = r#"/*
 * This is a test comment
 * with multiple lines
 */"#;

    let file = create_test_file(content, "test.txt");
    let units = fallback_parser
        .parse(&file)
        .expect("Parsing should succeed");

    assert!(!units.is_empty());

    let comment = &units[0];
    let lines: Vec<&str> = comment.content.lines().collect();

    assert!(lines.len() >= 2, "Should have at least 2 lines");
    assert!(lines[0].contains("This is a test comment"));
    assert!(lines[1].contains("with multiple lines"));
}
