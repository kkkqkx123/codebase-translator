//! Language Filter Integration Tests for AUTO mode
//!
//! This test file verifies that language filtering works correctly when using
//! AUTO mode for source language detection. Specifically, it tests the scenario
//! where source_langs = ["AUTO"] and target_lang = "EN", which should filter out
//! English text and only translate non-English content.

use std::path::PathBuf;
use std::sync::Arc;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::core::traits::ExtractionConfig;
use codebase_translate::parser::filtering::{ContentFilter, FilterConfig};
use codebase_translate::parser::ParserConfig;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

#[test]
fn test_auto_to_en_filters_english_comments() {
    let strategy_config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: false,
        format_strings: false,
        log_messages: false,
        string_literals: false,
    };

    let filter_config = FilterConfig {
        source_langs: vec!["AUTO".to_string()],
        target_lang: "EN".to_string(),
        ..Default::default()
    };

    let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

    let parser_config = ParserConfig::default();
    let coordinator = ParserCoordinator::new(parser_config, strategy_config, filter)
        .expect("Failed to create coordinator");

    let content = r#"
// This is an English comment - should be filtered
// Another English comment - should be filtered
/// English doc comment - should be filtered

// 这是一个中文注释 - should be translated
/// 中文文档注释 - should be translated
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse_file(&file).expect("Parsing failed");

    let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();

    println!("\nExtracted texts ({} total):", texts.len());
    for (i, text) in texts.iter().enumerate() {
        println!("  [{}] {}", i + 1, text);
    }

    println!("\nChecking assertions...");

    assert!(
        !texts.iter().any(|t| t.contains("English comment")),
        "English comments should be filtered out in AUTO->EN mode"
    );
    assert!(
        !texts.iter().any(|t| t.contains("English doc comment")),
        "English doc comments should be filtered out in AUTO->EN mode"
    );
    assert!(
        texts.iter().any(|t| t.contains("中文注释")),
        "Chinese comments should be extracted for translation"
    );
    assert!(
        texts.iter().any(|t| t.contains("中文文档注释")),
        "Chinese doc comments should be extracted for translation"
    );
}

#[test]
fn test_auto_to_en_filters_english_strings() {
    let strategy_config = ExtractionConfig {
        comments: false,
        docstrings: false,
        error_messages: true,
        format_strings: true,
        log_messages: true,
        string_literals: false,
    };

    let filter_config = FilterConfig {
        source_langs: vec!["AUTO".to_string()],
        target_lang: "EN".to_string(),
        ..Default::default()
    };

    let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

    let parser_config = ParserConfig::default();
    let coordinator = ParserCoordinator::new(parser_config, strategy_config, filter)
        .expect("Failed to create coordinator");

    let content = r#"
fn main() {
    println!("This is English output - should be filtered");
    eprintln!("Another English message - should be filtered");
    panic!("English error - should be filtered");

    println!("这是中文输出 - should be translated");
    eprintln!("中文错误消息 - should be translated");
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse_file(&file).expect("Parsing failed");

    let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();

    assert!(
        !texts.iter().any(|t| t.contains("English output")),
        "English output strings should be filtered out in AUTO->EN mode"
    );
    assert!(
        !texts.iter().any(|t| t.contains("English message")),
        "English message strings should be filtered out in AUTO->EN mode"
    );
    assert!(
        !texts.iter().any(|t| t.contains("English error")),
        "English error strings should be filtered out in AUTO->EN mode"
    );
    assert!(
        texts.iter().any(|t| t.contains("中文输出")),
        "Chinese output strings should be extracted for translation"
    );
    assert!(
        texts.iter().any(|t| t.contains("中文错误消息")),
        "Chinese error strings should be extracted for translation"
    );
}

#[test]
fn test_auto_to_en_mixed_content() {
    let strategy_config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: true,
        format_strings: true,
        log_messages: true,
        string_literals: false,
    };

    let filter_config = FilterConfig {
        source_langs: vec!["AUTO".to_string()],
        target_lang: "EN".to_string(),
        ..Default::default()
    };

    let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

    let parser_config = ParserConfig::default();
    let coordinator = ParserCoordinator::new(parser_config, strategy_config, filter)
        .expect("Failed to create coordinator");

    let content = r#"
// Pure English comment - should be filtered
// Mixed English and Chinese comment 你好 - should be translated
// 纯中文注释 - should be translated

/// Pure English doc - should be filtered
/// Mixed English and Chinese doc 你好 - should be translated
/// 纯中文文档 - should be translated

fn test() {
    println!("Pure English string - should be filtered");
    println!("Mixed English and Chinese string 你好 - should be translated");
    println!("纯中文字符串 - should be translated");
}
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse_file(&file).expect("Parsing failed");

    let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();

    assert!(
        !texts.iter().any(|t| t.contains("Pure English comment")),
        "Pure English comments should be filtered"
    );
    assert!(
        !texts.iter().any(|t| t.contains("Pure English doc")),
        "Pure English doc comments should be filtered"
    );
    assert!(
        !texts.iter().any(|t| t.contains("Pure English string")),
        "Pure English strings should be filtered"
    );

    assert!(
        texts
            .iter()
            .any(|t| t.contains("Mixed English and Chinese comment")),
        "Mixed comments should be extracted"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("Mixed English and Chinese doc")),
        "Mixed doc comments should be extracted"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("Mixed English and Chinese string")),
        "Mixed strings should be extracted"
    );

    assert!(
        texts.iter().any(|t| t.contains("纯中文注释")),
        "Pure Chinese comments should be extracted"
    );
    assert!(
        texts.iter().any(|t| t.contains("纯中文文档")),
        "Pure Chinese doc comments should be extracted"
    );
    assert!(
        texts.iter().any(|t| t.contains("纯中文字符串")),
        "Pure Chinese strings should be extracted"
    );
}

#[test]
fn test_auto_to_zh_filters_chinese() {
    let strategy_config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: false,
        format_strings: false,
        log_messages: false,
        string_literals: false,
    };

    let filter_config = FilterConfig {
        source_langs: vec!["AUTO".to_string()],
        target_lang: "ZH".to_string(),
        ..Default::default()
    };

    let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

    let parser_config = ParserConfig::default();
    let coordinator = ParserCoordinator::new(parser_config, strategy_config, filter)
        .expect("Failed to create coordinator");

    let content = r#"
// 这是中文注释 - should be filtered
/// 中文文档注释 - should be filtered

// This is an English comment - should be translated
/// English doc comment - should be translated
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse_file(&file).expect("Parsing failed");

    let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();

    assert!(
        !texts.iter().any(|t| t.contains("中文注释")),
        "Chinese comments should be filtered out in AUTO->ZH mode"
    );
    assert!(
        !texts.iter().any(|t| t.contains("中文文档注释")),
        "Chinese doc comments should be filtered out in AUTO->ZH mode"
    );
    assert!(
        texts.iter().any(|t| t.contains("English comment")),
        "English comments should be extracted for translation"
    );
    assert!(
        texts.iter().any(|t| t.contains("English doc comment")),
        "English doc comments should be extracted for translation"
    );
}

#[test]
fn test_empty_source_langs_auto_behavior() {
    let strategy_config = ExtractionConfig {
        comments: true,
        docstrings: true,
        error_messages: false,
        format_strings: false,
        log_messages: false,
        string_literals: false,
    };

    let filter_config = FilterConfig {
        source_langs: vec![],
        target_lang: "EN".to_string(),
        ..Default::default()
    };

    let filter = Arc::new(ContentFilter::new(filter_config).expect("Failed to create filter"));

    let parser_config = ParserConfig::default();
    let coordinator = ParserCoordinator::new(parser_config, strategy_config, filter)
        .expect("Failed to create coordinator");

    let content = r#"
// This is an English comment - should be filtered
// 这是一个中文注释 - should be translated
"#;

    let file = create_test_file(content, "test.rs");
    let units = coordinator.parse_file(&file).expect("Parsing failed");

    let texts: Vec<_> = units.iter().map(|u| u.content.as_str()).collect();

    assert!(
        !texts.iter().any(|t| t.contains("English comment")),
        "English comments should be filtered out when source_langs is empty (AUTO mode)"
    );
    assert!(
        texts.iter().any(|t| t.contains("中文注释")),
        "Chinese comments should be extracted for translation"
    );
}
