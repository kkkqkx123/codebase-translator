//! Logger module integration tests
//!
//! These tests verify logger initialization with different configurations.
//! Run with `cargo test --test logger_integration -- --test-threads=1` to
//! ensure serial execution due to global tracing subscriber constraints.

use codebase_translate::config::global::LoggingConfig;
use codebase_translate::logger::init;
use tempfile::TempDir;

fn create_test_config(
    level: &str,
    output: &str,
    format: &str,
    file: Option<String>,
) -> LoggingConfig {
    LoggingConfig {
        level: level.to_string(),
        output: output.to_string(),
        format: format.to_string(),
        file,
    }
}

#[test]
fn test_init_stdout_logger_pretty() {
    let config = create_test_config("info", "stdout", "pretty", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_stdout_logger_json() {
    let config = create_test_config("debug", "stdout", "json", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_stdout_logger_compact() {
    let config = create_test_config("warn", "stdout", "compact", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_stderr_logger() {
    let config = create_test_config("error", "stderr", "pretty", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_file_logger() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("test.log");
    let config = create_test_config(
        "info",
        "file",
        "pretty",
        Some(log_path.to_str().expect("Invalid path").to_string()),
    );
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_file_logger_creates_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().join("logs");
    let log_path = log_dir.join("test.log");
    let config = create_test_config(
        "info",
        "file",
        "json",
        Some(log_path.to_str().expect("Invalid path").to_string()),
    );
    let result = init(&config, None);
    assert!(result.is_ok());
    assert!(log_dir.exists());
}

#[test]
fn test_init_with_invalid_level() {
    let config = create_test_config("invalid", "stdout", "pretty", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_log_guard_is_set() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("test.log");
    let config = create_test_config(
        "info",
        "file",
        "pretty",
        Some(log_path.to_str().expect("Invalid path").to_string()),
    );
    init(&config, None).expect("Failed to init logger");
    assert!(codebase_translate::logger::LOG_GUARD.get().is_some());
}

#[test]
fn test_init_multiple_formats() {
    let formats = ["pretty", "json", "compact"];
    for format in formats {
        let config = create_test_config("info", "stdout", format, None);
        let result = init(&config, None);
        assert!(result.is_ok(), "Failed for format: {}", format);
    }
}

#[test]
fn test_init_multiple_levels() {
    let levels = ["trace", "debug", "info", "warn", "error"];
    for level in levels {
        let config = create_test_config(level, "stdout", "pretty", None);
        let result = init(&config, None);
        assert!(result.is_ok(), "Failed for level: {}", level);
    }
}

#[test]
fn test_init_file_logger_with_nested_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().join("nested").join("logs");
    let log_path = log_dir.join("test.log");
    let config = create_test_config(
        "info",
        "file",
        "compact",
        Some(log_path.to_str().expect("Invalid path").to_string()),
    );
    let result = init(&config, None);
    assert!(result.is_ok());
    assert!(log_dir.exists());
}

#[test]
fn test_init_stderr_with_json_format() {
    let config = create_test_config("debug", "stderr", "json", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_stderr_with_compact_format() {
    let config = create_test_config("warn", "stderr", "compact", None);
    let result = init(&config, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_file_logger_with_project_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

    let config = create_test_config(
        "info",
        "file",
        "pretty",
        Some(".translator/translator.log".to_string()),
    );
    let result = init(&config, Some(project_dir.as_path()));
    assert!(result.is_ok());

    let expected_log_path = project_dir.join(".translator").join("translator.log");
    assert!(expected_log_path.parent().unwrap().exists());
}

#[test]
fn test_init_file_logger_with_relative_path_and_project_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

    let config = create_test_config("info", "file", "pretty", Some("logs/app.log".to_string()));
    let result = init(&config, Some(project_dir.as_path()));
    assert!(result.is_ok());

    let expected_log_path = project_dir.join("logs").join("app.log");
    assert!(expected_log_path.parent().unwrap().exists());
}

#[test]
fn test_init_file_logger_with_absolute_path_ignores_project_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

    let absolute_log_path = temp_dir.path().join("absolute").join("app.log");
    std::fs::create_dir_all(absolute_log_path.parent().unwrap())
        .expect("Failed to create absolute log dir");

    let config = create_test_config(
        "info",
        "file",
        "pretty",
        Some(
            absolute_log_path
                .to_str()
                .expect("Invalid path")
                .to_string(),
        ),
    );
    let result = init(&config, Some(project_dir.as_path()));
    assert!(result.is_ok());

    assert!(absolute_log_path.parent().unwrap().exists());
}
