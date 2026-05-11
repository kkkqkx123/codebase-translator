//! Debug test to understand why verify finds 0 matches for template-renderer.ts

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::project::{FilterConfig, ProjectConfig, TranslateConfig};
use crate::core::models::File;
use crate::parser::coordinator::ParserCoordinator;
use crate::parser::filtering::composite::from_project_config;
use crate::parser::filtering::Filter;
use crate::parser::ParserConfig;

fn create_test_file(content: &str, path: &str) -> File {
    File::new(PathBuf::from(path), content.as_bytes().to_vec(), "utf-8")
}

#[test]
fn test_verify_filter_on_doc_comment_segments() {
    // Create default project config (source_langs=["auto"], target_lang="en")
    let project_config = ProjectConfig::default();
    
    // Create filter from project config
    let filter = Arc::new(
        from_project_config(&project_config.filter, &project_config.translate)
            .expect("Failed to create filter"),
    );

    // Test segments from template-renderer.ts doc comments
    let test_segments = vec![
        "TemplateRenderer - 模板渲染器\n提供模板变量替换功能，支持嵌套路径解析",
        "功能：\n- 支持 {{variable}} 占位符替换\n- 支持嵌套路径解析（如 user.name）\n- 支持数组索引访问（如 items[0].name）\n- 支持条件渲染 {{#if variable}}...{{/if}}\n- 支持循环渲染 {{#each array}}...{{/each}}\n- 提供安全的变量值获取",
        "渲染模板\n替换模板中的 {{variable}} 占位符，支持条件和循环",
        "@param template 模板字符串，包含 {{variable}} 占位符\n@param variables 变量对象\n@returns 渲染后的字符串",
        "// 结果: 'Hello, Alice! Today is 2024-01-01.'",
        "@example 条件渲染",
        "@example 循环渲染",
    ];

    println!("\n=== Filter Test Results ===");
    println!("Source langs: {:?}", project_config.translate.source_langs);
    println!("Target lang: {}", project_config.translate.target_lang);
    println!();

    for (i, segment) in test_segments.iter().enumerate() {
        let result = filter.should_translate(segment);
        println!("Segment {}: {}", i + 1, if result { "PASS" } else { "FILTERED" });
        println!("  Content: {:.80}...", segment.chars().take(80).collect::<String>());
        
        if !result {
            // Test individual filter to find which one is filtering
            println!("  -> This segment was filtered out!");
        }
        println!();
    }
}

#[test]
fn test_verify_filter_on_error_message_strings() {
    let project_config = ProjectConfig::default();
    
    let filter = Arc::new(
        from_project_config(&project_config.filter, &project_config.translate)
            .expect("Failed to create filter"),
    );

    // Test error message strings from template-renderer.ts
    let test_strings = vec![
        "不支持的循环特殊变量 '${trimmedName}'。支持的变量: ${Array.from(SUPPORTED_LOOP_SPECIAL_VARS).join(\", \")}",
        "不支持的循环特殊变量 '${varName}'。支持的变量: ${Array.from(SUPPORTED_LOOP_SPECIAL_VARS).join(\", \")}",
        "The loop special variable '${trimmedName}' can only be used inside a {{#each}} loop",
    ];

    println!("\n=== Error Message Filter Test Results ===");

    for (i, s) in test_strings.iter().enumerate() {
        let result = filter.should_translate(s);
        println!("String {}: {}", i + 1, if result { "PASS" } else { "FILTERED" });
        println!("  Content: {:.80}...", s.chars().take(80).collect::<String>());
        println!();
    }
}

#[test]
fn test_parser_coordinator_on_template_renderer() {
    // Read the actual file
    let file_path = "tests/temp/template-renderer.ts";
    let content = std::fs::read_to_string(file_path).expect("Failed to read file");
    
    let file = create_test_file(&content, file_path);
    
    // Create parser coordinator with default project config
    let project_config = ProjectConfig::default();
    let parser = ParserCoordinator::from_project_config(
        ParserConfig::default(),
        &project_config,
    ).expect("Failed to create parser coordinator");

    // Parse the file
    let units = parser.parse_file(&file).expect("Failed to parse file");
    
    println!("\n=== Parser Coordinator Results ===");
    println!("Total units extracted: {}", units.len());
    
    let units_to_translate: Vec<_> = units.iter().filter(|u| u.should_translate).collect();
    println!("Units with should_translate=true: {}", units_to_translate.len());
    
    let units_filtered: Vec<_> = units.iter().filter(|u| !u.should_translate).collect();
    println!("Units with should_translate=false: {}", units_filtered.len());
    
    if !units_to_translate.is_empty() {
        println!("\nUnits that would be translated:");
        for (i, unit) in units_to_translate.iter().enumerate() {
            println!("  {}. [{}] {:.60}...", 
                i + 1, 
                unit.node_type, 
                unit.content.chars().take(60).collect::<String>());
        }
    }
    
    if !units_filtered.is_empty() {
        println!("\nUnits that were filtered out:");
        for (i, unit) in units_filtered.iter().take(10).enumerate() {
            println!("  {}. [{}] {:.60}...", 
                i + 1, 
                unit.node_type, 
                unit.content.chars().take(60).collect::<String>());
        }
    }
}
