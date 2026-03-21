//! Common Module Integration Tests
//!
//! Tests for common types and utilities, focusing on conversion
//! functions and configuration structures behavior.

use codebase_translate::translator::common::{
    chars_to_tokens, tokens_to_chars, BatchOptions, BatchResult, DeepLXConfig, LimitPolicy,
    LLMConfig, TencentConfig, TranslateRequest, TranslateResponse,
};

/// Test character to token conversion ratio
#[test]
fn test_chars_to_tokens_conversion() {
    // 1 token ≈ 1.5 characters
    assert_eq!(chars_to_tokens(0), 0);
    assert_eq!(chars_to_tokens(1), 1);
    assert_eq!(chars_to_tokens(15), 10);
    assert_eq!(chars_to_tokens(150), 100);
    assert_eq!(chars_to_tokens(1500), 1000);
}

/// Test token to character conversion ratio
#[test]
fn test_tokens_to_chars_conversion() {
    // 1 token ≈ 1.5 characters
    assert_eq!(tokens_to_chars(0), 0);
    assert_eq!(tokens_to_chars(10), 15);
    assert_eq!(tokens_to_chars(100), 150);
    assert_eq!(tokens_to_chars(1000), 1500);
}

/// Test LimitPolicy creation from character count
#[test]
fn test_limit_policy_from_char_count() {
    let policy = LimitPolicy::from_char_count(5000);
    assert_eq!(policy.max_char_count, 5000);
    assert_eq!(policy.split_max_chars, 4000); // 80% of max
    assert_eq!(policy.rate_limit, 10);

    let policy = LimitPolicy::from_char_count(10000);
    assert_eq!(policy.max_char_count, 10000);
    assert_eq!(policy.split_max_chars, 8000);
}

/// Test LimitPolicy creation from token count
#[test]
fn test_limit_policy_from_token_count() {
    let policy = LimitPolicy::from_token_count(1000);
    assert_eq!(policy.max_char_count, 1500); // 1000 * 1.5
    assert_eq!(policy.split_max_chars, 1200); // 80% of max
    assert_eq!(policy.rate_limit, 10);
}

/// Test LLMConfig max_input_chars calculation
#[test]
fn test_llm_config_max_input_chars() {
    // Reserves 30% for prompt template and output
    let config = LLMConfig {
        max_tokens: 1000,
        ..Default::default()
    };
    let expected = ((1000.0 * 0.7) as usize * 15) / 10; // 70% of tokens * 1.5
    assert_eq!(config.max_input_chars(), expected);

    // Zero max_tokens returns default
    let config = LLMConfig {
        max_tokens: 0,
        ..Default::default()
    };
    assert_eq!(config.max_input_chars(), 4000);

    // Negative max_tokens returns default
    let config = LLMConfig {
        max_tokens: -100,
        ..Default::default()
    };
    assert_eq!(config.max_input_chars(), 4000);
}

/// Test BatchOptions default values
#[test]
fn test_batch_options_default() {
    let options = BatchOptions::default();
    assert_eq!(options.rate_limit, 10);
    assert_eq!(options.workers, 5);
    assert_eq!(options.max_retries, 3);
    assert!(options.limit_policy.is_none());
}

/// Test LimitPolicy default values
#[test]
fn test_limit_policy_default() {
    let policy = LimitPolicy::default();
    assert_eq!(policy.rate_limit, 10);
    assert_eq!(policy.max_char_count, 5000);
    assert_eq!(policy.split_max_chars, 4000);
}

/// Test TranslateRequest creation
#[test]
fn test_translate_request_creation() {
    let request = TranslateRequest {
        text: "Hello world".to_string(),
        source_lang: "EN".to_string(),
        target_lang: "ZH".to_string(),
    };

    assert_eq!(request.text, "Hello world");
    assert_eq!(request.source_lang, "EN");
    assert_eq!(request.target_lang, "ZH");
}

/// Test TranslateResponse creation and defaults
#[test]
fn test_translate_response_default() {
    let response = TranslateResponse::default();
    assert!(response.original_text.is_empty());
    assert!(response.translated_text.is_empty());
    assert!(response.source_lang.is_empty());
    assert!(response.target_lang.is_empty());
    assert!(response.alternatives.is_empty());
}

/// Test BatchResult default values
#[test]
fn test_batch_result_default() {
    let result = BatchResult::default();
    assert_eq!(result.total_count, 0);
    assert_eq!(result.success_count, 0);
    assert_eq!(result.failed_count, 0);
    assert!(result.results.is_empty());
    assert!(result.errors.is_empty());
    assert_eq!(result.processing_time, 0);
    assert_eq!(result.total_chars, 0);
    assert_eq!(result.total_tokens, 0);
    assert_eq!(result.average_latency_ms, 0.0);
}

/// Test DeepLXConfig default values
#[test]
fn test_deeplx_config_default() {
    let config = DeepLXConfig::default();
    assert!(config.api_url.is_empty());
    assert!(config.api_key.is_none());
    assert!(config.proxy_url.is_none());
    assert_eq!(config.max_retries, 0);
}

/// Test TencentConfig default values
#[test]
fn test_tencent_config_default() {
    let config = TencentConfig::default();
    assert!(config.secret_id.is_empty());
    assert!(config.secret_key.is_empty());
    assert!(config.region.is_empty());
    assert_eq!(config.project_id, 0);
    assert!(config.proxy_url.is_none());
    assert_eq!(config.timeout, 0);
    assert_eq!(config.max_retries, 0);
    assert!(config.untranslated_text.is_empty());
    assert!(config.term_repo_id_list.is_empty());
    assert!(config.sent_repo_id_list.is_empty());
}

/// Test conversion roundtrip
#[test]
fn test_conversion_roundtrip() {
    let original_chars = 1500;
    let tokens = chars_to_tokens(original_chars);
    let back_to_chars = tokens_to_chars(tokens);

    // Due to rounding, values may differ slightly
    assert!(back_to_chars >= original_chars);
    assert!(back_to_chars <= original_chars + 15); // Within 1.5 char tolerance
}

