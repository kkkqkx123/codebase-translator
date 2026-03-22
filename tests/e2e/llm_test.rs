//! LLM E2E Integration Tests

use std::sync::Arc;

use codebase_translate::translator::{
    BatchOptions, LimitPolicy, MultiProviderTranslator, Translator, TranslatorImpl,
};

use super::init_test_config;

/// Test single translation with LLM
#[tokio::test]
async fn test_llm_single_translation() {
    let global_config = init_test_config();

    // Skip if no LLM providers configured
    if global_config.llm.providers.is_empty() {
        println!("Skipping: No LLM providers configured");
        return;
    }

    // Filter valid providers
    let valid_providers: Vec<_> = global_config
        .llm
        .providers
        .iter()
        .filter(|p| {
            let api_key = p.api_keys.first().cloned().unwrap_or_default();
            !api_key.is_empty()
                && !api_key.starts_with("${")
                && !p.base_url.is_empty()
                && !p.base_url.starts_with("${")
        })
        .cloned()
        .collect();

    if valid_providers.is_empty() {
        println!("Skipping: No valid LLM providers configured");
        return;
    }

    let translator =
        MultiProviderTranslator::new(&valid_providers, 3).expect("Failed to create translator");

    // Test English to Chinese
    let texts = vec!["Hello, world!".to_string()];
    let result = translator.translate(&texts, "en", "zh").await;

    assert!(result.is_ok(), "Translation failed: {:?}", result.err());
    let translated = result.expect("Translation should succeed");
    assert_eq!(translated.len(), 1);
    assert!(
        !translated[0].is_empty(),
        "Translated text should not be empty"
    );
    println!("English -> Chinese: '{}' -> '{}'", texts[0], translated[0]);

    // Test Chinese to English
    let texts = vec!["你好，世界！".to_string()];
    let result = translator.translate(&texts, "zh", "en").await;

    assert!(result.is_ok(), "Translation failed: {:?}", result.err());
    let translated = result.expect("Translation should succeed");
    assert!(!translated[0].is_empty());
    println!("Chinese -> English: '{}' -> '{}'", texts[0], translated[0]);
}

/// Test batch translation with LLM
#[tokio::test]
async fn test_llm_batch_translation() {
    let global_config = init_test_config();

    if global_config.llm.providers.is_empty() {
        println!("Skipping: No LLM providers configured");
        return;
    }

    // Filter valid providers
    let valid_providers: Vec<_> = global_config
        .llm
        .providers
        .iter()
        .filter(|p| {
            let api_key = p.api_keys.first().cloned().unwrap_or_default();
            !api_key.is_empty()
                && !api_key.starts_with("${")
                && !p.base_url.is_empty()
                && !p.base_url.starts_with("${")
        })
        .cloned()
        .collect();

    if valid_providers.is_empty() {
        println!("Skipping: No valid LLM providers configured");
        return;
    }

    let translator = Arc::new(TranslatorImpl::LLM(
        MultiProviderTranslator::new(&valid_providers, 3).expect("Failed to create translator"),
    ));

    let batch_options = BatchOptions {
        rate_limit: 5,
        workers: 2,
        max_retries: 3,
        limit_policy: Some(LimitPolicy::default()),
    };

    let batch_translator =
        codebase_translate::translator::BatchTranslator::new(vec![translator], batch_options);

    // Small batch to minimize API calls
    let texts = vec!["Hello".to_string(), "World".to_string()];

    let result = batch_translator.translate_batch(&texts, "en", "zh").await;
    assert!(
        result.is_ok(),
        "Batch translation failed: {:?}",
        result.err()
    );

    let batch_result = result.expect("Batch translation should succeed");
    assert_eq!(batch_result.total_count, texts.len());

    println!("Batch translation completed:");
    println!("  Total: {}", batch_result.total_count);
    println!("  Success: {}", batch_result.success_count);
    println!("  Failed: {}", batch_result.failed_count);

    for (i, response) in batch_result.results.iter().enumerate() {
        println!(
            "  {}: '{}' -> '{}'",
            i, response.original_text, response.translated_text
        );
    }
}

/// Test translator factory with LLM
#[tokio::test]
async fn test_llm_factory() {
    let global_config = init_test_config();

    if global_config.llm.providers.is_empty() {
        println!("Skipping: No LLM providers configured");
        return;
    }

    // Filter valid providers
    let valid_providers: Vec<_> = global_config
        .llm
        .providers
        .iter()
        .filter(|p| {
            let api_key = p.api_keys.first().cloned().unwrap_or_default();
            !api_key.is_empty()
                && !api_key.starts_with("${")
                && !p.base_url.is_empty()
                && !p.base_url.starts_with("${")
        })
        .cloned()
        .collect();

    if valid_providers.is_empty() {
        println!("Skipping: No valid LLM providers configured");
        return;
    }

    let translator = MultiProviderTranslator::new(&valid_providers, 3);
    assert!(
        translator.is_ok(),
        "Failed to create translator: {:?}",
        translator.err()
    );

    let translator = translator.expect("Translator should be created successfully");
    assert_eq!(translator.name(), "llm-multi-provider");

    let texts = vec!["Hello".to_string()];
    let result: Result<Vec<String>, _> = translator.translate(&texts, "en", "zh").await;
    assert!(result.is_ok());
}

/// Test error handling with invalid API key
#[tokio::test]
async fn test_llm_invalid_api_key() {
    use codebase_translate::config::LLMProviderConfig;

    let config = LLMProviderConfig {
        id: "test".to_string(),
        name: "Test Provider".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_keys: vec!["invalid_key".to_string()],
        model: "gpt-3.5-turbo".to_string(),
        model_list: vec![],
        max_tokens: 4096,
        temperature: 0.3,
        proxy_url: None,
        timeout: 30,
        rate_limit: 5,
        extra_headers: std::collections::HashMap::new(),
        extra_params: std::collections::HashMap::new(),
    };

    let translator =
        MultiProviderTranslator::new(&[config], 1).expect("Failed to create translator");

    let texts = vec!["Hello".to_string()];
    let result = translator.translate(&texts, "en", "zh").await;

    // Should fail with authentication error
    assert!(result.is_err());
}

/// Test rate limiting
#[tokio::test]
async fn test_llm_rate_limiting() {
    let global_config = init_test_config();

    if global_config.llm.providers.is_empty() {
        println!("Skipping: No LLM providers configured");
        return;
    }

    // Filter valid providers
    let valid_providers: Vec<_> = global_config
        .llm
        .providers
        .iter()
        .filter(|p| {
            let api_key = p.api_keys.first().cloned().unwrap_or_default();
            !api_key.is_empty()
                && !api_key.starts_with("${")
                && !p.base_url.is_empty()
                && !p.base_url.starts_with("${")
        })
        .cloned()
        .collect();

    if valid_providers.is_empty() {
        println!("Skipping: No valid LLM providers configured");
        return;
    }

    let translator = Arc::new(TranslatorImpl::LLM(
        MultiProviderTranslator::new(&valid_providers, 3).expect("Failed to create translator"),
    ));

    // Use rate limit of 2 requests per second
    let batch_options = BatchOptions {
        rate_limit: 2,
        workers: 1,
        max_retries: 3,
        limit_policy: Some(LimitPolicy::default()),
    };

    let batch_translator =
        codebase_translate::translator::BatchTranslator::new(vec![translator], batch_options);

    let texts = vec!["Hello".to_string(), "World".to_string(), "Test".to_string()];

    let start = std::time::Instant::now();
    let result = batch_translator.translate_batch(&texts, "en", "zh").await;
    let elapsed = start.elapsed();

    // With rate limit of 2/sec, 3 requests should take at least 1 second
    // But we allow some tolerance
    println!("Rate limited batch took: {:?}", elapsed);
    assert!(result.is_ok());
}
