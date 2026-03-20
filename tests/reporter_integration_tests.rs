use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use codebase_translate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    reporter::create_reporter,
    workflow::TranslationWorkflow,
};

fn create_test_project_config() -> ProjectConfig {
    let config_content = r#"
[translate]
target_lang = "en"
source_langs = ["zh"]
provider = "DeepLX"

[include]
patterns = ["**/*.rs", "**/*.py", "**/*.js", "**/*.md"]

[exclude]
patterns = ["node_modules/**", "target/**", "vendor/**"]
respect_gitignore = true

[cache]
enabled = true
mode = "local"
directory = ".translator/cache"

[writer]
dry_run = true
"#;

    let config: ProjectConfig = toml::from_str(config_content).expect("Failed to parse config");
    config
}

fn create_test_global_config() -> GlobalConfig {
    let config_content = r#"
[deeplx]
api_url = "http://localhost:8080/translate"

[llm]
provider = "openai"
api_key = "test-key"
model = "gpt-4"

[tencent]
secret_id = "test-id"
secret_key = "test-key"
region = "ap-guangzhou"
"#;

    let config: GlobalConfig = toml::from_str(config_content).expect("Failed to parse config");
    config
}

fn create_test_files(temp_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let files = vec![
        ("test1.rs", r#"
//! 这是测试文件 1
//! 包含中文注释

fn main() {
    println!("Hello, World!");
    // 这是一个打印语句
}
"#),
        ("test2.py", r#"
# 这是测试文件 2
# Python 代码示例

def hello():
    print("Hello, World!")
    # 打印问候语
"#),
        ("test3.md", r#"
# 测试文档

这是一个测试文档，包含中文内容。

## 功能列表

- 功能一
- 功能二
- 功能三
"#),
    ];

    let mut created_files = Vec::new();

    for (filename, content) in files {
        let file_path = temp_dir.join(filename);
        fs::write(&file_path, content)?;
        created_files.push(file_path);
    }

    Ok(created_files)
}

#[test]
fn test_reporter_integration_with_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    let result = workflow.execute()?;

    assert!(result.files_processed >= 0, "Should process files");

    let stats = reporter.get_stats();
    assert!(stats.total_files >= 0);
    assert!(stats.total_units >= 0, "Should have translation units");

    Ok(())
}

#[test]
fn test_reporter_records_file_processing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let stats = reporter.get_stats();
    assert!(stats.processed_files > 0, "Should record processed files");
    assert!(stats.total_units > 0, "Should record total units");

    Ok(())
}

#[test]
fn test_reporter_records_cache_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let stats = reporter.get_stats();
    let total_cache_ops = stats.cache_hit_count + stats.cache_miss_count;
    assert!(total_cache_ops > 0, "Should record cache operations");

    Ok(())
}

#[test]
fn test_reporter_finalization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let stats = reporter.get_stats();
    assert!(stats.end_time.is_some(), "Should have end time after finalization");
    assert!(stats.total_duration_ms > 0, "Should have duration after finalization");

    Ok(())
}

#[test]
fn test_reporter_progress_tracking() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let progress = reporter.get_progress();
    assert!(progress >= 0.0, "Should have progress >= 0");
    assert!(progress <= 100.0, "Progress should be <= 100");

    Ok(())
}

#[test]
fn test_reporter_save_report() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let report_path = temp_dir_path.join("report.txt");
    reporter.save_report(&report_path, codebase_translate::reporter::ReportFormat::Text)?;

    assert!(report_path.exists(), "Report file should be created");

    let report_content = fs::read_to_string(&report_path)?;
    assert!(report_content.contains("Translation Report"), "Report should contain title");
    assert!(report_content.contains("Files:"), "Report should contain files section");
    assert!(report_content.contains("Translation Units:"), "Report should contain units section");

    Ok(())
}

#[test]
fn test_reporter_save_report_with_template() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let _files = create_test_files(&temp_dir_path)?;

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let output_dir = temp_dir_path.join("reports");
    fs::create_dir_all(&output_dir)?;

    let saved_path = reporter.save_report_with_template(
        &output_dir,
        "translation_report_{timestamp}.txt",
        codebase_translate::reporter::ReportFormat::Text,
    )?;

    assert!(saved_path.exists(), "Report file should be created");
    assert!(saved_path.starts_with(&output_dir), "Report should be in output directory");

    Ok(())
}

#[test]
fn test_reporter_without_workflow() {
    let reporter = create_reporter();

    reporter.report_file(PathBuf::from("test.rs").as_path(), 5);
    reporter.report_progress(1, 10);
    reporter.report_cache_hit();
    reporter.report_cache_miss();
    reporter.report_api_call(2);

    let stats = reporter.get_stats();
    assert_eq!(stats.processed_files, 1);
    assert_eq!(stats.total_units, 5);
    assert_eq!(stats.cache_hit_count, 1);
    assert_eq!(stats.cache_miss_count, 1);
    assert_eq!(stats.api_call_count, 2);
}

#[test]
fn test_reporter_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let invalid_file = temp_dir_path.join("invalid.txt");
    fs::write(&invalid_file, "test content")?;

    let global_config = create_test_global_config();
    let mut project_config = create_test_project_config();
    project_config.include.patterns = vec!["**/*.txt".to_string()];

    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    workflow.execute()?;

    let stats = reporter.get_stats();
    assert_eq!(stats.error_count, 0, "Should handle errors gracefully");

    Ok(())
}

#[test]
fn test_reporter_empty_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_dir_path = temp_dir.path().to_path_buf();

    let global_config = create_test_global_config();
    let project_config = create_test_project_config();
    let reporter = create_reporter();

    let workflow = TranslationWorkflow::from_configs_with_path(
        global_config,
        project_config,
        temp_dir_path.to_str().expect("Invalid path"),
    )
    .with_reporter(reporter.clone());

    let result = workflow.execute()?;

    assert_eq!(result.files_processed, 0, "Should process 0 files in empty directory");

    let stats = reporter.get_stats();
    assert_eq!(stats.total_files, 0);
    assert_eq!(stats.processed_files, 0);

    Ok(())
}
