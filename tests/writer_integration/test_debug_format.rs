//! Debug test for format_multiline_translation

use codebase_translate::writer::TranslationApplier;

#[test]
fn test_debug_format() {
    let raw_match = "/*\n * Line 1\n * Line 2\n */";
    let translated = "第一行\n第二行";

    // Use a public method or create a test wrapper
    // For now, let's just print what we expect
    println!("Raw match: {:?}", raw_match);
    println!("Translated: {:?}", translated);

    // Let's manually trace through the format_multiline_translation logic
    let raw_lines: Vec<&str> = raw_match.lines().collect();
    let translated_lines: Vec<&str> = translated.lines().collect();

    println!("Raw lines: {:?}", raw_lines);
    println!("Translated lines: {:?}", translated_lines);

    // Check if block comment
    let is_block_comment = raw_lines
        .first()
        .map(|line| line.trim().starts_with("/*"))
        .unwrap_or(false);
    println!("Is block comment: {}", is_block_comment);
}
