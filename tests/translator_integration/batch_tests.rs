//! Batch Translator Integration Tests
//!
//! Tests for the batch translator with rate limiting and retry logic.
//! Validates batch processing, rate limiting, and result aggregation.

use std::sync::Arc;

use codebase_translate::translator::{
    create_batch_translator, BatchOptions, BatchResult, BatchTranslator, DeepLXConfig,
    LimitPolicy, ProviderType, TranslatorConfig, TranslatorImpl,
};

/// Test BatchTranslator creation with default options
#[test]
fn test_batch_translator_creation() {
    let config = DeepLXConfig::default();
    let translator = Arc::new(
        TranslatorImpl::from_config(&TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(config),
            ..Default::default()
        })
        .expect("Should create translator"),
    );

    let options = BatchOptions::default();
    let batch = BatchTranslator::new(translator, options);

    assert_eq!(batch.name(), "deeplx");
}

/// Test create_batch_translator helper function
#[test]
fn test_create_batch_translator_helper() {
    let config = DeepLXConfig::default();
    let translator = Arc::new(
        TranslatorImpl::from_config(&TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(config),
            ..Default::default()
        })
        .expect("Should create translator"),
    );

    let options = BatchOptions::default();
    let batch = create_batch_translator(translator, options);

    assert_eq!(batch.name(), "deeplx");
}

/// Test BatchTranslator with custom options
#[test]
fn test_batch_translator_custom_options() {
    let config = DeepLXConfig::default();
    let translator = Arc::new(
        TranslatorImpl::from_config(&TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(config),
            ..Default::default()
        })
        .expect("Should create translator"),
    );

    let options = BatchOptions {
        rate_limit: 5,
        workers: 3,
        max_retries: 2,
        limit_policy: Some(LimitPolicy::from_char_count(3000)),
    };
    let batch = BatchTranslator::new(translator, options);

    assert_eq!(batch.name(), "deeplx");
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

/// Test BatchOptions default values
#[test]
fn test_batch_options_default() {
    let options = BatchOptions::default();
    assert_eq!(options.rate_limit, 10);
    assert_eq!(options.workers, 5);
    assert_eq!(options.max_retries, 3);
    assert!(options.limit_policy.is_none());
}

/// Test BatchOptions custom values
#[test]
fn test_batch_options_custom() {
    let options = BatchOptions {
        rate_limit: 20,
        workers: 10,
        max_retries: 5,
        limit_policy: Some(LimitPolicy::default()),
    };
    assert_eq!(options.rate_limit, 20);
    assert_eq!(options.workers, 10);
    assert_eq!(options.max_retries, 5);
    assert!(options.limit_policy.is_some());
}

/// Test BatchOptions clone
#[test]
fn test_batch_options_clone() {
    let options = BatchOptions {
        rate_limit: 15,
        workers: 7,
        max_retries: 4,
        limit_policy: Some(LimitPolicy::from_char_count(2000)),
    };
    let cloned = options.clone();
    assert_eq!(options.rate_limit, cloned.rate_limit);
    assert_eq!(options.workers, cloned.workers);
    assert_eq!(options.max_retries, cloned.max_retries);
}

/// Test LimitPolicy default values
#[test]
fn test_limit_policy_default() {
    let policy = LimitPolicy::default();
    assert_eq!(policy.rate_limit, 10);
    assert_eq!(policy.max_char_count, 5000);
    assert_eq!(policy.split_max_chars, 4000);
}

/// Test LimitPolicy from_char_count
#[test]
fn test_limit_policy_from_char_count() {
    let policy = LimitPolicy::from_char_count(10000);
    assert_eq!(policy.max_char_count, 10000);
    assert_eq!(policy.split_max_chars, 8000); // 80% of max
    assert_eq!(policy.rate_limit, 10);
}

/// Test LimitPolicy from_token_count
#[test]
fn test_limit_policy_from_token_count() {
    let policy = LimitPolicy::from_token_count(1000);
    assert_eq!(policy.max_char_count, 1500); // 1000 * 1.5
    assert_eq!(policy.split_max_chars, 1200); // 80% of max
    assert_eq!(policy.rate_limit, 10);
}

/// Test LimitPolicy clone
#[test]
fn test_limit_policy_clone() {
    let policy = LimitPolicy::from_char_count(3000);
    let cloned = policy.clone();
    assert_eq!(policy.rate_limit, cloned.rate_limit);
    assert_eq!(policy.max_char_count, cloned.max_char_count);
    assert_eq!(policy.split_max_chars, cloned.split_max_chars);
}

/// Test BatchResult with data
#[test]
fn test_batch_result_with_data() {
    use codebase_translate::translator::common::TranslateResponse;

    let result = BatchResult {
        total_count: 10,
        success_count: 8,
        failed_count: 2,
        results: vec![
            TranslateResponse {
                original_text: "Hello".to_string(),
                translated_text: "你好".to_string(),
                source_lang: "EN".to_string(),
                target_lang: "ZH".to_string(),
                alternatives: vec![],
            },
        ],
        errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        processing_time: 1500,
        total_chars: 100,
        total_tokens: 50,
        average_latency_ms: 150.0,
    };

    assert_eq!(result.total_count, 10);
    assert_eq!(result.success_count, 8);
    assert_eq!(result.failed_count, 2);
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.errors.len(), 2);
    assert_eq!(result.processing_time, 1500);
    assert_eq!(result.total_chars, 100);
    assert_eq!(result.total_tokens, 50);
    assert_eq!(result.average_latency_ms, 150.0);
}

/// Test BatchTranslator with different translator types
#[test]
fn test_batch_translator_with_different_types() {
    // DeepLX
    let deeplx_config = DeepLXConfig::default();
    let deeplx_translator = Arc::new(
        TranslatorImpl::from_config(&TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(deeplx_config),
            ..Default::default()
        })
        .expect("Should create translator"),
    );

    let options = BatchOptions::default();
    let batch = BatchTranslator::new(deeplx_translator, options);
    assert_eq!(batch.name(), "deeplx");

    // LLM
    let llm_config = codebase_translate::translator::LLMConfig {
        base_url: "http://localhost".to_string(),
        api_key: "test".to_string(),
        model: "test".to_string(),
        max_tokens: 100,
        temperature: 0.5,
        top_p: None,
        proxy_url: None,
        timeout: 10,
        max_retries: 3,
        extra_headers: None,
        extra_params: None,
    };
    let llm_translator = Arc::new(
        TranslatorImpl::from_config(&TranslatorConfig {
            provider: ProviderType::LLM,
            llm: Some(llm_config),
            ..Default::default()
        })
        .expect("Should create translator"),
    );

    let options = BatchOptions::default();
    let batch = BatchTranslator::new(llm_translator, options);
    assert_eq!(batch.name(), "llm");
}

/// Test BatchResult clone
#[test]
fn test_batch_result_clone() {
    let result = BatchResult {
        total_count: 5,
        success_count: 4,
        failed_count: 1,
        results: vec![],
        errors: vec!["error".to_string()],
        processing_time: 1000,
        total_chars: 50,
        total_tokens: 25,
        average_latency_ms: 200.0,
    };

    let cloned = result.clone();
    assert_eq!(result.total_count, cloned.total_count);
    assert_eq!(result.success_count, cloned.success_count);
    assert_eq!(result.failed_count, cloned.failed_count);
    assert_eq!(result.processing_time, cloned.processing_time);
}

/// Test BatchResult debug format
#[test]
fn test_batch_result_debug() {
    let result = BatchResult::default();
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("BatchResult"));
    assert!(debug_str.contains("total_count"));
    assert!(debug_str.contains("success_count"));
}

/// Test TranslateResponse default
#[test]
fn test_translate_response_default() {
    let response = codebase_translate::translator::common::TranslateResponse::default();
    assert!(response.original_text.is_empty());
    assert!(response.translated_text.is_empty());
    assert!(response.source_lang.is_empty());
    assert!(response.target_lang.is_empty());
    assert!(response.alternatives.is_empty());
}

/// Test TranslateResponse with data
#[test]
fn test_translate_response_with_data() {
    use codebase_translate::translator::common::TranslateResponse;

    let response = TranslateResponse {
        original_text: "Hello world".to_string(),
        translated_text: "你好世界".to_string(),
        source_lang: "EN".to_string(),
        target_lang: "ZH".to_string(),
        alternatives: vec!["您好世界".to_string()],
    };

    assert_eq!(response.original_text, "Hello world");
    assert_eq!(response.translated_text, "你好世界");
    assert_eq!(response.source_lang, "EN");
    assert_eq!(response.target_lang, "ZH");
    assert_eq!(response.alternatives.len(), 1);
}
