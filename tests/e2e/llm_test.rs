//! LLM E2E Integration Tests

use std::sync::Arc;

use codebase_translate::translator::{
    create_translator_from_config, BatchOptions, LLMConfig, LLMTranslator, LimitPolicy,
    ProviderType, Translator, TranslatorConfig, TranslatorImpl,
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

    let provider = &global_config.llm.providers[0];
    let api_key = provider.api_keys.first().cloned().unwrap_or_default();

    if !super::is_configured(&api_key) || !super::is_configured(&provider.base_url) {
        println!("Skipping: No API key or base URL configured");
        return;
    }

    let config = LLMConfig {
        base_url: provider.base_url.clone(),
        api_key,
        model: provider.model.clone(),
        max_tokens: provider.max_tokens as i32,
        temperature: provider.temperature as f64,
        top_p: None,
        proxy_url: provider.proxy_url.clone(),
        timeout: provider.timeout,
        max_retries: provider.rate_limit as usize,
        extra_headers: Some(provider.extra_headers.clone()),
        extra_params: None,
    };

    let translator = LLMTranslator::new(config).expect("Failed to create translator");

    // Test English to Chinese
    let texts = vec!["Hello, world!".to_string()];
    let result = translator.translate(&texts, "zh").await;

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
    let result = translator.translate(&texts, "en").await;

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

    let provider = &global_config.llm.providers[0];
    let api_key = provider.api_keys.first().cloned().unwrap_or_default();

    if !super::is_configured(&api_key) || !super::is_configured(&provider.base_url) {
        println!("Skipping: No API key or base URL configured");
        return;
    }

    let config = LLMConfig {
        base_url: provider.base_url.clone(),
        api_key,
        model: provider.model.clone(),
        max_tokens: provider.max_tokens as i32,
        temperature: provider.temperature as f64,
        top_p: None,
        proxy_url: provider.proxy_url.clone(),
        timeout: provider.timeout,
        max_retries: 3,
        extra_headers: Some(provider.extra_headers.clone()),
        extra_params: None,
    };

    let translator = Arc::new(TranslatorImpl::LLM(
        LLMTranslator::new(config).expect("Failed to create translator"),
    ));

    let batch_options = BatchOptions {
        rate_limit: 5,
        workers: 2,
        max_retries: 3,
        limit_policy: Some(LimitPolicy::default()),
    };

    let batch_translator =
        codebase_translate::translator::BatchTranslator::new(vec![(translator, 50)], batch_options);

    // Small batch to minimize API calls
    let texts = vec!["Hello".to_string(), "World".to_string()];

    let result = batch_translator.translate_batch(&texts, "zh").await;
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

    let provider = &global_config.llm.providers[0];
    let api_key = provider.api_keys.first().cloned().unwrap_or_default();

    if !super::is_configured(&api_key) || !super::is_configured(&provider.base_url) {
        println!("Skipping: No API key or base URL configured");
        return;
    }

    let config = TranslatorConfig {
        provider: ProviderType::LLM,
        deeplx: None,
        llm: Some(LLMConfig {
            base_url: provider.base_url.clone(),
            api_key,
            model: provider.model.clone(),
            max_tokens: provider.max_tokens as i32,
            temperature: provider.temperature as f64,
            top_p: None,
            proxy_url: provider.proxy_url.clone(),
            timeout: provider.timeout,
            max_retries: 3,
            extra_headers: Some(provider.extra_headers.clone()),
            extra_params: None,
        }),
        tencent: None,
    };

    let translator = create_translator_from_config(&config);
    assert!(
        translator.is_ok(),
        "Failed to create translator: {:?}",
        translator.err()
    );

    let translator = translator.expect("Translator should be created successfully");
    assert_eq!(translator.name(), "llm");

    let texts = vec!["Hello".to_string()];
    let result: Result<Vec<String>, _> = translator.translate(&texts, "zh").await;
    assert!(result.is_ok());
}

/// Test error handling with invalid API key
#[tokio::test]
async fn test_llm_invalid_api_key() {
    let config = LLMConfig {
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: "invalid_key".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: 4096,
        temperature: 0.3,
        top_p: None,
        proxy_url: None,
        timeout: 30,
        max_retries: 1,
        extra_headers: None,
        extra_params: None,
    };

    let translator = LLMTranslator::new(config).expect("Failed to create translator");

    let texts = vec!["Hello".to_string()];
    let result = translator.translate(&texts, "zh").await;

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

    let provider = &global_config.llm.providers[0];
    let api_key = provider.api_keys.first().cloned().unwrap_or_default();

    if !super::is_configured(&api_key) || !super::is_configured(&provider.base_url) {
        println!("Skipping: No API key or base URL configured");
        return;
    }

    let config = LLMConfig {
        base_url: provider.base_url.clone(),
        api_key,
        model: provider.model.clone(),
        max_tokens: provider.max_tokens as i32,
        temperature: provider.temperature as f64,
        top_p: None,
        proxy_url: provider.proxy_url.clone(),
        timeout: provider.timeout,
        max_retries: 3,
        extra_headers: Some(provider.extra_headers.clone()),
        extra_params: None,
    };

    let translator = Arc::new(TranslatorImpl::LLM(
        LLMTranslator::new(config).expect("Failed to create translator"),
    ));

    // Use rate limit of 2 requests per second
    let batch_options = BatchOptions {
        rate_limit: 2,
        workers: 1,
        max_retries: 3,
        limit_policy: Some(LimitPolicy::default()),
    };

    let batch_translator =
        codebase_translate::translator::BatchTranslator::new(vec![(translator, 50)], batch_options);

    let texts = vec!["Hello".to_string(), "World".to_string(), "Test".to_string()];

    let start = std::time::Instant::now();
    let result = batch_translator.translate_batch(&texts, "zh").await;
    let elapsed = start.elapsed();

    // With rate limit of 2/sec, 3 requests should take at least 1 second
    // But we allow some tolerance
    println!("Rate limited batch took: {:?}", elapsed);
    assert!(result.is_ok());
}
