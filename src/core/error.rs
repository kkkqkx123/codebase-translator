use thiserror::Error;

/// Main error type for the translator
#[derive(Error, Debug)]
pub enum TranslateError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Parse error (includes serialization errors)
    #[error("Parse error: {0}")]
    Parse(String),

    /// Translation API error
    #[error("Translation error: {0}")]
    Translation(String),

    /// Cache operation error
    #[error("Cache error: {0}")]
    Cache(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Lock acquisition failed
    #[error("Lock error: {0}")]
    Lock(String),
}

impl Clone for TranslateError {
    fn clone(&self) -> Self {
        match self {
            TranslateError::Io(s) => TranslateError::Io(s.clone()),
            TranslateError::Config(s) => TranslateError::Config(s.clone()),
            TranslateError::Parse(s) => TranslateError::Parse(s.clone()),
            TranslateError::Translation(s) => TranslateError::Translation(s.clone()),
            TranslateError::Cache(s) => TranslateError::Cache(s.clone()),
            TranslateError::Http(s) => TranslateError::Http(s.clone()),
            TranslateError::InvalidArgument(s) => TranslateError::InvalidArgument(s.clone()),
            TranslateError::RateLimit => TranslateError::RateLimit,
            TranslateError::Authentication(s) => TranslateError::Authentication(s.clone()),
            TranslateError::NotFound(s) => TranslateError::NotFound(s.clone()),
            TranslateError::Lock(s) => TranslateError::Lock(s.clone()),
        }
    }
}

impl From<std::io::Error> for TranslateError {
    fn from(err: std::io::Error) -> Self {
        TranslateError::Io(err.to_string())
    }
}

impl From<reqwest::Error> for TranslateError {
    fn from(err: reqwest::Error) -> Self {
        TranslateError::Http(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, TranslateError>;

impl From<serde_json::Error> for TranslateError {
    fn from(err: serde_json::Error) -> Self {
        TranslateError::Parse(format!("JSON error: {}", err))
    }
}

impl From<toml::de::Error> for TranslateError {
    fn from(err: toml::de::Error) -> Self {
        TranslateError::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for TranslateError {
    fn from(err: toml::ser::Error) -> Self {
        TranslateError::Parse(format!("TOML serialize error: {}", err))
    }
}

impl From<rmp_serde::encode::Error> for TranslateError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        TranslateError::Parse(format!("MessagePack encode error: {}", err))
    }
}

impl From<rmp_serde::decode::Error> for TranslateError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        TranslateError::Parse(format!("MessagePack decode error: {}", err))
    }
}

impl From<regex::Error> for TranslateError {
    fn from(err: regex::Error) -> Self {
        TranslateError::Parse(format!("Regex error: {}", err))
    }
}

impl From<glob::PatternError> for TranslateError {
    fn from(err: glob::PatternError) -> Self {
        TranslateError::InvalidArgument(format!("Invalid glob pattern: {}", err))
    }
}

impl From<crate::encoding::error::Error> for TranslateError {
    fn from(err: crate::encoding::error::Error) -> Self {
        use crate::encoding::error::Error as EncodingError;
        match err {
            EncodingError::Io(e) => TranslateError::Io(e.to_string()),
            EncodingError::DetectionFailed(msg) => TranslateError::Parse(msg),
            EncodingError::LowConfidence {
                encoding,
                confidence,
                threshold,
            } => TranslateError::Parse(format!(
                "Low confidence encoding detection: encoding={} confidence={:.2} threshold={:.2}",
                encoding, confidence, threshold
            )),
            EncodingError::ConversionFailed {
                from_encoding,
                to_encoding,
                message,
            } => TranslateError::Parse(format!(
                "Encoding conversion failed: {} -> {}: {}",
                from_encoding, to_encoding, message
            )),
            EncodingError::UnsupportedEncoding(msg) => TranslateError::InvalidArgument(msg),
            EncodingError::InvalidBOM(msg) => TranslateError::Parse(msg),
            EncodingError::InvalidUtf8(e) => TranslateError::Parse(e.to_string()),
            EncodingError::FileNotFound(msg) => TranslateError::NotFound(msg),
            EncodingError::InvalidData(msg) => TranslateError::Parse(msg),
        }
    }
}

impl TranslateError {
    /// Create an IO error
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    /// Create a translation error
    pub fn translation(message: impl Into<String>) -> Self {
        Self::Translation(message.into())
    }

    /// Create a cache error
    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache(message.into())
    }

    /// Create an HTTP error
    pub fn http(message: impl Into<String>) -> Self {
        Self::Http(message.into())
    }

    /// Create an invalid argument error
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument(message.into())
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication(message.into())
    }

    /// Create a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Create a lock error
    pub fn lock(message: impl Into<String>) -> Self {
        Self::Lock(message.into())
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            TranslateError::Http(_) => true,
            TranslateError::Translation(msg) => {
                let msg_lower = msg.to_lowercase();
                !msg_lower.contains("invalid api key")
                    && !msg_lower.contains("authentication failed")
                    && !msg_lower.contains("unauthorized")
                    && !msg_lower.contains("forbidden")
                    && !msg_lower.contains("invalid request")
                    && !msg_lower.contains("quota exceeded")
                    && !msg_lower.contains("limit exceeded")
                    && !msg_lower.contains("insufficient")
            }
            TranslateError::Io(_) => true,
            TranslateError::Config(_) => false,
            TranslateError::Parse(_) => false,
            TranslateError::Cache(_) => true,
            TranslateError::InvalidArgument(_) => false,
            TranslateError::RateLimit => true,
            TranslateError::Authentication(_) => false,
            TranslateError::NotFound(_) => false,
            TranslateError::Lock(_) => true,
        }
    }

    /// Calculate exponential backoff delay
    pub fn calculate_backoff(attempt: usize) -> std::time::Duration {
        const BASE_DELAY_MS: u64 = 100;
        const MAX_DELAY_MS: u64 = 5000;

        let delay_ms = BASE_DELAY_MS * 2_u64.pow(attempt as u32);
        let delay_ms = delay_ms.min(MAX_DELAY_MS);

        let jitter = (delay_ms as f64 * 0.1) as u64;
        std::time::Duration::from_millis(delay_ms + jitter)
    }
}
