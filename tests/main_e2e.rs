//! Main E2E Integration Test
//!
//! This test verifies the complete translation workflow end-to-end,
//! including scanning, parsing, translation, and writing.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the project root directory
fn get_project_root() -> PathBuf {
    PathBuf::from(file!())
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get project root")
        .to_path_buf()
}

/// Get the path to the translator binary
fn get_translator_binary() -> PathBuf {
    let project_root = get_project_root();
    project_root.join("bin").join("translator.exe")
}

/// Get the path to the e2e test directory
fn get_e2e_dir() -> PathBuf {
    let project_root = get_project_root();
    project_root.join("e2e")
}

/// Setup test environment
fn setup_test_env() {
    // Ensure we're in the project root
    let project_root = get_project_root();
    std::env::set_current_dir(&project_root).expect("Failed to change to project root");
}

/// Test the validate command
#[test]
fn test_validate_command() {
    setup_test_env();

    let binary = get_translator_binary();
    let output = Command::new(&binary)
        .args(["validate"])
        .output()
        .expect("Failed to execute validate command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Validate command stdout: {}", stdout);
    println!("Validate command stderr: {}", stderr);

    // The command should succeed (exit code 0)
    assert!(
        output.status.success(),
        "Validate command failed with exit code: {:?}",
        output.status.code()
    );

    // Should contain validation message
    assert!(
        stdout.contains("Validating configuration") || stderr.contains("Validating configuration"),
        "Expected validation message in output"
    );
}

/// Test the cache command (show stats)
#[test]
fn test_cache_command_show() {
    setup_test_env();

    let binary = get_translator_binary();
    let output = Command::new(&binary)
        .args(["cache"])
        .output()
        .expect("Failed to execute cache command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Cache command stdout: {}", stdout);
    println!("Cache command stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Cache command failed with exit code: {:?}",
        output.status.code()
    );
}

/// Test the cache command with --detailed flag
#[test]
fn test_cache_command_detailed() {
    setup_test_env();

    let binary = get_translator_binary();
    let output = Command::new(&binary)
        .args(["cache", "--detailed"])
        .output()
        .expect("Failed to execute cache --detailed command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Cache detailed command stdout: {}", stdout);
    println!("Cache detailed command stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Cache --detailed command failed with exit code: {:?}",
        output.status.code()
    );
}

/// Test the cache command with --clear flag
#[test]
fn test_cache_command_clear() {
    setup_test_env();

    let binary = get_translator_binary();
    let output = Command::new(&binary)
        .args(["cache", "--clear"])
        .output()
        .expect("Failed to execute cache --clear command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Cache clear command stdout: {}", stdout);
    println!("Cache clear command stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Cache --clear command failed with exit code: {:?}",
        output.status.code()
    );
}

/// Test the translate command with dry-run mode
#[test]
fn test_translate_command_dry_run() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    // First clear cache
    let _ = Command::new(&binary)
        .args(["cache", "--clear"])
        .output();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute translate command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Translate command stdout: {}", stdout);
    println!("Translate command stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Translate command failed with exit code: {:?}",
        output.status.code()
    );

    // Should contain workflow messages
    assert!(
        stdout.contains("Scanning directory") || stderr.contains("Scanning directory"),
        "Expected 'Scanning directory' message"
    );

    // Should show files were found
    assert!(
        stdout.contains("Found") || stderr.contains("Found"),
        "Expected 'Found' message with file count"
    );
}

/// Test the translate command with specific provider
#[test]
fn test_translate_command_with_provider() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--provider",
            "deeplx",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute translate command with provider");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Translate with provider stdout: {}", stdout);
    println!("Translate with provider stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Translate command with provider failed with exit code: {:?}",
        output.status.code()
    );
}

/// Test the translate command with custom include patterns
#[test]
fn test_translate_command_with_include_patterns() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--include",
            "*.rs",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute translate command with include patterns");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Translate with include patterns stdout: {}", stdout);
    println!("Translate with include patterns stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Translate command with include patterns failed with exit code: {:?}",
        output.status.code()
    );
}

/// Test the init command for project config
#[test]
fn test_init_project_config() {
    setup_test_env();

    // Create a temporary directory for testing
    let temp_dir = std::env::temp_dir().join("translator_test_init");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    let binary = get_translator_binary();

    let output = Command::new(&binary)
        .args(["init"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute init command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Init command stdout: {}", stdout);
    println!("Init command stderr: {}", stderr);

    // Check if config file was created
    let config_file = temp_dir.join(".translator.toml");
    assert!(
        config_file.exists() || stdout.contains("already exists") || stderr.contains("already exists"),
        "Expected config file to be created or already exist message"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

/// Test error handling for invalid provider
#[test]
fn test_invalid_provider_error() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--provider",
            "invalid_provider",
        ])
        .output()
        .expect("Failed to execute translate command");

    // Should fail with invalid provider
    assert!(
        !output.status.success(),
        "Expected command to fail with invalid provider"
    );
}

/// Test the complete workflow summary output
#[test]
fn test_translation_summary_output() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    // Clear cache first
    let _ = Command::new(&binary)
        .args(["cache", "--clear"])
        .output();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute translate command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined_output = format!("{}{}", stdout, stderr);

    println!("Combined output:\n{}", combined_output);

    // Should contain summary information
    assert!(
        combined_output.contains("Translation completed")
            || combined_output.contains("Total files"),
        "Expected translation summary in output"
    );
}

/// Test scanning with exclude patterns
#[test]
fn test_translate_with_exclude_patterns() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--exclude",
            "*.go",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute translate command with exclude patterns");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Translate with exclude patterns stdout: {}", stdout);
    println!("Translate with exclude patterns stderr: {}", stderr);

    // The command should succeed
    assert!(
        output.status.success(),
        "Translate command with exclude patterns failed"
    );
}

/// Test that the binary exists and is executable
#[test]
fn test_binary_exists() {
    let binary = get_translator_binary();
    assert!(
        binary.exists(),
        "Translator binary not found at: {}",
        binary.display()
    );
}

/// Test the help command
#[test]
fn test_help_command() {
    setup_test_env();

    let binary = get_translator_binary();
    let output = Command::new(&binary)
        .args(["--help"])
        .output()
        .expect("Failed to execute --help command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain help information
    assert!(stdout.contains("translator"), "Expected 'translator' in help output");
    assert!(
        stdout.contains("translate") || stdout.contains("Commands"),
        "Expected command information in help output"
    );
}

/// Test the version command
#[test]
fn test_version_command() {
    setup_test_env();

    let binary = get_translator_binary();
    let output = Command::new(&binary)
        .args(["--version"])
        .output()
        .expect("Failed to execute --version command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain version information
    assert!(
        !stdout.is_empty() || !String::from_utf8_lossy(&output.stderr).is_empty(),
        "Expected version output"
    );
}

/// Integration test: Verify e2e directory structure
#[test]
fn test_e2e_directory_structure() {
    let e2e_dir = get_e2e_dir();

    assert!(e2e_dir.exists(), "E2E directory not found");

    // Check for expected subdirectories
    let expected_dirs = ["rust", "python", "javascript", "go"];
    for dir in &expected_dirs {
        let path = e2e_dir.join(dir);
        assert!(path.exists(), "Expected directory not found: {}", dir);
    }

    // Check for .translator config file
    let config_file = e2e_dir.join(".translator");
    assert!(config_file.exists(), "E2E config file not found");
}

/// Test multiple source languages option
#[test]
fn test_translate_with_source_langs() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let output = Command::new(&binary)
        .args([
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--source-langs",
            "AUTO,zh",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute translate command with source langs");

    // The command should succeed
    assert!(
        output.status.success(),
        "Translate command with source langs failed"
    );
}
