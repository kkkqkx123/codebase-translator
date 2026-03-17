//! ConcurrentWriter integration tests

use codebase_translate::core::models::{CommentStyle, File, FormatInfo, Position};
use codebase_translate::writer::{ConcurrentWriter, WriterConfig};

use crate::writer_common::*;

#[tokio::test]
async fn test_concurrent_writer_basic() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let mut files = Vec::new();

    for i in 0..3 {
        let content = format!("File {} content\nLine 2", i);
        let file: File = create_test_file(&temp_path, &format!("test_{}.txt", i), &content).await;
        files.push(file);
    }

    let mut file_units = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let mut units = vec![create_translation_unit(
            &format!("unit_{}", i),
            "File",
            1,
            1,
            5,
        )];
        units[0].set_translated(format!("文件{}", i));
        file_units.push((file.clone(), units));
    }

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 2);

    let results = writer.write_files(file_units).await;

    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(
            result.success,
            "All writes should succeed: {:?}",
            result.error
        );
        assert_eq!(result.units_written, 1);
    }

    for file in &files {
        let content: String = read_file_content(&file.path).await;
        assert!(
            content.contains("文件"),
            "File should contain translated text"
        );
    }
}

#[tokio::test]
async fn test_concurrent_writer_with_different_concurrency() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let mut files = Vec::new();
    let file_count = 5;

    for i in 0..file_count {
        let content = format!("Content {}\nLine 2", i);
        let file: File =
            create_test_file(&temp_path, &format!("concurrent_{}.txt", i), &content).await;
        files.push(file);
    }

    let mut file_units = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let mut units = vec![create_translation_unit(
            &format!("unit_{}", i),
            "Content",
            1,
            1,
            8,
        )];
        units[0].set_translated(format!("内容{}", i));
        file_units.push((file.clone(), units));
    }

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 3);

    let results = writer.write_files(file_units).await;

    assert_eq!(results.len(), file_count);
    for result in &results {
        assert!(result.success, "All writes should succeed");
    }
}

#[tokio::test]
async fn test_concurrent_writer_with_backup() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Original content\nLine 2";
    let file = create_test_file(&temp_path, "test_concurrent_backup.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Original", 1, 1, 9)];
    units[0].set_translated("修改后的");

    let config = WriterConfig {
        backup: true,
        backup_dir: None,
        ..Default::default()
    };
    let writer = ConcurrentWriter::new(config, 1);

    let results = writer.write_files(vec![(file.clone(), units)]).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);

    let written_content: String = read_file_content(&file.path).await;
    assert!(written_content.contains("修改后的"));

    let mut backup_files = tokio::fs::read_dir(temp_path)
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
async fn test_concurrent_writer_preview_mode() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Original content\nLine 2";
    let file = create_test_file(&temp_path, "test_concurrent_preview.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Original", 1, 1, 9)];
    units[0].set_translated("修改后的");

    let config = WriterConfig {
        preview_only: true,
        ..Default::default()
    };
    let writer = ConcurrentWriter::new(config, 1);

    let results = writer.write_files(vec![(file.clone(), units)]).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);

    let written_content: String = read_file_content(&file.path).await;
    assert_eq!(
        written_content, content,
        "File should not be modified in preview mode"
    );
}

#[tokio::test]
async fn test_concurrent_writer_mixed_success_failure() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content1 = "Valid content\nLine 2";
    let file1: File = create_test_file(&temp_path, "test_valid.txt", content1).await;

    let content2 = "Another valid content\nLine 2";
    let file2: File = create_test_file(&temp_path, "test_valid2.txt", content2).await;

    let mut units1 = vec![create_translation_unit("1", "Valid", 1, 1, 6)];
    units1[0].set_translated("有效");

    let units2 = vec![create_translation_unit("2", "Another", 1, 1, 8)];

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 2);

    let results = writer
        .write_files(vec![(file1.clone(), units1), (file2.clone(), units2)])
        .await;

    assert_eq!(results.len(), 2);

    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();

    assert!(success_count >= 1, "At least one file should succeed");
    assert!(failure_count >= 1, "At least one file should fail");
}

#[tokio::test]
async fn test_concurrent_writer_streaming() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let (sender, receiver) = tokio::sync::mpsc::channel(10);

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 2);

    tokio::spawn(async move {
        for i in 0..3 {
            let content = format!("Stream content {}\nLine 2", i);
            let file: File =
                create_test_file(&temp_path, &format!("stream_{}.txt", i), &content).await;

            let mut units = vec![create_translation_unit(
                &format!("unit_{}", i),
                "Stream",
                1,
                1,
                7,
            )];
            units[0].set_translated(format!("流{}", i));

            sender
                .send((file, units))
                .await
                .expect("Failed to send to channel");
        }
        drop(sender);
    });

    let mut results = Vec::new();
    let mut result_receiver = writer.write_files_streaming(receiver).await;

    while let Some(result) = result_receiver.recv().await {
        results.push(result);
    }

    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(result.success, "All streaming writes should succeed");
    }
}

#[tokio::test]
async fn test_concurrent_writer_with_format_info() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "    // This is a comment\nint x = 5;";
    let file: File = create_test_file(&temp_path, "test_format.rs", content).await;

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
        8,
        25,
        format_info,
    )];
    units[0].set_translated("这是一个注释");

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 1);

    let results = writer
        .write_files(vec![(file.clone(), units.clone())])
        .await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);

    let written_content: String = read_file_content(&file.path).await;

    // Write output for comparison
    crate::writer_common::write_test_result(
        "test_concurrent_writer_with_format_info",
        content,
        &written_content,
        &units,
    );

    assert!(written_content.contains("    // 这是一个注释"));
}

#[tokio::test]
async fn test_concurrent_writer_max_concurrent() {
    let config = WriterConfig::default();
    let mut writer = ConcurrentWriter::new(config, 5);

    assert_eq!(writer.max_concurrent(), 5);

    writer.set_max_concurrent(10);
    assert_eq!(writer.max_concurrent(), 10);

    writer.set_max_concurrent(0);
    assert_eq!(writer.max_concurrent(), 1, "Should be at least 1");
}

#[tokio::test]
async fn test_concurrent_writer_large_batch() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let file_count = 20;
    let mut files = Vec::new();
    let mut file_units = Vec::new();

    for i in 0..file_count {
        let content = format!("Batch content {}\nLine 2", i);
        let file: File = create_test_file(&temp_path, &format!("batch_{}.txt", i), &content).await;
        files.push(file.clone());

        let mut units = vec![create_translation_unit(
            &format!("unit_{}", i),
            "Batch",
            1,
            1,
            6,
        )];
        units[0].set_translated(format!("批次{}", i));
        file_units.push((file, units));
    }

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 5);

    let results = writer.write_files(file_units).await;

    assert_eq!(results.len(), file_count);

    let success_count = results.iter().filter(|r| r.success).count();
    assert_eq!(
        success_count, file_count,
        "All files should be written successfully"
    );
}

#[tokio::test]
async fn test_concurrent_writer_empty_batch() {
    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 2);

    let results = writer.write_files(Vec::new()).await;

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_concurrent_writer_with_crlf() {
    let temp_dir = create_temp_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let content = "Line 1\r\nLine 2\r\nLine 3";
    let file: File = create_test_file(&temp_path, "test_crlf.txt", content).await;

    let mut units = vec![create_translation_unit("1", "Line 1", 1, 1, 7)];
    units[0].set_translated("第一行");

    let config = WriterConfig::default();
    let writer = ConcurrentWriter::new(config, 1);

    let results = writer.write_files(vec![(file.clone(), units)]).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);

    let written_content: String = read_file_content(&file.path).await;
    assert!(
        written_content.contains("\r\n"),
        "CRLF line endings should be preserved"
    );
    assert!(written_content.contains("第一行"));
}
