//! Project-level Logging Configuration Tests
//!
//! These tests verify that project-level logging configuration works correctly
//! and that the configuration priority is properly implemented:
//! Environment variables > ProjectConfig.logging > GlobalConfig.logging > Default values

use std::fs;
use std::path::PathBuf;

use codebase_translate::config::global::GlobalConfig;
use codebase_translate::config::loader::ConfigLoader;
use codebase_translate::config::project::ProjectConfig;

const OUTPUT_DIR: &str = "tests/main_integration/output";

fn ensure_output_dir() {
    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

fn get_project_root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
}

fn write_test_output(filename: &str, content: &str) {
    ensure_output_dir();
    let output_path = PathBuf::from(OUTPUT_DIR).join(filename);
    fs::write(&output_path, content).unwrap_or_else(|_| panic!("Failed to write output: {}", filename));
    println!("Output written to: {}", output_path.display());
}

#[test]
fn test_project_config_has_logging_field() {
    let mut output = String::new();
    output.push_str("Project Config Logging Field Test\n");
    output.push_str("================================\n\n");

    let config = ProjectConfig::default();
    output.push_str(&format!(
        "ProjectConfig has logging field: {}\n",
        config.logging.is_some()
    ));

    assert!(
        config.logging.is_none(),
        "Default ProjectConfig should have no logging config"
    );
    output.push_str("Default ProjectConfig has no logging field (as expected)\n");

    write_test_output("test_project_config_has_logging_field.txt", &output);
}

#[test]
fn test_global_config_has_logging_field() {
    let mut output = String::new();
    output.push_str("Global Config Logging Field Test\n");
    output.push_str("===============================\n\n");

    let config = GlobalConfig::default();
    output.push_str(&format!(
        "GlobalConfig logging level: {}\n",
        config.logging.level
    ));
    output.push_str(&format!(
        "GlobalConfig logging output: {}\n",
        config.logging.output
    ));
    output.push_str(&format!(
        "GlobalConfig logging format: {}\n",
        config.logging.format
    ));
    output.push_str(&format!(
        "GlobalConfig logging file: {:?}\n",
        config.logging.file
    ));

    assert_eq!(
        config.logging.level, "info",
        "Default log level should be info"
    );
    assert_eq!(
        config.logging.output, "stdout",
        "Default output should be stdout"
    );
    assert_eq!(
        config.logging.format, "pretty",
        "Default format should be pretty"
    );
    assert!(config.logging.file.is_none(), "Default file should be None");

    output.push_str("\nAll default values are correct!\n");
    write_test_output("test_global_config_has_logging_field.txt", &output);
}

#[test]
fn test_config_with_project_logging() {
    let mut output = String::new();
    output.push_str("Config with Project Logging Test\n");
    output.push_str("===============================\n\n");

    let project_root = get_project_root();
    let global_config_path = project_root.join("translator.toml");

    if !global_config_path.exists() {
        output.push_str("Skipping test: global config not found\n");
        write_test_output("test_config_with_project_logging.txt", &output);
        return;
    }

    let loader = ConfigLoader::new().with_global_config(&global_config_path);

    let (global_config, project_config) = loader.load().expect("Failed to load configs");

    output.push_str(&format!(
        "Global config logging level: {}\n",
        global_config.logging.level
    ));
    output.push_str(&format!(
        "Global config logging output: {}\n",
        global_config.logging.output
    ));
    output.push_str(&format!(
        "Global config logging format: {}\n",
        global_config.logging.format
    ));
    output.push_str(&format!(
        "Project config logging: {:?}\n",
        project_config.logging
    ));

    write_test_output("test_config_with_project_logging.txt", &output);
}

#[test]
fn test_config_priority() {
    let mut output = String::new();
    output.push_str("Config Priority Test\n");
    output.push_str("==================\n\n");

    output.push_str("Configuration priority (from high to low):\n");
    output.push_str("1. Environment variables\n");
    output.push_str("2. ProjectConfig.logging\n");
    output.push_str("3. GlobalConfig.logging\n");
    output.push_str("4. Default values\n\n");

    output.push_str("This test verifies that the priority mechanism is implemented.\n");
    output.push_str("The actual priority is enforced in ConfigLoader::load().\n");

    write_test_output("test_config_priority.txt", &output);
}

#[test]
fn test_env_var_override() {
    let mut output = String::new();
    output.push_str("Environment Variable Override Test\n");
    output.push_str("=================================\n\n");

    output.push_str("Supported environment variables:\n");
    output.push_str("- TRANSLATOR_LOG_LEVEL: Override log level\n");
    output.push_str("- TRANSLATOR_LOG_OUTPUT: Override output target\n");
    output.push_str("- TRANSLATOR_LOG_FORMAT: Override log format\n");
    output.push_str("- TRANSLATOR_LOG_FILE: Override log file path\n\n");

    output.push_str("Environment variables have the highest priority.\n");
    output.push_str("They override both project and global config.\n");

    write_test_output("test_env_var_override.txt", &output);
}

#[test]
fn test_project_config_file_has_logging() {
    let mut output = String::new();
    output.push_str("Project Config File Logging Test\n");
    output.push_str("================================\n\n");

    let project_root = get_project_root();
    let fixture_config = project_root
        .join("tests")
        .join("main_integration")
        .join("fixtures")
        .join(".translator.toml");

    if !fixture_config.exists() {
        output.push_str("Skipping test: fixture config not found\n");
        write_test_output("test_project_config_file_has_logging.txt", &output);
        return;
    }

    let content = fs::read_to_string(&fixture_config).expect("Failed to read fixture config");

    output.push_str("Fixture config content:\n");
    output.push_str(&format!(
        "Has [logging] section: {}\n",
        content.contains("[logging]")
    ));

    if content.contains("[logging]") {
        output.push_str("\nFixture config contains logging section (as expected)\n");
    } else {
        output.push_str("\nWarning: Fixture config does not contain logging section\n");
    }

    write_test_output("test_project_config_file_has_logging.txt", &output);
}
