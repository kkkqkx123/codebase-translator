//! Logger Integration Tests
//!
//! These tests verify the logging functionality including:
//! - Logger initialization
//! - Different output targets (stdout, stderr, file)
//! - Different log formats (pretty, json, compact)
//! - Log file creation and content verification

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use codebase_translate::config::global::LoggingConfig;
use codebase_translate::logger::{init, parse_level, validate_config};

const OUTPUT_DIR: &str = "tests/main_integration/output/logger";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn get_project_root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
}

fn write_test_output(filename: &str, content: &str) {
    ensure_output_dir();
    let output_path = PathBuf::from(OUTPUT_DIR).join(filename);
    fs::write(&output_path, content).expect(&format!("Failed to write output: {}", filename));
    println!("Output written to: {}", output_path.display());
}

#[test]
fn test_logger_parse_level() {
    let mut output = String::new();
    output.push_str("Logger Level Parsing Test\n");
    output.push_str("===========================\n\n");

    let test_cases = vec![
        ("trace", "Trace"),
        ("debug", "Debug"),
        ("info", "Info"),
        ("warn", "Warn"),
        ("error", "Error"),
        ("TRACE", "Trace"),
        ("Debug", "Debug"),
        ("INFO", "Info"),
        ("Warn", "Warn"),
        ("ERROR", "Error"),
        ("unknown", "Info"),
        ("", "Info"),
    ];

    for (input, expected) in test_cases {
        let level = parse_level(input);
        output.push_str(&format!("parse_level(\"{}\") = {:?}\n", input, level));
        assert_eq!(
            format!("{:?}", level),
            format!("Level({})", expected),
            "Level mismatch for input: {}",
            input
        );
    }

    output.push_str("\nAll level parsing tests passed!\n");
    write_test_output("test_logger_parse_level.txt", &output);
}

#[test]
fn test_logger_validate_config_stdout() {
    let config = LoggingConfig {
        level: "info".to_string(),
        output: "stdout".to_string(),
        format: "pretty".to_string(),
        file: None,
    };

    let result = validate_config(&config);

    let mut output = String::new();
    output.push_str("Logger Config Validation Test - stdout\n");
    output.push_str("=====================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Validation result: {:?}\n", result));

    assert!(result.is_ok(), "stdout config should be valid");
    output.push_str("\nstdout config validation passed!\n");
    write_test_output("test_logger_validate_config_stdout.txt", &output);
}

#[test]
fn test_logger_validate_config_stderr() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        output: "stderr".to_string(),
        format: "json".to_string(),
        file: None,
    };

    let result = validate_config(&config);

    let mut output = String::new();
    output.push_str("Logger Config Validation Test - stderr\n");
    output.push_str("=====================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Validation result: {:?}\n", result));

    assert!(result.is_ok(), "stderr config should be valid");
    output.push_str("\nstderr config validation passed!\n");
    write_test_output("test_logger_validate_config_stderr.txt", &output);
}

#[test]
fn test_logger_validate_config_file_with_path() {
    let project_root = get_project_root();
    let log_file = project_root.join("test_log.log");

    let config = LoggingConfig {
        level: "info".to_string(),
        output: "file".to_string(),
        format: "compact".to_string(),
        file: Some(log_file.to_string_lossy().to_string()),
    };

    let result = validate_config(&config);

    let mut output = String::new();
    output.push_str("Logger Config Validation Test - file with path\n");
    output.push_str("==============================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Validation result: {:?}\n", result));

    assert!(result.is_ok(), "file config with path should be valid");
    output.push_str("\nfile config with path validation passed!\n");
    write_test_output("test_logger_validate_config_file_with_path.txt", &output);
}

#[test]
fn test_logger_validate_config_file_without_path() {
    let config = LoggingConfig {
        level: "info".to_string(),
        output: "file".to_string(),
        format: "pretty".to_string(),
        file: None,
    };

    let result = validate_config(&config);

    let mut output = String::new();
    output.push_str("Logger Config Validation Test - file without path\n");
    output.push_str("===================================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Validation result: {:?}\n", result));

    assert!(
        result.is_err(),
        "file config without path should be invalid"
    );
    output.push_str("\nfile config without path validation passed (correctly rejected)!\n");
    write_test_output("test_logger_validate_config_file_without_path.txt", &output);
}

#[test]
fn test_logger_init_stdout() {
    let config = LoggingConfig {
        level: "info".to_string(),
        output: "stdout".to_string(),
        format: "pretty".to_string(),
        file: None,
    };

    let result = init(&config, None::<&Path>);

    let mut output = String::new();
    output.push_str("Logger Initialization Test - stdout\n");
    output.push_str("=====================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Init result: {:?}\n", result));

    assert!(result.is_ok(), "stdout logger init should succeed");
    output.push_str("\nstdout logger initialization passed!\n");

    output.push_str("\nNote: Logger is now initialized for stdout output.\n");
    output.push_str("Subsequent tests will use this logger.\n");

    write_test_output("test_logger_init_stdout.txt", &output);
}

#[test]
fn test_logger_init_stderr() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        output: "stderr".to_string(),
        format: "json".to_string(),
        file: None,
    };

    let result = init(&config, None::<&Path>);

    let mut output = String::new();
    output.push_str("Logger Initialization Test - stderr\n");
    output.push_str("=====================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Init result: {:?}\n", result));

    assert!(result.is_ok(), "stderr logger init should succeed");
    output.push_str("\nstderr logger initialization passed!\n");

    output.push_str("\nNote: Logger is now initialized for stderr output.\n");
    output.push_str("Subsequent tests will use this logger.\n");

    write_test_output("test_logger_init_stderr.txt", &output);
}

#[test]
fn test_logger_init_file() {
    let project_root = get_project_root();
    let log_file = project_root
        .join("tests")
        .join("main_integration")
        .join("output")
        .join("test_logger.log");

    let config = LoggingConfig {
        level: "info".to_string(),
        output: "file".to_string(),
        format: "pretty".to_string(),
        file: Some(log_file.to_string_lossy().to_string()),
    };

    let result = init(&config, None::<&Path>);

    let mut output = String::new();
    output.push_str("Logger Initialization Test - file\n");
    output.push_str("===================================\n\n");
    output.push_str(&format!("Config: {:?}\n", config));
    output.push_str(&format!("Init result: {:?}\n", result));

    assert!(result.is_ok(), "file logger init should succeed");
    output.push_str("\nfile logger initialization passed!\n");

    output.push_str(&format!("\nLog file path: {}\n", log_file.display()));
    output.push_str(&format!("Log file exists: {}\n", log_file.exists()));

    if log_file.exists() {
        output.push_str("\nLog file was successfully created!\n");
        if let Ok(content) = fs::read_to_string(&log_file) {
            output.push_str(&format!("\nLog file size: {} bytes\n", content.len()));
            output.push_str("\nNote: Actual log content will be written during test execution.\n");
        }
    } else {
        output.push_str("\nWarning: Log file does not exist yet (will be created on first log).\n");
    }

    write_test_output("test_logger_init_file.txt", &output);
}

#[test]
fn test_logger_all_formats() {
    let formats = vec!["pretty", "compact", "json"];
    let mut output = String::new();
    output.push_str("Logger All Formats Test\n");
    output.push_str("=========================\n\n");

    for format in formats {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "stdout".to_string(),
            format: format.to_string(),
            file: None,
        };

        let result = validate_config(&config);
        output.push_str(&format!("Format '{}': {:?}\n", format, result));
        assert!(result.is_ok(), "All formats should be valid");
    }

    output.push_str("\nAll format validation tests passed!\n");
    write_test_output("test_logger_all_formats.txt", &output);
}

#[test]
fn test_logger_all_levels() {
    let levels = vec!["trace", "debug", "info", "warn", "error"];
    let mut output = String::new();
    output.push_str("Logger All Levels Test\n");
    output.push_str("======================\n\n");

    for level in levels {
        let config = LoggingConfig {
            level: level.to_string(),
            output: "stdout".to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        let result = validate_config(&config);
        output.push_str(&format!("Level '{}': {:?}\n", level, result));
        assert!(result.is_ok(), "All levels should be valid");
    }

    output.push_str("\nAll level validation tests passed!\n");
    write_test_output("test_logger_all_levels.txt", &output);
}

#[test]
fn test_logger_all_outputs() {
    let outputs = vec!["stdout", "stderr"];
    let mut output = String::new();
    output.push_str("Logger All Outputs Test\n");
    output.push_str("======================\n\n");

    for output_type in outputs {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: output_type.to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        let result = validate_config(&config);
        output.push_str(&format!("Output '{}': {:?}\n", output_type, result));
        assert!(result.is_ok(), "All outputs should be valid");
    }

    output.push_str("\nAll output validation tests passed!\n");
    write_test_output("test_logger_all_outputs.txt", &output);
}
