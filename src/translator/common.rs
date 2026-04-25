//! Common types and utilities for translators

use serde::{Deserialize, Serialize};

/// Token to character conversion ratio
///
/// This is a conservative estimate to account for Unicode characters and mixed languages.
/// 1 token ≈ 1.5 characters on average.
const TOKEN_TO_CHAR_RATIO: f64 = 1.5;

/// Convert character count to estimated token count
pub fn chars_to_tokens(chars: usize) -> usize {
    ((chars as f64) / TOKEN_TO_CHAR_RATIO).ceil() as usize
}

/// Convert token count to estimated character limit
pub fn tokens_to_chars(tokens: usize) -> usize {
    ((tokens as f64) * TOKEN_TO_CHAR_RATIO).floor() as usize
}

/// Translation request
#[derive(Debug, Clone)]
pub struct TranslateRequest {
    /// Text to translate
    pub text: String,
    /// Source language code
    pub source_lang: String,
    /// Target language code
    pub target_lang: String,
}

/// Translation response
#[derive(Debug, Clone, Default)]
pub struct TranslateResponse {
    /// Original text
    pub original_text: String,
    /// Translated text
    pub translated_text: String,
    /// Source language
    pub source_lang: String,
    /// Target language
    pub target_lang: String,
    /// Alternative translations
    pub alternatives: Vec<String>,
}

/// Batch translation result
#[derive(Debug, Clone, Default)]
pub struct BatchResult {
    /// Total count
    pub total_count: usize,
    /// Success count
    pub success_count: usize,
    /// Failed count
    pub failed_count: usize,
    /// Results
    pub results: Vec<TranslateResponse>,
    /// Errors
    pub errors: Vec<String>,
    /// Processing time in milliseconds
    pub processing_time: u64,
    /// Total characters processed
    pub total_chars: usize,
    /// Total tokens used (for LLM)
    pub total_tokens: u64,
    /// Average latency in milliseconds
    pub average_latency_ms: f64,
    /// Total number of actual API calls made (batch requests, not translation units)
    /// This should equal the sum of all translator_stats[].total_calls
    pub total_batches: usize,
}

/// Limit policy for rate limiting
#[derive(Debug, Clone)]
pub struct LimitPolicy {
    /// Rate limit (requests per second)
    pub rate_limit: u32,
    /// Maximum character count per request
    pub max_char_count: usize,
    /// Maximum characters per split chunk
    pub split_max_chars: usize,
}

impl LimitPolicy {
    /// Create a new limit policy from character count
    pub fn from_char_count(max_char_count: usize) -> Self {
        Self {
            rate_limit: 10,
            max_char_count,
            split_max_chars: max_char_count * 4 / 5,
        }
    }

    /// Create a new limit policy from token count (converted to characters)
    pub fn from_token_count(max_tokens: usize) -> Self {
        let max_char_count = tokens_to_chars(max_tokens);
        Self {
            rate_limit: 10,
            max_char_count,
            split_max_chars: max_char_count * 4 / 5,
        }
    }
}

impl Default for LimitPolicy {
    fn default() -> Self {
        Self {
            rate_limit: 10,
            max_char_count: 5000,
            split_max_chars: 4000,
        }
    }
}

/// Batch translation options
#[derive(Debug, Clone)]
pub struct BatchOptions {
    /// Rate limit (requests per second)
    pub rate_limit: u32,
    /// Number of concurrent workers
    pub workers: usize,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Limit policy
    pub limit_policy: Option<LimitPolicy>,
    /// Batch size for translation API calls
    pub batch_size: usize,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            rate_limit: 10,
            workers: 5,
            max_retries: 3,
            limit_policy: None,
            batch_size: 50,
        }
    }
}

/// Configuration for DeepLX translator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeepLXConfig {
    /// API URL
    pub api_url: String,
    /// API Key
    pub api_key: Option<String>,
    /// Proxy URL
    pub proxy_url: Option<String>,
    /// Maximum retry attempts
    pub max_retries: usize,
}

/// Configuration for LLM translator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Base URL for API
    pub base_url: String,
    /// API Key
    pub api_key: String,
    /// Model name
    pub model: String,
    /// Maximum tokens
    pub max_tokens: i32,
    /// Temperature
    pub temperature: f64,
    /// Top P
    pub top_p: Option<f64>,
    /// Proxy URL
    pub proxy_url: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Extra headers
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
    /// Extra parameters
    pub extra_params: Option<serde_json::Value>,
}

impl LLMConfig {
    /// Calculate maximum input characters based on max_tokens
    /// Reserves 30% for prompt template and output
    pub fn max_input_chars(&self) -> usize {
        if self.max_tokens <= 0 {
            return 4000;
        }
        let available_tokens = (self.max_tokens as f64 * 0.7) as usize;
        tokens_to_chars(available_tokens)
    }
}

/// Configuration for Tencent translator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TencentConfig {
    /// Secret ID
    pub secret_id: String,
    /// Secret Key
    pub secret_key: String,
    /// Region
    pub region: String,
    /// Project ID
    pub project_id: i64,
    /// Proxy URL
    pub proxy_url: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Untranslated text patterns
    pub untranslated_text: Vec<String>,
    /// Term repository ID list
    pub term_repo_id_list: Vec<String>,
    /// Sentence repository ID list
    pub sent_repo_id_list: Vec<String>,
}
