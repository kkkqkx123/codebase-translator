//! Tests for multiline doc comment handling

use codebase_translate::parser::core::string_processor::StringProcessor;

#[test]
fn test_multiline_outer_doc_comment() {
    let processor = StringProcessor::new();

    // Standard multiline outer doc comment
    let text = "/// 第一行\n/// 第二行\n/// 第三行";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "第一行\n第二行\n第三行");
}

#[test]
fn test_multiline_inner_doc_comment() {
    let processor = StringProcessor::new();

    // Standard multiline inner doc comment
    let text = "//! 模块文档第一行\n//! 模块文档第二行";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "模块文档第一行\n模块文档第二行");
}

#[test]
fn test_multiline_doc_with_empty_lines() {
    let processor = StringProcessor::new();

    // Multiline doc with empty lines (common in Rust documentation)
    let text = "/// 标题\n/// \n/// 内容描述";
    let result = processor.clean_doc_comment(text);
    // Empty lines after stripping /// should be preserved
    assert_eq!(result, "标题\n\n内容描述");
}

#[test]
fn test_multiline_doc_with_code_example() {
    let processor = StringProcessor::new();

    // Doc comment with code example (markdown code block)
    let text = "/// 示例代码\n/// ```\n/// let x = 1;\n/// ```";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "示例代码\n```\nlet x = 1;\n```");
}

#[test]
fn test_multiline_doc_with_trailing_newline() {
    let processor = StringProcessor::new();

    // Multiline doc with trailing newline (from tree-sitter)
    let text = "/// 第一行\n/// 第二行\n";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "第一行\n第二行");
}

#[test]
fn test_doc_with_mixed_content() {
    let processor = StringProcessor::new();

    // This is the problematic case - doc followed by code
    // Should only extract doc lines
    let text = "/// 文档说明\npub fn foo() {}";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "文档说明");
}

#[test]
fn test_multiline_doc_followed_by_code() {
    let processor = StringProcessor::new();

    // Multiple doc lines followed by code
    let text = "/// 第一行\n/// 第二行\npub fn foo() {}";
    let result = processor.clean_doc_comment(text);
    assert_eq!(result, "第一行\n第二行");
}
