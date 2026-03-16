//! Logging macros for structured logging

/// Get a logger instance for a module
#[macro_export]
macro_rules! get_logger {
    () => {
        tracing::span!(tracing::Level::DEBUG, module_path!())
    };
}

/// Create a span with fields
#[macro_export]
macro_rules! span_with_fields {
    ($level:expr, $name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::span!($level, $name, $($key = $value),*)
    };
}

/// Log a message with structured fields
#[macro_export]
macro_rules! log_fields {
    ($level:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::event!($level, $($key = $value),*)
    };
}

/// Log an error with context
#[macro_export]
macro_rules! log_error {
    ($error:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::error!(error = %$error, $($key = $value),*)
    };
}

/// Log a warning with context
#[macro_export]
macro_rules! log_warn {
    ($($key:ident = $value:expr),* $(,)?) => {
        tracing::warn!($($key = $value),*)
    };
}

/// Log info with context
#[macro_export]
macro_rules! log_info {
    ($($key:ident = $value:expr),* $(,)?) => {
        tracing::info!($($key = $value),*)
    };
}

/// Log debug with context
#[macro_export]
macro_rules! log_debug {
    ($($key:ident = $value:expr),* $(,)?) => {
        tracing::debug!($($key = $value),*)
    };
}

/// Log trace with context
#[macro_export]
macro_rules! log_trace {
    ($($key:ident = $value:expr),* $(,)?) => {
        tracing::trace!($($key = $value),*)
    };
}

/// Log a duration
#[macro_export]
macro_rules! log_duration {
    ($name:expr, $duration:expr) => {
        tracing::info!(duration_ms = $duration.as_millis(), $name)
    };
}
