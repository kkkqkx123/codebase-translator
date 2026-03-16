//! Configuration management for logging

use tracing::Level;

use crate::config::global::LoggingConfig;
use crate::core::error::{Result, TranslateError};

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

/// Validate logging configuration
pub fn validate_config(config: &LoggingConfig) -> Result<()> {
    parse_level(&config.level);

    if config.output == "file" && config.file.is_none() {
        return Err(TranslateError::Config(
            "Log file path must be specified when output is 'file'".to_string(),
        ));
    }

    Ok(())
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

        assert!(validate_config(&config).is_err());
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
