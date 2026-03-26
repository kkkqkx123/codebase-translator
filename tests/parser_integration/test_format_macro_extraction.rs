//! Test format! macro string extraction with Chinese content

use std::fs;
use std::path::PathBuf;

use codebase_translate::core::models::File;
use codebase_translate::parser::coordinator::ParserCoordinator;
use codebase_translate::parser::ParserConfig;

#[test]
fn test_format_macro_chinese_extraction() {
    let config = ParserConfig {
        extract_strings: true,
        ..Default::default()
    };

    let coordinator =
        ParserCoordinator::with_unified_config(config).expect("Failed to create coordinator");

    // Test Rust code with format! macro containing Chinese
    let content = r#"
fn main() {
    let name = "test_space";
    let message = format!("图空间 '{}' 不存在", name);
    println!("{}", message);

    let error = format!("配置文件 '{}' 未找到", "config.toml");
    eprintln!("{}", error);

    let info = format!("成功连接到数据库 '{}'", "postgres");
    log::info!("{}", info);

    // Test with multiple placeholders
    let result = format!("用户 '{}' 在 {} 时访问了资源", "admin", "2024-01-01");
    println!("{}", result);

    // Test format_args!
    let msg = format_args!("处理文件 '{}' 大小为 {} 字节", "data.txt", 1024);
    println!("{}", msg);

    // Test print! macros
    print!("正在处理: {}", name);
    println!("完成: {}", name);
    eprint!("错误: {}", name);
    eprintln!("警告: {}", name);
}
"#;

    let file = File::new(
        PathBuf::from("test_format_chinese.rs"),
        content.as_bytes().to_vec(),
        "utf-8",
    );

    let units = coordinator.parse_file(&file).expect("Parsing failed");

    // Write output for inspection
    fs::create_dir_all("tests/parser_integration/output").expect("Failed to create output dir");
    let output_path =
        PathBuf::from("tests/parser_integration/output/format_macro_chinese_extraction.txt");

    let mut output = String::new();
    output.push_str(&format!("Extracted {} translation units\n", units.len()));
    output.push_str("==================================================\n\n");

    for (i, unit) in units.iter().enumerate() {
        output.push_str(&format!("--- Unit {} ---\n", i + 1));
        output.push_str(&format!("ID: {}\n", unit.id));
        output.push_str(&format!("Type: {:?}\n", unit.node_type));
        output.push_str(&format!(
            "Position: Line {}, Column {} (Offset: {})\n",
            unit.start_pos.line, unit.start_pos.column, unit.start_pos.offset
        ));
        output.push_str(&format!(
            "End Position: Line {}, Column {} (Offset: {})\n",
            unit.end_pos.line, unit.end_pos.column, unit.end_pos.offset
        ));
        output.push_str(&format!("Content:\n{}\n", unit.content));
        if let Some(raw) = &unit.raw_match {
            output.push_str(&format!("Raw Match:\n{}\n", raw));
        }
        output.push_str(&format!("Should Translate: {}\n", unit.should_translate));
        output.push('\n');
    }

    fs::write(&output_path, output).expect("Failed to write output file");
    println!("Output written to: {}", output_path.display());

    // Print summary
    println!("\n=== Summary ===");
    println!("Total units: {}", units.len());

    // Count units by type
    let format_strings = units
        .iter()
        .filter(|u| {
            matches!(
                u.node_type,
                codebase_translate::core::models::NodeType::FormatString
            )
        })
        .count();

    let log_messages = units
        .iter()
        .filter(|u| {
            matches!(
                u.node_type,
                codebase_translate::core::models::NodeType::LogMessage
            )
        })
        .count();

    println!("Format strings: {}", format_strings);
    println!("Log messages: {}", log_messages);

    // Check if we extracted the specific Chinese strings
    let has_graph_space = units.iter().any(|u| u.content.contains("图空间"));
    let has_config_file = units.iter().any(|u| u.content.contains("配置文件"));
    let has_database = units.iter().any(|u| u.content.contains("数据库"));
    let has_user_access = units.iter().any(|u| u.content.contains("用户"));
    let has_file_size = units.iter().any(|u| u.content.contains("处理文件"));

    println!("\n=== String Extraction Verification ===");
    println!("Contains '图空间': {}", has_graph_space);
    println!("Contains '配置文件': {}", has_config_file);
    println!("Contains '数据库': {}", has_database);
    println!("Contains '用户': {}", has_user_access);
    println!("Contains '处理文件': {}", has_file_size);

    // Print all extracted strings for manual inspection
    println!("\n=== All Extracted Strings ===");
    for (i, unit) in units.iter().enumerate() {
        println!("{}: {:?} - '{}'", i + 1, unit.node_type, unit.content);
    }

    // Assertions to verify extraction
    assert!(
        has_graph_space,
        "Should extract '图空间' from format! macro"
    );
    assert!(
        has_config_file,
        "Should extract '配置文件' from format! macro"
    );
    assert!(has_database, "Should extract '数据库' from format! macro");
    assert!(has_user_access, "Should extract '用户' from format! macro");
    assert!(
        has_file_size,
        "Should extract '处理文件' from format_args! macro"
    );

    // We should have at least 5 format strings extracted
    assert!(
        format_strings >= 5,
        "Should extract at least 5 format strings, got {}",
        format_strings
    );
}
