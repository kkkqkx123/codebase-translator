//! Configuration management for logging

use tracing::Level;

use crate::config::global::LoggingConfig;
use crate::core::error::Result;
use std::path::Path;

/// Parse log level string to tracing::Level
pub fn parse_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

/// Default log file path (relative to project directory)
pub const DEFAULT_LOG_FILE: &str = ".translator/translator.log";

/// Validate logging configuration
pub fn validate_config(config: &LoggingConfig) -> Result<()> {
    parse_level(&config.level);

    // Validate output format
    let valid_formats = ["pretty", "json", "compact"];
    if !valid_formats.contains(&config.format.as_str()) {
        return Err(crate::core::error::TranslateError::Config(format!(
            "Invalid log format: '{}'. Valid values: pretty, json, compact",
            config.format
        )));
    }

    // Validate output target
    let valid_outputs = ["stdout", "stderr", "file"];
    if !valid_outputs.contains(&config.output.as_str()) {
        return Err(crate::core::error::TranslateError::Config(format!(
            "Invalid log output: '{}'. Valid values: stdout, stderr, file",
            config.output
        )));
    }

    // File output is always valid, will use default path if not specified
    // The actual path resolution happens in get_log_file_path

    Ok(())
}

/// Get log file path with default fallback
/// 
/// If project_dir is provided and the file path is relative, it will be resolved
/// relative to the project directory. Otherwise, it will be resolved relative to
/// the current working directory.
pub fn get_log_file_path(config: &LoggingConfig, project_dir: Option<&Path>) -> String {
    let file_path = config
        .file
        .clone()
        .unwrap_or_else(|| DEFAULT_LOG_FILE.to_string());

    resolve_log_path(&file_path, project_dir)
}

/// Resolve log path relative to project directory if provided
fn resolve_log_path(file_path: &str, project_dir: Option<&Path>) -> String {
    let path = Path::new(file_path);

    if path.is_absolute() {
        return file_path.to_string();
    }

    if let Some(project_dir) = project_dir {
        project_dir.join(file_path).to_string_lossy().to_string()
    } else {
        file_path.to_string()
    }
}

/// Get the output format string
pub fn get_format_string(config: &LoggingConfig) -> &str {
    config.format.as_str()
}

/// Get the output target string
pub fn get_output_string(config: &LoggingConfig) -> &str {
    config.output.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_level() {
        assert_eq!(parse_level("trace"), Level::TRACE);
        assert_eq!(parse_level("debug"), Level::DEBUG);
        assert_eq!(parse_level("info"), Level::INFO);
        assert_eq!(parse_level("warn"), Level::WARN);
        assert_eq!(parse_level("error"), Level::ERROR);
        assert_eq!(parse_level("unknown"), Level::INFO);
    }

    #[test]
    fn test_parse_level_case_insensitive() {
        assert_eq!(parse_level("TRACE"), Level::TRACE);
        assert_eq!(parse_level("Debug"), Level::DEBUG);
        assert_eq!(parse_level("INFO"), Level::INFO);
        assert_eq!(parse_level("Warn"), Level::WARN);
        assert_eq!(parse_level("ERROR"), Level::ERROR);
    }

    #[test]
    fn test_validate_config_valid() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "stdout".to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_file_without_path() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        // Now file output without explicit path uses default, so validation passes
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_get_log_file_path_default() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        assert_eq!(get_log_file_path(&config, None), DEFAULT_LOG_FILE);
    }

    #[test]
    fn test_get_log_file_path_with_project_dir() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        let project_dir = Path::new("/home/user/project");
        assert_eq!(
            get_log_file_path(&config, Some(project_dir)),
            "/home/user/project/.translator/translator.log"
        );
    }

    #[test]
    fn test_get_log_file_path_custom() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: Some("custom.log".to_string()),
        };

        assert_eq!(get_log_file_path(&config, None), "custom.log");
    }

    #[test]
    fn test_get_log_file_path_custom_with_project_dir() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: Some("custom.log".to_string()),
        };

        let project_dir = Path::new("/home/user/project");
        assert_eq!(
            get_log_file_path(&config, Some(project_dir)),
            "/home/user/project/custom.log"
        );
    }

    #[test]
    fn test_get_log_file_path_absolute_path() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: Some("/var/log/translator.log".to_string()),
        };

        let project_dir = Path::new("/home/user/project");
        // Absolute path should not be affected by project_dir
        assert_eq!(
            get_log_file_path(&config, Some(project_dir)),
            "/var/log/translator.log"
        );
    }

    #[test]
    fn test_validate_config_file_with_path() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "file".to_string(),
            format: "pretty".to_string(),
            file: Some("test.log".to_string()),
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_get_format_string() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "stdout".to_string(),
            format: "json".to_string(),
            file: None,
        };

        assert_eq!(get_format_string(&config), "json");
    }

    #[test]
    fn test_get_output_string() {
        let config = LoggingConfig {
            level: "info".to_string(),
            output: "stderr".to_string(),
            format: "pretty".to_string(),
            file: None,
        };

        assert_eq!(get_output_string(&config), "stderr");
    }
}
