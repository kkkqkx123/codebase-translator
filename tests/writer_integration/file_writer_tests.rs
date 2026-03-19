//! FileWriter integration tests

use codebase_translate::core::models::{CommentStyle, FormatInfo, NodeType, Position};
use codebase_translate::writer::{FileWriter, WriterConfig};

use super::common::*;

#[tokio::test]
async fn test_file_writer_basic_write() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Hello world\nThis is a test";
    let file = create_test_file(&temp_path, "test_basic.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];
    units[0].set_translated("你好");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok(), "Write should succeed");

    let written_content = read_file_content(&file.path).await;
    assert!(written_content.contains("你好"));
    assert!(written_content.contains("world"));

    // Write output for inspection
    write_output("test_file_writer_basic_write", &written_content);
}

#[tokio::test]
async fn test_file_writer_with_backup() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Original content\nLine 2";
    let file = create_test_file(&temp_path, "test_backup.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Original", 1, 1, 9)];
    units[0].set_translated("修改后的");

    let config = WriterConfig {
        backup: true,
        backup_dir: None,
        ..Default::default()
    };
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert!(written_content.contains("修改后的"));

    let mut backup_files = tokio::fs::read_dir(&temp_path)
        .await
        .expect("Failed to read dir");
    let mut backup_count = 0;
    while let Ok(Some(entry)) = backup_files.next_entry().await {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.contains(".bak.") {
            backup_count += 1;
        }
    }
    assert!(backup_count >= 1, "Should have at least one backup file");
}

#[tokio::test]
async fn test_file_writer_preview_mode() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Hello world\nThis is a test";
    let file = create_test_file(&temp_path, "test_preview.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];
    units[0].set_translated("你好");

    let config = WriterConfig {
        preview_only: true,
        ..Default::default()
    };
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert_eq!(
        written_content, content,
        "File should not be modified in preview mode"
    );
}

#[tokio::test]
async fn test_file_writer_no_changes() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Hello world\nThis is a test";
    let file = create_test_file(&temp_path, "test_no_changes.txt", content).await;

    let units = vec![create_translation_unit("1", "Hello", 1, 1, 6)];

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert_eq!(
        written_content, content,
        "File should not be modified when no translations"
    );
}

#[tokio::test]
async fn test_file_writer_multiple_units() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "First line\nSecond line\nThird line";
    let file = create_test_file(&temp_path, "test_multiple.txt", content).await;

    let mut units = vec![
        create_translation_unit("1", "First", 1, 1, 6),
        create_translation_unit("2", "Second", 2, 1, 7),
        create_translation_unit("3", "Third", 3, 1, 6),
    ];
    units[0].set_translated("第一");
    units[1].set_translated("第二");
    units[2].set_translated("第三");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert!(written_content.contains("第一"));
    assert!(written_content.contains("第二"));
    assert!(written_content.contains("第三"));
}

#[tokio::test]
async fn test_file_writer_with_line_comment_format() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "    // This is a comment\nint x = 5;";
    let file = create_test_file(&temp_path, "test_line_comment.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::Line,
        base_indent: "    ".to_string(),
        line_prefix: Some("// ".to_string()),
        ends_with_newline: false,
        is_multiline: false,
    };

    let mut units = vec![create_translation_unit_with_format(
        "1",
        "This is a comment",
        1,
        8,  // Start after "    // " (1-indexed)
        25, // End of "This is a comment" (1-indexed, exclusive)
        format_info,
    )];
    units[0].set_translated("这是一个注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert!(written_content.contains("    // 这是一个注释"));

    // Write output for inspection
    write_output("test_file_writer_with_line_comment_format", &written_content);
}

#[tokio::test]
async fn test_file_writer_with_block_comment_format() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "/* This is a comment */\nint x = 5;";
    let file = create_test_file(&temp_path, "test_block_comment.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::BlockSingle,
        base_indent: "".to_string(),
        line_prefix: None,
        ends_with_newline: false,
        is_multiline: false,
    };

    let mut units = vec![create_translation_unit_with_format(
        "1",
        "/* This is a comment */",
        1,
        1,
        22,
        format_info,
    )];
    units[0].set_translated("这是一个注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert!(written_content.contains("/* 这是一个注释 */"));
}

#[tokio::test]
async fn test_file_writer_with_multiline_block_comment() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "/*\n * Line 1\n * Line 2\n */\nint x = 5;";
    let file = create_test_file(&temp_path, "test_multiline_comment.rs", content).await;

    let format_info = FormatInfo {
        style: CommentStyle::BlockMulti,
        base_indent: "".to_string(),
        line_prefix: Some(" * ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![create_translation_unit_with_format(
        "1",
        "/*\n * Line 1\n * Line 2\n */",
        1,
        1,
        22,
        format_info,
    )];
    units[0].set_translated("第一行\n第二行");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert!(written_content.contains("/*\n * 第一行\n * 第二行\n */"));
}

#[tokio::test]
async fn test_file_writer_crlf_preservation() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Line 1\r\nLine 2\r\nLine 3";
    let file = create_test_file(&temp_path, "test_crlf.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Line 1", 1, 1, 7)];
    units[0].set_translated("第一行");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    assert!(
        written_content.contains("\r\n"),
        "CRLF line endings should be preserved"
    );
    assert!(written_content.contains("第一行"));
}

#[tokio::test]
async fn test_file_writer_config_validation() {
    let config = WriterConfig::default();
    assert!(config.validate().is_ok(), "Default config should be valid");

    let abs_path = std::env::current_dir().expect("Should get current dir");
    let config_with_backup = WriterConfig {
        backup_dir: Some(abs_path),
        ..Default::default()
    };
    assert!(config_with_backup.validate().is_ok());
}

#[tokio::test]
async fn test_file_writer_set_preview_mode() {
    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    assert_eq!(writer.config().await.unwrap().preview_only, false);

    writer.set_preview_mode(true).await;
    assert_eq!(writer.config().await.unwrap().preview_only, true);

    writer.set_preview_mode(false).await;
    assert_eq!(writer.config().await.unwrap().preview_only, false);
}

#[tokio::test]
async fn test_file_writer_set_backup_mode() {
    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    assert_eq!(writer.config().await.unwrap().backup, true);

    writer.set_backup_mode(false).await;
    assert_eq!(writer.config().await.unwrap().backup, false);

    writer.set_backup_mode(true).await;
    assert_eq!(writer.config().await.unwrap().backup, true);
}

#[tokio::test]
async fn test_file_writer_with_rust_fixture() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = load_fixture("simple_rust.rs");
    let file = create_test_file(&temp_path, "simple_rust.rs", &content).await;

    let format_info = FormatInfo {
        style: CommentStyle::Line,
        base_indent: "".to_string(),
        line_prefix: Some("// ".to_string()),
        ends_with_newline: true,
        is_multiline: false,
    };

    let mut units = vec![
        create_translation_unit_with_format("1", "Test file with simple comments", 1, 3, 30, format_info.clone()),
        create_translation_unit_with_format("2", "This is a line comment", 2, 3, 25, format_info.clone()),
        create_translation_unit_with_format("3", "Another comment", 4, 6, 22, format_info),
    ];
    units[0].set_translated("测试文件，包含简单注释");
    units[1].set_translated("这是一个行注释");
    units[2].set_translated("另一个注释");

    let config = WriterConfig::default();
    let writer = FileWriter::new(config);

    let result = writer.write(&file, &units).await;
    assert!(result.is_ok());

    let written_content = read_file_content(&file.path).await;
    
    // Print for debugging
    println!("Original content:\n{}", content);
    println!("Written content:\n{}", written_content);
    
    assert!(written_content.contains("// 测试文件，包含简单注释"));
    assert!(written_content.contains("// 这是一个行注释"));
    assert!(written_content.contains("// 另一个注释"));

    // Write output for inspection
    write_output("test_file_writer_with_rust_fixture", &written_content);
}
