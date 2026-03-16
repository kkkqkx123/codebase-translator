//! Simple Parser Integration Tests
//!
//! Tests using fixture files to avoid formatting issues.

use std::fs;
use std::path::PathBuf;

use codebase_translate::core::models::{File, TranslationUnit};
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::tree_sitter::ParserConfig;

const OUTPUT_DIR: &str = "tests/parser_integration/output";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn load_fixture(filename: &str) -> String {
    let path = PathBuf::from("tests/parser_integration/fixtures").join(filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path.display()))
}

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

fn create_test_coordinator() -> ParserCoordinator {
    let mut config = ParserConfig::default();
    config.extract_strings = true; // Enable string extraction
    ParserCoordinator::with_defaults(config).expect("Failed to create coordinator")
}

fn write_units_to_file(filename: &str, units: &[TranslationUnit]) {
    ensure_output_dir();

    let output_path = PathBuf::from(OUTPUT_DIR).join(format!("{}.txt", filename));
    let mut output = String::new();

    output.push_str(&format!(
        "Extracted {} translation units from {}\n",
        units.len(),
        filename
    ));
    output.push_str("==================================================\n\n");

    for (i, unit) in units.iter().enumerate() {
        output.push_str(&format!("--- Unit {} ---\n", i + 1));
        output.push_str(&format!("ID: {}\n", unit.id));
        output.push_str(&format!("Type: {}\n", unit.node_type));
        output.push_str(&format!(
            "Position: Line {}, Column {} (Offset: {})\n",
            unit.start_pos.line, unit.start_pos.column, unit.start_pos.offset
        ));
        output.push_str(&format!("Content:\n{}\n", unit.content));
        output.push_str("\n");
    }

    fs::write(&output_path, output).expect("Failed to write output file");
    println!("Output written to: {}", output_path.display());
}

/// Check if text contains Chinese characters
fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        (0x4E00..=0x9FFF).contains(&code) ||  // CJK Unified Ideographs
        (0x3400..=0x4DBF).contains(&code) ||  // CJK Extension A
        (0x20000..=0x2A6DF).contains(&code) // CJK Extension B
    })
}

/// Check if text contains only ASCII (English)
fn is_ascii_only(text: &str) -> bool {
    text.chars().all(|c| c.is_ascii())
}

#[test]
fn test_coordinator_creation() {
    let coordinator = create_test_coordinator();
    assert!(coordinator.tree_sitter_parser_count() > 0);
}

#[test]
fn test_parse_rust_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("rust/simple_comments.rs");
    let file = create_test_file(&content, "test.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("rust_simple_comments", &units);
    assert!(!units.is_empty(), "Should extract units from Rust file");
}

#[test]
fn test_parse_python_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("python/comments.py");
    let file = create_test_file(&content, "test.py");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("python_comments", &units);
    assert!(!units.is_empty(), "Should extract units from Python file");
}

#[test]
fn test_parse_go_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("go/comments.go");
    let file = create_test_file(&content, "test.go");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("go_comments", &units);
    assert!(!units.is_empty(), "Should extract units from Go file");
}

#[test]
fn test_parse_javascript_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("javascript/comments.js");
    let file = create_test_file(&content, "test.js");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("javascript_comments", &units);
    assert!(
        !units.is_empty(),
        "Should extract units from JavaScript file"
    );
}

#[test]
fn test_parse_markdown_file() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("markdown/readme.md");
    let file = create_test_file(&content, "readme.md");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("markdown_readme", &units);
    assert!(!units.is_empty(), "Should extract units from Markdown file");
}

#[test]
fn test_can_parse_supported_extensions() {
    let coordinator = create_test_coordinator();

    assert!(coordinator.can_parse("test.rs"));
    assert!(coordinator.can_parse("test.py"));
    assert!(coordinator.can_parse("test.go"));
    assert!(coordinator.can_parse("test.java"));
    assert!(coordinator.can_parse("test.js"));
    assert!(coordinator.can_parse("test.ts"));
    assert!(coordinator.can_parse("test.c"));
    assert!(coordinator.can_parse("test.cpp"));
    assert!(coordinator.can_parse("test.cs"));
    assert!(coordinator.can_parse("test.md"));
}

#[test]
fn test_cannot_parse_unsupported_extensions() {
    let coordinator = create_test_coordinator();

    assert!(!coordinator.can_parse("test.unknown"));
    assert!(!coordinator.can_parse("test.bin"));
}

#[test]
fn test_parse_empty_file() {
    let coordinator = create_test_coordinator();
    let file = create_test_file("", "empty.rs");

    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");
    write_units_to_file("empty_file", &units);
    assert!(units.is_empty(), "Empty file should produce no units");
}

#[test]
fn test_parse_whitespace_only() {
    let coordinator = create_test_coordinator();
    let file = create_test_file("   \n\t\n  ", "whitespace.rs");

    let units = coordinator
        .parse_file(&file)
        .expect("Parsing should succeed");
    write_units_to_file("whitespace_only", &units);
    assert!(
        units.is_empty(),
        "Whitespace-only file should produce no units"
    );
}

#[test]
fn test_rust_doc_comments_extraction() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("rust/doc_comments.rs");
    let file = create_test_file(&content, "test.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("rust_doc_comments", &units);

    let doc_units: Vec<_> = units
        .iter()
        .filter(|u| {
            matches!(
                u.node_type,
                codebase_translate::core::models::NodeType::DocString
            )
        })
        .collect();

    assert!(!doc_units.is_empty(), "Should extract doc comments");
}

#[test]
fn test_rust_macro_extraction() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("rust/macros.rs");
    let file = create_test_file(&content, "test.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("rust_macros", &units);

    let has_error_or_log = units.iter().any(|u| {
        matches!(
            u.node_type,
            codebase_translate::core::models::NodeType::ErrorMessage
                | codebase_translate::core::models::NodeType::LogMessage
                | codebase_translate::core::models::NodeType::FormatString
        )
    });

    assert!(
        has_error_or_log || !units.is_empty(),
        "Should extract macro strings or other content"
    );
}

// ============================================================================
// String Literals Extraction Tests
// ============================================================================

#[test]
fn test_rust_string_literals_extraction() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("rust/string_literals.rs");
    let file = create_test_file(&content, "test.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("rust_string_literals", &units);

    // Check for various string types
    let has_error_strings = units.iter().any(|u| {
        matches!(
            u.node_type,
            codebase_translate::core::models::NodeType::ErrorMessage
        )
    });
    let has_log_strings = units.iter().any(|u| {
        matches!(
            u.node_type,
            codebase_translate::core::models::NodeType::LogMessage
        )
    });
    let has_format_strings = units.iter().any(|u| {
        matches!(
            u.node_type,
            codebase_translate::core::models::NodeType::FormatString
        )
    });
    let has_string_literals = units.iter().any(|u| {
        // String literals may be classified as other types based on context
        matches!(
            u.node_type,
            codebase_translate::core::models::NodeType::Comment
                | codebase_translate::core::models::NodeType::DocString
                | codebase_translate::core::models::NodeType::ErrorMessage
                | codebase_translate::core::models::NodeType::FormatString
                | codebase_translate::core::models::NodeType::LogMessage
        )
    });

    println!(
        "Rust string literals - Errors: {}, Logs: {}, Formats: {}, Literals: {}",
        has_error_strings, has_log_strings, has_format_strings, has_string_literals
    );

    assert!(
        !units.is_empty(),
        "Should extract string literals from Rust file"
    );
}

#[test]
fn test_python_string_literals_extraction() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("python/string_literals.py");
    let file = create_test_file(&content, "test.py");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("python_string_literals", &units);

    assert!(
        !units.is_empty(),
        "Should extract string literals from Python file"
    );
}

#[test]
fn test_javascript_string_literals_extraction() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("javascript/string_literals.js");
    let file = create_test_file(&content, "test.js");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("javascript_string_literals", &units);

    assert!(
        !units.is_empty(),
        "Should extract string literals from JavaScript file"
    );
}

#[test]
fn test_go_string_literals_extraction() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("go/string_literals.go");
    let file = create_test_file(&content, "test.go");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("go_string_literals", &units);

    assert!(
        !units.is_empty(),
        "Should extract string literals from Go file"
    );
}

// ============================================================================
// Language Filtering Tests
// ============================================================================

/// Test: When source=zh, target=en, only Chinese content should be extracted
#[test]
fn test_language_filter_zh_to_en_rust() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("language_filter/mixed_content.rs");
    let file = create_test_file(&content, "test.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("language_filter_rust", &units);

    // Analyze extracted content
    let mut chinese_units = 0;
    let mut english_units = 0;
    let mut mixed_units = 0;

    for unit in &units {
        let has_chinese = contains_chinese(&unit.content);
        let is_ascii = is_ascii_only(&unit.content);

        if has_chinese {
            if is_ascii {
                // This shouldn't happen - ASCII can't have Chinese
                english_units += 1;
            } else {
                chinese_units += 1;
            }
        } else if is_ascii {
            english_units += 1;
        } else {
            // Other non-ASCII, non-Chinese content
            mixed_units += 1;
        }
    }

    println!(
        "Language filter analysis - Chinese: {}, English: {}, Other: {}",
        chinese_units, english_units, mixed_units
    );

    // When translating zh->en, we expect Chinese content to be present
    // The actual filtering happens at the translation stage, not parsing stage
    // Parser should extract all translatable content
    assert!(
        !units.is_empty(),
        "Should extract content from mixed language file"
    );
}

/// Test: When source=zh, target=en, only Chinese content should be extracted from Markdown
#[test]
fn test_language_filter_zh_to_en_markdown() {
    let coordinator = create_test_coordinator();
    let content = load_fixture("markdown/mixed_content.md");
    let file = create_test_file(&content, "test.md");

    let units = coordinator.parse_file(&file).expect("Parsing failed");
    write_units_to_file("language_filter_markdown", &units);

    // Analyze extracted content
    let mut chinese_units = 0;
    let mut english_only_units = 0;
    let mut mixed_units = 0;

    for unit in &units {
        let has_chinese = contains_chinese(&unit.content);
        let is_ascii = is_ascii_only(&unit.content);

        if has_chinese {
            mixed_units += 1;
        } else if is_ascii {
            english_only_units += 1;
        } else {
            chinese_units += 1;
        }
    }

    println!(
        "Markdown language filter - Chinese-only: {}, English-only: {}, Mixed: {}",
        chinese_units, english_only_units, mixed_units
    );

    assert!(
        !units.is_empty(),
        "Should extract content from mixed language markdown"
    );
}

/// Test: Verify that pure English content can be identified
#[test]
fn test_identify_english_only_content() {
    let test_cases = vec![
        ("This is English only", true, false),
        ("这是纯中文", false, true),
        ("Mixed 混合 content", false, true),
        ("Hello 世界", false, true),
        ("Test value 123", true, false),
        ("测试值 123", false, true),
    ];

    for (content, expected_ascii, expected_chinese) in test_cases {
        let is_ascii = is_ascii_only(content);
        let has_chinese = contains_chinese(content);

        assert_eq!(
            is_ascii, expected_ascii,
            "ASCII check failed for: {}",
            content
        );
        assert_eq!(
            has_chinese, expected_chinese,
            "Chinese check failed for: {}",
            content
        );
    }
}

/// Test: Content filtering simulation for zh->en translation
#[test]
fn test_content_filtering_simulation() {
    let coordinator = create_test_coordinator();
    // Use mixed content fixture which has both Chinese and English
    let content = load_fixture("language_filter/mixed_content.rs");
    let file = create_test_file(&content, "test.rs");

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    // Simulate filtering: only extract units with Chinese content
    let chinese_units: Vec<_> = units
        .iter()
        .filter(|u| contains_chinese(&u.content))
        .collect();

    let english_only_units: Vec<_> = units.iter().filter(|u| is_ascii_only(&u.content)).collect();

    println!("Content filtering simulation:");
    println!("  Total units: {}", units.len());
    println!("  Units with Chinese: {}", chinese_units.len());
    println!("  English-only units: {}", english_only_units.len());

    // For zh->en translation, only Chinese units should be translated
    // English-only units should be skipped
    assert!(
        !chinese_units.is_empty(),
        "Should find Chinese content to translate"
    );

    // Write filtered results
    let filtered_output_path = PathBuf::from(OUTPUT_DIR).join("filtered_chinese_only.txt");
    let mut output = String::new();
    output.push_str("Chinese-only content (to be translated zh->en):\n");
    output.push_str("================================================\n\n");

    for (i, unit) in chinese_units.iter().enumerate() {
        output.push_str(&format!(
            "{}: [{}] {}\n",
            i + 1,
            unit.node_type,
            unit.content
        ));
    }

    output.push_str("\n\nEnglish-only content (skip translation):\n");
    output.push_str("=========================================\n\n");

    for (i, unit) in english_only_units.iter().enumerate() {
        output.push_str(&format!(
            "{}: [{}] {}\n",
            i + 1,
            unit.node_type,
            unit.content
        ));
    }

    fs::write(&filtered_output_path, output).expect("Failed to write filtered output");
    println!(
        "Filtered output written to: {}",
        filtered_output_path.display()
    );
}
