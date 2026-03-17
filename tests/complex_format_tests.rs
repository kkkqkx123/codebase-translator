//! Complex format preservation tests for writer integration

use codebase_translate::writer::{FileWriter, WriterConfig};
use codebase_translate::core::models::{CommentStyle, FormatInfo, NodeType, Position};

use crate::writer_common::*;

#[tokio::test]
async fn test_complex_nested_block_comment() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"fn main() {
    /* Outer comment
     * /* Inner comment */
     * More outer comment
     */
    println!("Hello");
}"#;

    let file = create_test_file(&temp_path, "nested_comment.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::BlockMulti,
        base_indent: "    ".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![create_translation_unit_with_format(
        "1",
        "/* Outer comment\n     * /* Inner comment */\n     * More outer comment\n     */",
        2,
        5,
        67,
        format_info,
    )];
    units[0].set_translated("外部注释\n内部注释\n更多外部注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    assert!(written_content.contains("fn main()"), "Function declaration should be preserved");
    assert!(written_content.contains("println!(\"Hello\")"), "Code should be preserved");
    assert!(written_content.contains("/*"), "Comment start should be preserved");
    assert!(written_content.contains("*/"), "Comment end should be preserved");
}

#[tokio::test]
async fn test_string_literal_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"fn main() {
    let message = "Hello world";
    println!("{}", message);
}"#;

    let file = create_test_file(&temp_path, "string_literal.rs", content).await;

    let mut units = vec![create_translation_unit("1", "\"Hello world\"", 2, 20, 32)];
    units[0].set_translated("\"你好世界\"");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    assert!(written_content.contains("fn main()"), "Function declaration should be preserved");
    assert!(written_content.contains("let message ="), "Variable declaration should be preserved");
    assert!(written_content.contains("println!"), "Macro call should be preserved");
    assert!(written_content.contains("\"你好世界\""), "String literal should be translated");
}

#[tokio::test]
async fn test_multiline_string_literal() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"fn main() {
    let message = r#"This is a
multiline
string literal"#;
    println!("{}", message);
}"##;

    let file = create_test_file(&temp_path, "multiline_string.rs", content).await;

    let mut units = vec![create_translation_unit("1", "This is a", 2, 22, 29)];
    units[0].set_translated("这是一个");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    assert!(written_content.contains("r#"), "Raw string prefix should be preserved");
    assert!(written_content.contains("#;"), "Raw string suffix should be preserved");
    assert!(written_content.contains("multiline"), "Other lines should be preserved");
}

#[tokio::test]
async fn test_mixed_comments_and_code() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"// Top comment
fn main() {
    // Function comment
    let x = 5; // Inline comment
    /* Block comment
     * with multiple lines
     */
    let y = 10;
}
// Bottom comment"##;

    let file = create_test_file(&temp_path, "mixed_comments.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::BlockMulti,
        base_indent: "    ".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![
        create_translation_unit("1", "// Top comment", 1, 1, 13),
        create_translation_unit("2", "// Function comment", 3, 5, 22),
        create_translation_unit("3", "// Inline comment", 4, 16, 32),
        create_translation_unit_with_format(
            "4",
            "/* Block comment\n     * with multiple lines\n     */",
            5,
            5,
            52,
            format_info,
        ),
        create_translation_unit("5", "// Bottom comment", 9, 1, 15),
    ];

    units[0].set_translated("顶部注释");
    units[1].set_translated("函数注释");
    units[2].set_translated("行内注释");
    units[3].set_translated("块注释\n多行");
    units[4].set_translated("底部注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    assert!(written_content.contains("fn main()"), "Function should be preserved");
    assert!(written_content.contains("let x = 5;"), "Variable x should be preserved");
    assert!(written_content.contains("let y = 10;"), "Variable y should be preserved");
    assert!(written_content.contains("顶部注释"), "Top comment should be translated");
    assert!(written_content.contains("函数注释"), "Function comment should be translated");
    assert!(written_content.contains("行内注释"), "Inline comment should be translated");
    assert!(written_content.contains("底部注释"), "Bottom comment should be translated");
}

#[tokio::test]
async fn test_indented_block_comment_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"fn main() {
        if true {
            /* Indented block comment
             * with multiple levels
             * of indentation
             */
            let x = 5;
        }
    }"##;

    let file = create_test_file(&temp_path, "indented_block.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::BlockMulti,
        base_indent: "        ".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![create_translation_unit_with_format(
        "1",
        "/* Indented block comment\n             * with multiple levels\n             * of indentation\n             */",
        3,
        13,
        91,
        format_info,
    )];
    units[0].set_translated("缩进块注释\n多级缩进\n更多缩进");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    assert!(written_content.contains("fn main()"), "Function should be preserved");
    assert!(written_content.contains("if true"), "If statement should be preserved");
    assert!(written_content.contains("let x = 5;"), "Variable should be preserved");
    assert!(written_content.contains("/*"), "Comment start should be preserved");
    assert!(written_content.contains("*/"), "Comment end should be preserved");
    assert!(written_content.contains("缩进块注释"), "Comment should be translated");
}

#[tokio::test]
async fn test_doc_comment_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"/// This is a doc comment
/// with multiple lines
fn example() -> i32 {
    42
}

/// Another doc comment
/// for another function
fn another() -> i32 {
    100
}"##;

    let file = create_test_file(&temp_path, "doc_comment.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocOuter,
        base_indent: "".to_string(),
        line_prefix: Some("/// ".to_string()),
        ends_with_newline: false,
        is_multiline: true,
    };

    let mut unit = create_translation_unit_with_format(
        "1",
        "This is a doc comment\nwith multiple lines",
        1,
        1,
        24,
        format_info.clone(),
    );
    unit.end_pos.line = 2;
    unit.end_pos.column = 24;
    unit.set_translated("这是一个文档注释\n多行");

    let mut unit2 = create_translation_unit_with_format(
        "2",
        "Another doc comment\nfor another function",
        7,
        1,
        25,
        format_info,
    );
    unit2.end_pos.line = 8;
    unit2.end_pos.column = 25;
    unit2.set_translated("另一个文档注释\n用于另一个函数");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &[unit.clone(), unit2.clone()]).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_doc_comment_preservation",
        content,
        &written_content,
        &[unit, unit2]
    );

    assert!(written_content.contains("fn example()"), "Function example should be preserved");
    assert!(written_content.contains("fn another()"), "Function another should be preserved");
    assert!(written_content.contains("-> i32"), "Return types should be preserved");
    assert!(written_content.contains("/// 这是一个文档注释"), "Doc comment line 1 should be translated");
    assert!(written_content.contains("/// 多行"), "Doc comment line 2 should be translated");
    assert!(written_content.contains("/// 另一个文档注释"), "Second doc comment line 1 should be translated");
    assert!(written_content.contains("/// 用于另一个函数"), "Second doc comment line 2 should be translated");
}

#[tokio::test]
async fn test_string_with_special_characters() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"fn main() {
    let path = "C:\\Users\\test\\file.txt";
    let quote = "He said Hello";
    let escape = "Line 1 text";
}"##;

    let file = create_test_file(&temp_path, "special_chars.rs", content).await;

    let mut units = vec![
        create_translation_unit("1", "He said Hello", 3, 18, 31),
        create_translation_unit("2", "Line 1 text", 4, 18, 30),
    ];

    units[0].set_translated("他说你好");
    units[1].set_translated("第一行文本");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_string_with_special_characters",
        content,
        &written_content,
        &units
    );

    assert!(written_content.contains("let path ="), "Path variable should be preserved");
    assert!(written_content.contains("let quote ="), "Quote variable should be preserved");
    assert!(written_content.contains("他说你好"), "Quote string should be translated");
    assert!(written_content.contains("第一行文本"), "Line 1 text should be translated");
}

#[tokio::test]
async fn test_complex_code_structure_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"// Module comment
mod example {
    // Nested module comment
    pub struct Example {
        /// Field doc comment
        pub field: i32,
    }

    impl Example {
        // Method comment
        pub fn new() -> Self {
            Self { field: 0 }
        }
    }
}"##;

    let file = create_test_file(&temp_path, "complex_structure.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocOuter,
        base_indent: "        ".to_string(),
        line_prefix: Some("/// ".to_string()),
        ends_with_newline: false,
        is_multiline: false,
    };

    let mut units = vec![
        create_translation_unit("1", "Module comment", 1, 4, 18),
        create_translation_unit("2", "Nested module comment", 3, 8, 29),
        create_translation_unit_with_format("3", "Field doc comment", 5, 13, 30, format_info.clone()),
        create_translation_unit("4", "Method comment", 10, 9, 26),
    ];

    units[0].set_translated("模块注释");
    units[1].set_translated("嵌套模块注释");
    units[2].set_translated("字段文档注释");
    units[3].set_translated("方法注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_complex_code_structure_preservation",
        content,
        &written_content,
        &units
    );

    assert!(written_content.contains("mod example"), "Module should be preserved");
    assert!(written_content.contains("pub struct Example"), "Struct should be preserved");
    assert!(written_content.contains("pub field: i32"), "Field should be preserved");
    assert!(written_content.contains("impl Example"), "Impl block should be preserved");
    assert!(written_content.contains("pub fn new()"), "Method should be preserved");
    assert!(written_content.contains("Self { field: 0 }"), "Method body should be preserved");
    assert!(written_content.contains("模块注释"), "Module comment should be translated");
    assert!(written_content.contains("嵌套模块注释"), "Nested comment should be translated");
}

#[tokio::test]
async fn test_multiline_doc_block_comment() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"/**
 * This is a doc block comment
 * with multiple lines
 * and detailed information
 */
fn example() -> i32 {
    42
}"##;

    let file = create_test_file(&temp_path, "doc_block.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocBlock,
        base_indent: "".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: true,
    };

    let mut unit = create_translation_unit_with_format(
        "1",
        "This is a doc block comment\nwith multiple lines\nand detailed information",
        1,
        1,
        4,
        format_info,
    );
    unit.start_pos.line = 1;
    unit.start_pos.column = 1;
    unit.end_pos.line = 5;
    unit.end_pos.column = 4;
    unit.set_translated("这是一个文档块注释\n多行\n详细信息");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &[unit.clone()]).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_multiline_doc_block_comment",
        content,
        &written_content,
        &[unit]
    );

    assert!(written_content.contains("fn example()"), "Function should be preserved");
    assert!(written_content.contains("/**"), "Doc block start should be preserved");
    assert!(written_content.contains("*/"), "Doc block end should be preserved");
    assert!(written_content.contains("这是一个文档块注释"), "Comment should be translated");
    assert!(written_content.contains("多行"), "Second line should be translated");
    assert!(written_content.contains("详细信息"), "Third line should be translated");
}

#[tokio::test]
async fn test_empty_lines_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"// First comment

fn main() {

    // Nested comment
    let x = 5;

    println!("{}", x);

}

// Last comment"##;

    let file = create_test_file(&temp_path, "empty_lines.rs", content).await;

    let mut units = vec![
        create_translation_unit("1", "First comment", 1, 4, 17),
        create_translation_unit("2", "Nested comment", 5, 8, 22),
        create_translation_unit("3", "Last comment", 12, 4, 16),
    ];

    units[0].set_translated("第一个注释");
    units[1].set_translated("嵌套注释");
    units[2].set_translated("最后一个注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_empty_lines_preservation",
        content,
        &written_content,
        &units
    );

    assert!(written_content.contains("fn main()"), "Function should be preserved");
    assert!(written_content.contains("第一个注释"), "First comment should be translated");
    assert!(written_content.contains("嵌套注释"), "Nested comment should be translated");
    assert!(written_content.contains("最后一个注释"), "Last comment should be translated");

    let lines: Vec<&str> = written_content.lines().collect();
    assert_eq!(lines.len(), 12, "Empty lines should be preserved");
}

#[tokio::test]
async fn test_multiline_comment_merged() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r##"/// This is a doc comment
/// with multiple lines
/// that should be merged
fn example() -> i32 {
    42
}"##;

    let file = create_test_file(&temp_path, "multiline_merged.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocOuter,
        base_indent: "".to_string(),
        line_prefix: Some("/// ".to_string()),
        ends_with_newline: false,
        is_multiline: true,
    };

    let mut unit = create_translation_unit_with_format(
        "1",
        "This is a doc comment\nwith multiple lines\nthat should be merged",
        1,
        1,
        24,
        format_info,
    );
    unit.end_pos.line = 3;
    unit.end_pos.column = 24;
    unit.set_translated("这是一个文档注释\n多行\n应该被合并");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &[unit.clone()]).await;
    assert!(result.is_ok());

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_multiline_comment_merged",
        content,
        &written_content,
        &[unit]
    );

    assert!(written_content.contains("fn example()"), "Function should be preserved");
    assert!(written_content.contains("/// 这是一个文档注释"), "First line should be translated with prefix");
    assert!(written_content.contains("/// 多行"), "Second line should be translated with prefix");
    assert!(written_content.contains("/// 应该被合并"), "Third line should be translated with prefix");
}
