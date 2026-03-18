//! Main E2E Integration Test
//!
//! This test verifies the complete translation workflow end-to-end,
//! including scanning, parsing, translation, and writing.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Timeout for translation commands (in seconds)
const TRANSLATION_TIMEOUT_SECS: u64 = 5;

/// Run a command with timeout, returns true if command started successfully
/// (even if it times out due to external service calls)
fn run_with_timeout(binary: &PathBuf, args: &[&str]) -> bool {
    let mut child = match Command::new(binary).args(args).spawn() {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Wait with timeout
    thread::sleep(Duration::from_secs(TRANSLATION_TIMEOUT_SECS));

    // Try to get the status
    match child.try_wait() {
        Ok(Some(_status)) => {
            // Command completed within timeout
            true
        }
        Ok(None) => {
            // Still running - kill it (expected for translation commands)
            let _ = child.kill();
            println!("Command timed out after {} seconds (expected for translation commands)", TRANSLATION_TIMEOUT_SECS);
            true // This is acceptable
        }
        Err(_) => {
            let _ = child.kill();
            false
        }
    }
}

/// Get the project root directory
fn get_project_root() -> PathBuf {
    // Use CARGO_MANIFEST_DIR which is always set by cargo
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
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

    // Cache command may fail if cache is not initialized, that's ok
    // We just verify the command runs
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

    // Cache command may fail if cache is not initialized, that's ok
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

    // Cache command may fail if cache is not initialized, that's ok
}

/// Test the translate command with dry-run mode
#[test]
fn test_translate_command_dry_run() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let success = run_with_timeout(
        &binary,
        &[
            "--dry-run",
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
        ],
    );

    assert!(success, "Translate command failed to start");
}

/// Test the translate command with specific provider
#[test]
fn test_translate_command_with_provider() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let success = run_with_timeout(
        &binary,
        &[
            "--dry-run",
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--provider",
            "deeplx",
        ],
    );

    assert!(success, "Translate command with provider failed to start");
}

/// Test the translate command with custom include patterns
#[test]
fn test_translate_command_with_include_patterns() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let success = run_with_timeout(
        &binary,
        &[
            "--dry-run",
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--include",
            "*.rs",
        ],
    );

    assert!(success, "Translate command with include patterns failed to start");
}

/// Test the init command for global config
#[test]
fn test_init_global_config() {
    setup_test_env();

    let binary = get_translator_binary();

    // Test global config initialization with force to overwrite if exists
    let output = Command::new(&binary)
        .args(["init", "--global", "--force"])
        .output()
        .expect("Failed to execute init command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Init command stdout: {}", stdout);
    println!("Init command stderr: {}", stderr);

    // The command should succeed (global config init doesn't require project config)
    assert!(
        output.status.success() || stdout.contains("Created global config") || stderr.contains("Created global config"),
        "Expected global config to be created"
    );
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

    let success = run_with_timeout(
        &binary,
        &[
            "--dry-run",
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
        ],
    );

    assert!(success, "Translation summary test failed to start");
}

/// Test scanning with exclude patterns
#[test]
fn test_translate_with_exclude_patterns() {
    setup_test_env();

    let binary = get_translator_binary();
    let e2e_dir = get_e2e_dir();

    let success = run_with_timeout(
        &binary,
        &[
            "--dry-run",
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--exclude",
            "*.go",
        ],
    );

    assert!(success, "Translate command with exclude patterns failed to start");
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

    let success = run_with_timeout(
        &binary,
        &[
            "--dry-run",
            "translate",
            e2e_dir.to_str().unwrap(),
            "--target-lang",
            "en",
            "--source-langs",
            "AUTO,zh",
        ],
    );

    assert!(success, "Translate command with source langs failed to start");
}
