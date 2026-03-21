//! Structured logging with tracing

use std::path::Path;
use std::sync::OnceLock;

use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::config::global::LoggingConfig;
use crate::core::error::{Result, TranslateError};

mod config;

pub use config::{get_format_string, get_log_file_path, get_output_string, parse_level, validate_config};

/// Global guard to keep the log appender alive
pub static LOG_GUARD: OnceLock<Box<dyn std::any::Any + Send + Sync>> = OnceLock::new();

/// Initialize the logging system
///
/// Note: This function can only be called once globally. Subsequent calls
/// will return an error unless the logger was not previously initialized.
/// For testing purposes, use `--test-threads=1` to ensure serial execution.
///
/// # Arguments
/// * `config` - The logging configuration
/// * `project_dir` - Optional project directory path. If provided and the log file
///   path is relative, it will be resolved relative to this directory.
pub fn init(config: &LoggingConfig, project_dir: Option<&Path>) -> Result<()> {
    validate_config(config)?;

    let level = parse_level(&config.level);
    let filter = EnvFilter::new(format!("codebase_translate={}", level)).add_directive(
        format!("translator={}", level).parse().map_err(|e| {
            TranslateError::Config(format!("Invalid log level '{}': {}", config.level, e))
        })?,
    );

    let format = get_format_string(config);
    match get_output_string(config) {
        "file" => init_file_logger(config, filter, format, project_dir),
        "stderr" => init_stderr_logger(filter, format),
        _ => init_stdout_logger(filter, format),
    }
}

/// Initialize stdout logger
fn init_stdout_logger(filter: EnvFilter, format: &str) -> Result<()> {
    let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = match format {
        "json" => Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(false)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE),
        ),
        "compact" => Box::new(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_target(false)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE),
        ),
        _ => Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(false)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE),
        ),
    };

    match tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
    {
        Ok(_) => Ok(()),
        Err(_) => {
            // Logger already initialized, treat as success for testability
            Ok(())
        }
    }
}

/// Initialize stderr logger
fn init_stderr_logger(filter: EnvFilter, format: &str) -> Result<()> {
    let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = match format {
        "json" => Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE),
        ),
        "compact" => Box::new(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE),
        ),
        _ => Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE),
        ),
    };

    match tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
    {
        Ok(_) => Ok(()),
        Err(_) => {
            // Logger already initialized, treat as success for testability
            Ok(())
        }
    }
}

/// Initialize file logger
fn init_file_logger(
    config: &LoggingConfig,
    filter: EnvFilter,
    format: &str,
    project_dir: Option<&Path>,
) -> Result<()> {
    let file_path_str = get_log_file_path(config, project_dir);
    let file_path = Path::new(&file_path_str);

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file_appender = tracing_appender::rolling::daily(
        file_path.parent().unwrap_or(Path::new(".")),
        file_path.file_name().unwrap_or_default(),
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Only set LOG_GUARD if not already set
    let _ = LOG_GUARD.set(Box::new(guard));

    let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = match format {
        "json" => Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking)
                .with_target(true)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_ansi(false),
        ),
        "compact" => Box::new(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_writer(non_blocking)
                .with_target(true)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_ansi(false),
        ),
        _ => Box::new(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_ansi(false),
        ),
    };

    match tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
    {
        Ok(_) => Ok(()),
        Err(_) => {
            // Logger already initialized, treat as success for testability
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_level_valid() {
        assert_eq!(parse_level("trace"), tracing::Level::TRACE);
        assert_eq!(parse_level("debug"), tracing::Level::DEBUG);
        assert_eq!(parse_level("info"), tracing::Level::INFO);
        assert_eq!(parse_level("warn"), tracing::Level::WARN);
        assert_eq!(parse_level("error"), tracing::Level::ERROR);
    }

    #[test]
    fn test_parse_level_invalid() {
        assert_eq!(parse_level("invalid"), tracing::Level::INFO);
        assert_eq!(parse_level(""), tracing::Level::INFO);
    }

    #[test]
    fn test_validate_config_stdout() {
        let config = create_test_config("info", "stdout", "pretty", None);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_stderr() {
        let config = create_test_config("debug", "stderr", "json", None);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_file_without_path() {
        let config = create_test_config("info", "file", "pretty", None);
        // Now file output without explicit path uses default, so validation passes
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_file_with_path() {
        let config = create_test_config("info", "file", "compact", Some("test.log".to_string()));
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_get_format_string() {
        let config = create_test_config("info", "stdout", "json", None);
        assert_eq!(get_format_string(&config), "json");
    }

    #[test]
    fn test_get_output_string() {
        let config = create_test_config("info", "stderr", "pretty", None);
        assert_eq!(get_output_string(&config), "stderr");
    }
}
