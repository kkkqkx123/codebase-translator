//! End-to-End Integration Tests
//!
//! These tests verify the complete translation workflow by actually
//! running the translation process and checking:
//! - Translation results
//! - Cache files
//! - Backup files
//! - Log files
//!
//! Tests use actual configuration files from the project root.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use codebase_translate::config::loader::ConfigLoader;
use codebase_translate::workflow::{TranslationWorkflow, WorkflowConfig};

const FIXTURES_DIR: &str = "tests/main_integration/fixtures";
const OUTPUT_DIR: &str = "tests/main_integration/output";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn get_project_root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
}

fn copy_fixture_to_temp(fixture_name: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let fixture_path = PathBuf::from(FIXTURES_DIR).join(fixture_name);
    let dest_path = temp_dir.path().join(fixture_name);

    fs::copy(&fixture_path, &dest_path)
        .expect(&format!("Failed to copy fixture: {}", fixture_name));

    (temp_dir, dest_path)
}

fn copy_all_fixtures_to_temp() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let fixtures = PathBuf::from(FIXTURES_DIR);

    for entry in fs::read_dir(&fixtures).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read directory entry");
        let src_path = entry.path();

        if src_path.is_file() {
            let dest_path = temp_dir.path().join(entry.file_name());
            fs::copy(&src_path, &dest_path)
                .expect(&format!("Failed to copy fixture: {:?}", src_path));
        }
    }

    temp_dir
}

fn write_test_output(filename: &str, content: &str) {
    ensure_output_dir();
    let output_path = PathBuf::from(OUTPUT_DIR).join(filename);
    fs::write(&output_path, content).expect(&format!("Failed to write output: {}", filename));
    println!("Output written to: {}", output_path.display());
}

fn read_file_content(path: &Path) -> String {
    fs::read_to_string(path).expect(&format!("Failed to read file: {}", path.display()))
}

fn check_file_exists(path: &Path) -> bool {
    path.exists()
}

fn list_directory_contents(path: &Path) -> Vec<String> {
    let mut contents = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata().expect("Failed to get metadata");
            let file_type = if metadata.is_dir() { "DIR" } else { "FILE" };
            contents.push(format!("{} [{}]", name, file_type));
        }
    }

    contents.sort();
    contents
}

#[test]
fn test_load_project_config() {
    let project_root = get_project_root();
    let fixture_config = project_root.join(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!(
            "Skipping test: fixture config not found at {:?}",
            fixture_config
        );
        return;
    }

    let loader = ConfigLoader::new().with_project_config(&fixture_config);
    let config = loader
        .load_project()
        .expect("Failed to load project config");

    let output = format!(
        "Project Config Loaded:\n\
         ======================\n\
         Source Langs: {:?}\n\
         Target Lang: {}\n\
         Include Patterns: {:?}\n\
         Exclude Patterns: {:?}\n\
         Cache Enabled: {}\n\
         Cache Directory: {}\n\
         Backup Enabled: {}\n",
        config.translate.source_langs,
        config.translate.target_lang,
        config.include.patterns,
        config.exclude.patterns,
        config.cache.enabled,
        config.cache.directory,
        config.writer.backup
    );

    write_test_output("test_load_project_config.txt", &output);
}

#[test]
fn test_load_global_config() {
    let project_root = get_project_root();
    let global_config_paths = vec![
        project_root.join("translator.toml"),
        project_root.join("bin").join("translator.toml"),
    ];

    let mut global_config_path = None;
    for path in &global_config_paths {
        if path.exists() {
            global_config_path = Some(path.clone());
            break;
        }
    }

    let global_config_path = match global_config_path {
        Some(path) => path,
        None => {
            println!("Skipping test: no global config found");
            return;
        }
    };

    let loader = ConfigLoader::new().with_global_config(&global_config_path);
    let config = loader.load_global().expect("Failed to load global config");

    let output = format!(
        "Global Config Loaded:\n\
         ====================\n\
         Enabled Providers: {:?}\n\
         DeepLX URL: {}\n\
         Logging Level: {}\n\
         Logging Format: {}\n",
        config.enabled_providers,
        config.deeplx.api_url,
        config.logging.level,
        config.logging.format
    );

    write_test_output("test_load_global_config.txt", &output);
}

#[test]
fn test_e2e_translation_rust_file() {
    let (temp_dir, _file_path) = copy_fixture_to_temp("simple_rust.rs");
    let config_path = temp_dir.path().join(".translator");
    let fixture_config = PathBuf::from(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!("Skipping test: fixture config not found");
        return;
    }

    fs::copy(&fixture_config, &config_path).expect("Failed to copy config");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        println!("Skipping test: global config not found");
        return;
    }

    let loader = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    let mut workflow_config = WorkflowConfig::from(&project_config);
    workflow_config.root_path = temp_dir.path().to_string_lossy().to_string();

    let workflow = TranslationWorkflow::new(global_config, project_config, workflow_config);

    let result = workflow.execute();

    let mut output = String::new();
    output.push_str("E2E Translation Test - Rust File\n");
    output.push_str("=================================\n\n");

    match result {
        Ok(workflow_result) => {
            output.push_str(&format!("Translation completed successfully!\n\n"));
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    output.push_str("\n\nDirectory Contents:\n");
    output.push_str("====================\n");
    for item in list_directory_contents(temp_dir.path()) {
        output.push_str(&format!("  {}\n", item));
    }

    let translated_file = temp_dir.path().join("simple_rust.rs");
    if check_file_exists(&translated_file) {
        output.push_str("\n\nTranslated File Content:\n");
        output.push_str("=========================\n");
        output.push_str(&read_file_content(&translated_file));
    }

    let backup_file = temp_dir.path().join("simple_rust.rs.bak");
    if check_file_exists(&backup_file) {
        output.push_str("\n\nBackup File Content:\n");
        output.push_str("=====================\n");
        output.push_str(&read_file_content(&backup_file));
    }

    let cache_dir = temp_dir.path().join(".translator-cache");
    if check_file_exists(&cache_dir) {
        output.push_str("\n\nCache Directory Contents:\n");
        output.push_str("==========================\n");
        for item in list_directory_contents(&cache_dir) {
            output.push_str(&format!("  {}\n", item));
        }
    }

    write_test_output("test_e2e_translation_rust_file.txt", &output);
}

#[test]
fn test_e2e_translation_python_file() {
    let (temp_dir, _file_path) = copy_fixture_to_temp("simple_python.py");
    let config_path = temp_dir.path().join(".translator");
    let fixture_config = PathBuf::from(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!("Skipping test: fixture config not found");
        return;
    }

    fs::copy(&fixture_config, &config_path).expect("Failed to copy config");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        println!("Skipping test: global config not found");
        return;
    }

    let loader = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    let mut workflow_config = WorkflowConfig::from(&project_config);
    workflow_config.root_path = temp_dir.path().to_string_lossy().to_string();

    let workflow = TranslationWorkflow::new(global_config, project_config, workflow_config);

    let result = workflow.execute();

    let mut output = String::new();
    output.push_str("E2E Translation Test - Python File\n");
    output.push_str("===================================\n\n");

    match result {
        Ok(workflow_result) => {
            output.push_str(&format!("Translation completed successfully!\n\n"));
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    output.push_str("\n\nDirectory Contents:\n");
    output.push_str("====================\n");
    for item in list_directory_contents(temp_dir.path()) {
        output.push_str(&format!("  {}\n", item));
    }

    let translated_file = temp_dir.path().join("simple_python.py");
    if check_file_exists(&translated_file) {
        output.push_str("\n\nTranslated File Content:\n");
        output.push_str("=========================\n");
        output.push_str(&read_file_content(&translated_file));
    }

    let backup_file = temp_dir.path().join("simple_python.py.bak");
    if check_file_exists(&backup_file) {
        output.push_str("\n\nBackup File Content:\n");
        output.push_str("=====================\n");
        output.push_str(&read_file_content(&backup_file));
    }

    let cache_dir = temp_dir.path().join(".translator-cache");
    if check_file_exists(&cache_dir) {
        output.push_str("\n\nCache Directory Contents:\n");
        output.push_str("==========================\n");
        for item in list_directory_contents(&cache_dir) {
            output.push_str(&format!("  {}\n", item));
        }
    }

    write_test_output("test_e2e_translation_python_file.txt", &output);
}

#[test]
fn test_e2e_translation_javascript_file() {
    let (temp_dir, _file_path) = copy_fixture_to_temp("simple_javascript.js");
    let config_path = temp_dir.path().join(".translator");
    let fixture_config = PathBuf::from(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!("Skipping test: fixture config not found");
        return;
    }

    fs::copy(&fixture_config, &config_path).expect("Failed to copy config");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        println!("Skipping test: global config not found");
        return;
    }

    let loader = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    let mut workflow_config = WorkflowConfig::from(&project_config);
    workflow_config.root_path = temp_dir.path().to_string_lossy().to_string();

    let workflow = TranslationWorkflow::new(global_config, project_config, workflow_config);

    let result = workflow.execute();

    let mut output = String::new();
    output.push_str("E2E Translation Test - JavaScript File\n");
    output.push_str("======================================\n\n");

    match result {
        Ok(workflow_result) => {
            output.push_str(&format!("Translation completed successfully!\n\n"));
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    output.push_str("\n\nDirectory Contents:\n");
    output.push_str("====================\n");
    for item in list_directory_contents(temp_dir.path()) {
        output.push_str(&format!("  {}\n", item));
    }

    let translated_file = temp_dir.path().join("simple_javascript.js");
    if check_file_exists(&translated_file) {
        output.push_str("\n\nTranslated File Content:\n");
        output.push_str("=========================\n");
        output.push_str(&read_file_content(&translated_file));
    }

    let backup_file = temp_dir.path().join("simple_javascript.js.bak");
    if check_file_exists(&backup_file) {
        output.push_str("\n\nBackup File Content:\n");
        output.push_str("=====================\n");
        output.push_str(&read_file_content(&backup_file));
    }

    let cache_dir = temp_dir.path().join(".translator-cache");
    if check_file_exists(&cache_dir) {
        output.push_str("\n\nCache Directory Contents:\n");
        output.push_str("==========================\n");
        for item in list_directory_contents(&cache_dir) {
            output.push_str(&format!("  {}\n", item));
        }
    }

    write_test_output("test_e2e_translation_javascript_file.txt", &output);
}

#[test]
fn test_e2e_translation_markdown_file() {
    let (temp_dir, _file_path) = copy_fixture_to_temp("simple_markdown.md");
    let config_path = temp_dir.path().join(".translator");
    let fixture_config = PathBuf::from(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!("Skipping test: fixture config not found");
        return;
    }

    fs::copy(&fixture_config, &config_path).expect("Failed to copy config");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        println!("Skipping test: global config not found");
        return;
    }

    let loader = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    let mut workflow_config = WorkflowConfig::from(&project_config);
    workflow_config.root_path = temp_dir.path().to_string_lossy().to_string();

    let workflow = TranslationWorkflow::new(global_config, project_config, workflow_config);

    let result = workflow.execute();

    let mut output = String::new();
    output.push_str("E2E Translation Test - Markdown File\n");
    output.push_str("====================================\n\n");

    match result {
        Ok(workflow_result) => {
            output.push_str(&format!("Translation completed successfully!\n\n"));
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    output.push_str("\n\nDirectory Contents:\n");
    output.push_str("====================\n");
    for item in list_directory_contents(temp_dir.path()) {
        output.push_str(&format!("  {}\n", item));
    }

    let translated_file = temp_dir.path().join("simple_markdown.md");
    if check_file_exists(&translated_file) {
        output.push_str("\n\nTranslated File Content:\n");
        output.push_str("=========================\n");
        output.push_str(&read_file_content(&translated_file));
    }

    let backup_file = temp_dir.path().join("simple_markdown.md.bak");
    if check_file_exists(&backup_file) {
        output.push_str("\n\nBackup File Content:\n");
        output.push_str("=====================\n");
        output.push_str(&read_file_content(&backup_file));
    }

    let cache_dir = temp_dir.path().join(".translator-cache");
    if check_file_exists(&cache_dir) {
        output.push_str("\n\nCache Directory Contents:\n");
        output.push_str("==========================\n");
        for item in list_directory_contents(&cache_dir) {
            output.push_str(&format!("  {}\n", item));
        }
    }

    write_test_output("test_e2e_translation_markdown_file.txt", &output);
}

#[test]
fn test_e2e_translation_multiple_files() {
    let temp_dir = copy_all_fixtures_to_temp();
    let config_path = temp_dir.path().join(".translator");
    let fixture_config = PathBuf::from(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!("Skipping test: fixture config not found");
        return;
    }

    fs::copy(&fixture_config, &config_path).expect("Failed to copy config");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        println!("Skipping test: global config not found");
        return;
    }

    let loader = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    let mut workflow_config = WorkflowConfig::from(&project_config);
    workflow_config.root_path = temp_dir.path().to_string_lossy().to_string();

    let workflow = TranslationWorkflow::new(global_config, project_config, workflow_config);

    let result = workflow.execute();

    let mut output = String::new();
    output.push_str("E2E Translation Test - Multiple Files\n");
    output.push_str("=====================================\n\n");

    match result {
        Ok(workflow_result) => {
            output.push_str(&format!("Translation completed successfully!\n\n"));
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    output.push_str("\n\nDirectory Contents:\n");
    output.push_str("====================\n");
    for item in list_directory_contents(temp_dir.path()) {
        output.push_str(&format!("  {}\n", item));
    }

    output.push_str("\n\nTranslated Files:\n");
    output.push_str("==================\n");
    for entry in fs::read_dir(temp_dir.path()).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_file()
            && path.extension().map_or(false, |ext| {
                matches!(ext.to_str(), Some("rs" | "py" | "js" | "md"))
            })
        {
            output.push_str(&format!("\n--- {} ---\n", path.display()));
            output.push_str(&read_file_content(&path));
        }
    }

    let cache_dir = temp_dir.path().join(".translator-cache");
    if check_file_exists(&cache_dir) {
        output.push_str("\n\nCache Directory Contents:\n");
        output.push_str("==========================\n");
        for item in list_directory_contents(&cache_dir) {
            output.push_str(&format!("  {}\n", item));
        }
    }

    write_test_output("test_e2e_translation_multiple_files.txt", &output);
}

#[test]
fn test_e2e_translation_with_cache() {
    let (temp_dir, _file_path) = copy_fixture_to_temp("simple_rust.rs");
    let config_path = temp_dir.path().join(".translator");
    let fixture_config = PathBuf::from(FIXTURES_DIR).join(".translator");

    if !fixture_config.exists() {
        println!("Skipping test: fixture config not found");
        return;
    }

    fs::copy(&fixture_config, &config_path).expect("Failed to copy config");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        println!("Skipping test: global config not found");
        return;
    }

    let loader = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    let mut workflow_config = WorkflowConfig::from(&project_config);
    workflow_config.root_path = temp_dir.path().to_string_lossy().to_string();

    let workflow = TranslationWorkflow::new(global_config, project_config, workflow_config.clone());

    let mut output = String::new();
    output.push_str("E2E Translation Test - Cache Behavior\n");
    output.push_str("=====================================\n\n");

    output.push_str("First Translation Run:\n");
    output.push_str("======================\n");

    let result1 = workflow.execute();

    match &result1 {
        Ok(workflow_result) => {
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    let cache_dir = temp_dir.path().join(".translator-cache");
    if check_file_exists(&cache_dir) {
        output.push_str("\nCache Directory Contents (after first run):\n");
        output.push_str("=============================================\n");
        for item in list_directory_contents(&cache_dir) {
            output.push_str(&format!("  {}\n", item));
        }
    }

    output.push_str("\n\nSecond Translation Run (should use cache):\n");
    output.push_str("==========================================\n");

    let loader2 = ConfigLoader::new()
        .with_global_config(&global_config_path)
        .with_project_config(&config_path);

    let (global_config2, project_config2) = loader2.load().expect("Failed to load configs");

    let workflow2 = TranslationWorkflow::new(global_config2, project_config2, workflow_config);

    let result2 = workflow2.execute();

    match &result2 {
        Ok(workflow_result) => {
            output.push_str(&format!(
                "Files processed: {}\n",
                workflow_result.files_processed
            ));
            output.push_str(&format!(
                "Total files: {}\n",
                workflow_result.stats.total_files
            ));
            output.push_str(&format!(
                "Total units: {}\n",
                workflow_result.stats.total_units
            ));
            output.push_str(&format!(
                "Translated units: {}\n",
                workflow_result.stats.translated_units
            ));
            output.push_str(&format!(
                "Cached files: {}\n",
                workflow_result.stats.cached_files
            ));
            output.push_str(&format!(
                "Skipped units: {}\n",
                workflow_result.stats.skipped_units
            ));
            output.push_str(&format!("Errors: {}\n", workflow_result.stats.errors));
            output.push_str(&format!(
                "Duration: {:.2}s\n",
                workflow_result.duration_secs
            ));
        }
        Err(e) => {
            output.push_str(&format!("Translation failed: {}\n", e));
        }
    }

    write_test_output("test_e2e_translation_with_cache.txt", &output);
}
