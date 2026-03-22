use std::fs;
use tempfile::TempDir;

use codebase_translate::config::global::GlobalConfig;
use codebase_translate::config::loader::ConfigLoader;
use codebase_translate::logger;

#[test]
fn test_log_path_resolution_with_relative_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("test_project");
    fs::create_dir_all(&project_dir).expect("Failed to create project dir");

    let config_content = r#"
[logging]
level = "info"
format = "pretty"
output = "file"
file = ".translator/translator.log"
"#;

    let config: GlobalConfig = toml::from_str(config_content).expect("Failed to parse config");

    // Test path resolution with project directory
    let resolved_path = logger::get_log_file_path(&config.logging, Some(project_dir.as_path()));
    let expected_path = project_dir.join(".translator/translator.log");
    assert_eq!(resolved_path, expected_path.to_string_lossy().to_string());
}

#[test]
fn test_log_path_resolution_without_project_dir() {
    let config_content = r#"
[logging]
level = "info"
format = "pretty"
output = "file"
file = "translator.log"
"#;

    let config: GlobalConfig = toml::from_str(config_content).expect("Failed to parse config");

    // Test path resolution without project directory (should use current directory)
    let resolved_path = logger::get_log_file_path(&config.logging, None);
    assert_eq!(resolved_path, "translator.log");
}

#[test]
fn test_log_path_resolution_with_absolute_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().join("custom_logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log dir");

    let absolute_log_path = log_dir.join("translator.log");

    let mut config: GlobalConfig = GlobalConfig::default();
    config.logging.file = Some(absolute_log_path.to_string_lossy().to_string());

    // Test that absolute path is not modified
    let resolved_path = logger::get_log_file_path(&config.logging, None);
    assert_eq!(
        resolved_path,
        absolute_log_path.to_string_lossy().to_string()
    );
}

#[test]
fn test_log_path_resolution_from_config_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");
    let project_dir = temp_dir.path().join("test_project");
    fs::create_dir_all(&project_dir).expect("Failed to create project dir");

    let config_content = r#"
[logging]
level = "info"
format = "pretty"
output = "file"
file = ".translator/translator.log"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Load config from file
    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();
    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Test path resolution with project directory
    let resolved_path = logger::get_log_file_path(&config.logging, Some(project_dir.as_path()));
    let expected_path = project_dir.join(".translator/translator.log");
    assert_eq!(resolved_path, expected_path.to_string_lossy().to_string());
}
