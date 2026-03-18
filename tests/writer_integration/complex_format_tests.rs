//! Complex format preservation tests for writer integration

use codebase_translate::core::models::{CommentStyle, FormatInfo, NodeType, Position};
use codebase_translate::writer::{FileWriter, WriterConfig};

use super::common::*;

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

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn main()"),
        "Function declaration should be preserved"
    );
    assert!(
        written_content.contains("println!(\"Hello\")"),
        "Code should be preserved"
    );
    assert!(
        written_content.contains("/*"),
        "Comment start should be preserved"
    );
    assert!(
        written_content.contains("*/"),
        "Comment end should be preserved"
    );
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

    let mut units = vec![create_translation_unit("1", "Hello world", 2, 20, 32)];
    units[0].set_translated("你好世界");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn main()"),
        "Function declaration should be preserved"
    );
    assert!(
        written_content.contains("let message ="),
        "Variable declaration should be preserved"
    );
    assert!(
        written_content.contains("println!"),
        "Macro call should be preserved"
    );
    assert!(
        written_content.contains("\"你好世界\""),
        "String literal should be translated"
    );
}

#[tokio::test]
async fn test_multiline_string_literal() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "fn main() {\n    let message = r#\"This is a\nmultiline\nstring literal\"#;\n    println!(\"{}\", message);\n}";

    let file = create_test_file(&temp_path, "multiline_string.rs", content).await;

    let mut units = vec![create_translation_unit("1", "This is a", 2, 20, 29)];
    units[0].set_translated("这是一个");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("r#\""),
        "Raw string prefix should be preserved"
    );
    assert!(
        written_content.contains("\"#;"),
        "Raw string suffix should be preserved"
    );
    assert!(
        written_content.contains("multiline"),
        "Other lines should be preserved"
    );
}

#[tokio::test]
async fn test_mixed_comments_and_code() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"// Top comment
fn main() {
    // Function comment
    let x = 5; // Inline comment
    /* Block comment
     * with multiple lines
     */
    let y = 10;
}
// Bottom comment"#;

    let file = create_test_file(&temp_path, "mixed_comments.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::BlockMulti,
        base_indent: "    ".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![
        create_translation_unit("1", "Top comment", 1, 1, 13),
        create_translation_unit("2", "Function comment", 3, 5, 22),
        create_translation_unit("3", "Inline comment", 4, 16, 32),
        create_translation_unit_with_format(
            "4",
            "/* Block comment\n     * with multiple lines\n     */",
            5,
            5,
            52,
            format_info,
        ),
        create_translation_unit("5", "Bottom comment", 9, 1, 15),
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

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn main()"),
        "Function should be preserved"
    );
    assert!(
        written_content.contains("let x = 5;"),
        "Variable x should be preserved"
    );
    assert!(
        written_content.contains("let y = 10;"),
        "Variable y should be preserved"
    );
    assert!(
        written_content.contains("顶部注释"),
        "Top comment should be translated"
    );
    assert!(
        written_content.contains("函数注释"),
        "Function comment should be translated"
    );
    assert!(
        written_content.contains("行内注释"),
        "Inline comment should be translated"
    );
    assert!(
        written_content.contains("底部注释"),
        "Bottom comment should be translated"
    );
}

#[tokio::test]
async fn test_indented_block_comment_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"fn main() {
        if true {
            /* Indented block comment
             * with multiple levels
             * of indentation
             */
            let x = 5;
        }
    }"#;

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

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn main()"),
        "Function should be preserved"
    );
    assert!(
        written_content.contains("if true"),
        "If statement should be preserved"
    );
    assert!(
        written_content.contains("let x = 5;"),
        "Variable should be preserved"
    );
    assert!(
        written_content.contains("/*"),
        "Comment start should be preserved"
    );
    assert!(
        written_content.contains("*/"),
        "Comment end should be preserved"
    );
    assert!(
        written_content.contains("缩进块注释"),
        "Comment should be translated"
    );
}

#[tokio::test]
async fn test_doc_comment_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"/// This is a doc comment
/// with multiple lines
fn example() -> i32 {
    42
}

/// Another doc comment
/// for another function
fn another() -> i32 {
    100
}"#;

    let file = create_test_file(&temp_path, "doc_comment.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocOuter,
        base_indent: "".to_string(),
        line_prefix: Some("/// ".to_string()),
        ends_with_newline: false,
        is_multiline: false,
    };

    let mut units = vec![
        create_translation_unit_with_format(
            "1",
            "/// This is a doc comment",
            1,
            1,
            24,
            format_info.clone(),
        ),
        create_translation_unit_with_format(
            "2",
            "/// with multiple lines",
            2,
            1,
            23,
            format_info.clone(),
        ),
        create_translation_unit_with_format(
            "3",
            "/// Another doc comment",
            8,
            1,
            24,
            format_info.clone(),
        ),
        create_translation_unit_with_format("4", "/// for another function", 9, 1, 27, format_info),
    ];

    units[0].set_translated("/// 这是一个文档注释");
    units[1].set_translated("/// 多行");
    units[2].set_translated("/// 另一个文档注释");
    units[3].set_translated("/// 用于另一个函数");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn example()"),
        "Function example should be preserved"
    );
    assert!(
        written_content.contains("fn another()"),
        "Function another should be preserved"
    );
    assert!(
        written_content.contains("-> i32"),
        "Return types should be preserved"
    );
    assert!(
        written_content.contains("这是一个文档注释"),
        "Doc comment should be translated"
    );
    assert!(
        written_content.contains("多行"),
        "Doc comment line 2 should be translated"
    );
}

#[tokio::test]
async fn test_string_with_special_characters() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"fn main() {
    let path = "C:\\Users\\test\\file.txt";
    let quote = "He said \"Hello\"";
    let escape = "Line 1\nLine 2\tTabbed";
}"#;

    let file = create_test_file(&temp_path, "special_chars.rs", content).await;

    let mut units = vec![
        create_translation_unit("1", "He said \"Hello\"", 3, 17, 32),
        create_translation_unit("2", "Line 1", 4, 17, 23),
    ];

    units[0].set_translated("他说\"你好\"");
    units[1].set_translated("第一行");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("let path ="),
        "Path variable should be preserved"
    );
    assert!(
        written_content.contains("C:\\Users\\test\\file.txt"),
        "Path string should be preserved"
    );
    assert!(
        written_content.contains("let quote ="),
        "Quote variable should be preserved"
    );
    assert!(
        written_content.contains("他说\"你好\""),
        "Quote string should be translated"
    );
    assert!(
        written_content.contains("第一行"),
        "Line 1 should be translated"
    );
    assert!(
        written_content.contains("\\n"),
        "Escape sequence should be preserved"
    );
    assert!(written_content.contains("\\t"), "Tab should be preserved");
}

#[tokio::test]
async fn test_complex_code_structure_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"// Module comment
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
}"#;

    let file = create_test_file(&temp_path, "complex_structure.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocOuter,
        base_indent: "        ".to_string(),
        line_prefix: Some("/// ".to_string()),
        ends_with_newline: false,
        is_multiline: false,
    };

    let mut units = vec![
        create_translation_unit("1", "Module comment", 1, 1, 15),
        create_translation_unit("2", "Nested module comment", 3, 5, 27),
        create_translation_unit_with_format(
            "3",
            "/// Field doc comment",
            6,
            9,
            28,
            format_info.clone(),
        ),
        create_translation_unit("4", "Method comment", 10, 9, 24),
    ];

    units[0].set_translated("模块注释");
    units[1].set_translated("嵌套模块注释");
    units[2].set_translated("/// 字段文档注释");
    units[3].set_translated("方法注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("mod example"),
        "Module should be preserved"
    );
    assert!(
        written_content.contains("pub struct Example"),
        "Struct should be preserved"
    );
    assert!(
        written_content.contains("pub field: i32"),
        "Field should be preserved"
    );
    assert!(
        written_content.contains("impl Example"),
        "Impl block should be preserved"
    );
    assert!(
        written_content.contains("pub fn new()"),
        "Method should be preserved"
    );
    assert!(
        written_content.contains("Self { field: 0 }"),
        "Method body should be preserved"
    );
    assert!(
        written_content.contains("模块注释"),
        "Module comment should be translated"
    );
    assert!(
        written_content.contains("嵌套模块注释"),
        "Nested comment should be translated"
    );
}

#[tokio::test]
async fn test_multiline_doc_block_comment() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"/**
 * This is a doc block comment
 * with multiple lines
 * and detailed information
 */
fn example() -> i32 {
    42
}"#;

    let file = create_test_file(&temp_path, "doc_block.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::DocBlock,
        base_indent: "".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![create_translation_unit_with_format(
        "1",
        "/**\n * This is a doc block comment\n * with multiple lines\n * and detailed information\n */",
        1,
        1,
        76,
        format_info,
    )];
    units[0].set_translated("这是一个文档块注释\n多行\n详细信息");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn example()"),
        "Function should be preserved"
    );
    assert!(
        written_content.contains("/**"),
        "Doc block start should be preserved"
    );
    assert!(
        written_content.contains("*/"),
        "Doc block end should be preserved"
    );
    assert!(
        written_content.contains("这是一个文档块注释"),
        "Comment should be translated"
    );
}

#[tokio::test]
async fn test_empty_lines_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = r#"// First comment

fn main() {

    // Nested comment
    let x = 5;

    println!("{}", x);

}

// Last comment"#;

    let file = create_test_file(&temp_path, "empty_lines.rs", content).await;

    let mut units = vec![
        create_translation_unit("1", "First comment", 1, 1, 15),
        create_translation_unit("2", "Nested comment", 5, 5, 21),
        create_translation_unit("3", "Last comment", 11, 1, 14),
    ];

    units[0].set_translated("第一个注释");
    units[1].set_translated("嵌套注释");
    units[2].set_translated("最后一个注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;

    assert!(
        written_content.contains("fn main()"),
        "Function should be preserved"
    );
    assert!(
        written_content.contains("第一个注释"),
        "First comment should be translated"
    );
    assert!(
        written_content.contains("嵌套注释"),
        "Nested comment should be translated"
    );
    assert!(
        written_content.contains("最后一个注释"),
        "Last comment should be translated"
    );

    let lines: Vec<&str> = written_content.lines().collect();
    assert_eq!(lines.len(), 12, "Empty lines should be preserved");
}
