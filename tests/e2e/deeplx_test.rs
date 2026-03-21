//! DeepLX E2E Integration Tests

use std::sync::Arc;

use codebase_translate::translator::{
    create_translator_from_config, BatchOptions, DeepLXConfig, DeepLXTranslator, LimitPolicy,
    ProviderType, Translator, TranslatorConfig, TranslatorImpl,
};

use super::init_test_config;

/// Test single translation with DeepLX
#[tokio::test]
async fn test_deeplx_single_translation() {
    let global_config = init_test_config();

    // Skip if no DeepLX URL configured
    if !super::is_configured(&global_config.deeplx.api_url) {
        println!("Skipping: No DeepLX URL configured");
        return;
    }

    let config = DeepLXConfig {
        api_url: global_config.deeplx.api_url.clone(),
        api_key: global_config.deeplx.api_key.clone(),
        proxy_url: global_config.deeplx.proxy_url.clone(),
        max_retries: global_config.deeplx.max_retries as usize,
    };

    let translator = DeepLXTranslator::new(config).expect("Failed to create translator");

    // Test English to Chinese
    let texts = vec!["Hello, world!".to_string()];
    let result = translator.translate(&texts, "ZH").await;

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
    let result = translator.translate(&texts, "EN").await;

    assert!(result.is_ok(), "Translation failed: {:?}", result.err());
    let translated = result.expect("Translation should succeed");
    assert!(!translated[0].is_empty());
    println!("Chinese -> English: '{}' -> '{}'", texts[0], translated[0]);
}

/// Test batch translation with DeepLX
#[tokio::test]
async fn test_deeplx_batch_translation() {
    let global_config = init_test_config();

    if !super::is_configured(&global_config.deeplx.api_url) {
        println!("Skipping: No DeepLX URL configured");
        return;
    }

    let config = DeepLXConfig {
        api_url: global_config.deeplx.api_url.clone(),
        api_key: global_config.deeplx.api_key.clone(),
        proxy_url: global_config.deeplx.proxy_url.clone(),
        max_retries: global_config.deeplx.max_retries as usize,
    };

    let translator = Arc::new(TranslatorImpl::DeepLX(
        DeepLXTranslator::new(config).expect("Failed to create translator"),
    ));

    let batch_options = BatchOptions {
        rate_limit: global_config.deeplx.rate_limit,
        workers: 5,
        max_retries: global_config.deeplx.max_retries as usize,
        limit_policy: Some(LimitPolicy::default()),
    };

    let batch_translator =
        codebase_translate::translator::BatchTranslator::new(vec![(translator, 50)], batch_options);

    let texts = vec![
        "Hello".to_string(),
        "World".to_string(),
        "Test".to_string(),
        "Translation".to_string(),
        "System".to_string(),
    ];

    let result = batch_translator.translate_batch(&texts, "ZH").await;
    assert!(
        result.is_ok(),
        "Batch translation failed: {:?}",
        result.err()
    );

    let batch_result = result.expect("Batch translation should succeed");
    assert_eq!(batch_result.total_count, texts.len());
    assert_eq!(batch_result.results.len(), texts.len());

    println!("Batch translation completed:");
    println!("  Total: {}", batch_result.total_count);
    println!("  Success: {}", batch_result.success_count);
    println!("  Failed: {}", batch_result.failed_count);
    println!("  Time: {}ms", batch_result.processing_time);

    for (i, response) in batch_result.results.iter().enumerate() {
        println!(
            "  {}: '{}' -> '{}'",
            i, response.original_text, response.translated_text
        );
    }
}

/// Test translator factory with DeepLX
#[tokio::test]
async fn test_deeplx_factory() {
    let global_config = init_test_config();

    if !super::is_configured(&global_config.deeplx.api_url) {
        println!("Skipping: No DeepLX URL configured");
        return;
    }

    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig {
            api_url: global_config.deeplx.api_url.clone(),
            api_key: global_config.deeplx.api_key.clone(),
            proxy_url: global_config.deeplx.proxy_url.clone(),
            max_retries: global_config.deeplx.max_retries as usize,
        }),
        llm: None,
        tencent: None,
    };

    let translator = create_translator_from_config(&config);
    assert!(
        translator.is_ok(),
        "Failed to create translator: {:?}",
        translator.err()
    );

    let translator = translator.expect("Translator should be created successfully");
    assert_eq!(translator.name(), "deeplx");

    let texts = vec!["Hello".to_string()];
    let result: Result<Vec<String>, _> = translator.translate(&texts, "ZH").await;
    assert!(result.is_ok());
}

/// Test empty text handling
#[tokio::test]
async fn test_deeplx_empty_text() {
    let global_config = init_test_config();

    if !super::is_configured(&global_config.deeplx.api_url) {
        println!("Skipping: No DeepLX URL configured");
        return;
    }

    let config = DeepLXConfig {
        api_url: global_config.deeplx.api_url.clone(),
        api_key: global_config.deeplx.api_key.clone(),
        proxy_url: global_config.deeplx.proxy_url.clone(),
        max_retries: global_config.deeplx.max_retries as usize,
    };

    let translator = DeepLXTranslator::new(config).expect("Failed to create translator");

    let texts = vec!["".to_string()];
    let result = translator.translate(&texts, "ZH").await;

    // Empty text should either return empty or error gracefully
    match result {
        Ok(translated) => {
            assert!(translated.is_empty() || translated[0].is_empty());
        }
        Err(e) => {
            println!("Empty text returned error (expected): {}", e);
        }
    }
}
