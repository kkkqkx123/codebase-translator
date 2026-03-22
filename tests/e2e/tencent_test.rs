//! Tencent Cloud E2E Integration Tests

use std::sync::Arc;

use codebase_translate::translator::{
    create_translator_from_config, BatchOptions, LimitPolicy, ProviderType, TencentConfig,
    TencentTranslator, Translator, TranslatorConfig, TranslatorImpl,
};

use super::init_test_config;

/// Test single translation with Tencent Cloud
#[tokio::test]
async fn test_tencent_single_translation() {
    let global_config = init_test_config();

    // Skip if no credentials configured
    let secret_id = global_config.tencent.secret_id.clone().unwrap_or_default();
    let secret_key = global_config.tencent.secret_key.clone().unwrap_or_default();

    if !super::is_configured(&secret_id) || !super::is_configured(&secret_key) {
        println!("Skipping: No Tencent Cloud credentials configured");
        return;
    }

    let config = TencentConfig {
        secret_id,
        secret_key,
        region: global_config.tencent.region.clone(),
        project_id: 0,
        proxy_url: None,
        timeout: global_config.tencent.max_retries as u64,
        max_retries: global_config.tencent.max_retries as usize,
        untranslated_text: Vec::new(),
        term_repo_id_list: Vec::new(),
        sent_repo_id_list: Vec::new(),
    };

    let translator = TencentTranslator::new(config).expect("Failed to create translator");

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

/// Test batch translation with Tencent Cloud
#[tokio::test]
async fn test_tencent_batch_translation() {
    let global_config = init_test_config();

    let secret_id = global_config.tencent.secret_id.clone().unwrap_or_default();
    let secret_key = global_config.tencent.secret_key.clone().unwrap_or_default();

    if !super::is_configured(&secret_id) || !super::is_configured(&secret_key) {
        println!("Skipping: No Tencent Cloud credentials configured");
        return;
    }

    let config = TencentConfig {
        secret_id,
        secret_key,
        region: global_config.tencent.region.clone(),
        project_id: 0,
        proxy_url: None,
        timeout: global_config.tencent.max_retries as u64,
        max_retries: global_config.tencent.max_retries as usize,
        untranslated_text: Vec::new(),
        term_repo_id_list: Vec::new(),
        sent_repo_id_list: Vec::new(),
    };

    let translator = Arc::new(TranslatorImpl::Tencent(
        TencentTranslator::new(config).expect("Failed to create translator"),
    ));

    let batch_options = BatchOptions {
        rate_limit: global_config.tencent.rate_limit,
        workers: 3,
        max_retries: global_config.tencent.max_retries as usize,
        limit_policy: Some(LimitPolicy::default()),
    };

    let batch_translator =
        codebase_translate::translator::BatchTranslator::new(vec![translator], batch_options);

    let texts = vec![
        "Hello".to_string(),
        "World".to_string(),
        "Test".to_string(),
        "Translation".to_string(),
        "System".to_string(),
    ];

    let result = batch_translator.translate_batch(&texts, "en", "zh").await;
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

/// Test translator factory with Tencent
#[tokio::test]
async fn test_tencent_factory() {
    let global_config = init_test_config();

    let secret_id = global_config.tencent.secret_id.clone().unwrap_or_default();
    let secret_key = global_config.tencent.secret_key.clone().unwrap_or_default();

    if !super::is_configured(&secret_id) || !super::is_configured(&secret_key) {
        println!("Skipping: No Tencent Cloud credentials configured");
        return;
    }

    let config = TranslatorConfig {
        provider: ProviderType::Tencent,
        deeplx: None,
        llm: None,
        tencent: Some(TencentConfig {
            secret_id,
            secret_key,
            region: global_config.tencent.region.clone(),
            project_id: 0,
            proxy_url: None,
            timeout: global_config.tencent.max_retries as u64,
            max_retries: global_config.tencent.max_retries as usize,
            untranslated_text: Vec::new(),
            term_repo_id_list: Vec::new(),
            sent_repo_id_list: Vec::new(),
        }),
    };

    let translator = create_translator_from_config(&config);
    assert!(
        translator.is_ok(),
        "Failed to create translator: {:?}",
        translator.err()
    );

    let translator = translator.expect("Translator should be created successfully");
    assert_eq!(translator.name(), "tencent");

    let texts = vec!["Hello".to_string()];
    let result: Result<Vec<String>, _> = translator.translate(&texts, "en", "zh").await;
    assert!(result.is_ok());
}

/// Test empty text handling
#[tokio::test]
async fn test_tencent_empty_text() {
    let global_config = init_test_config();

    let secret_id = global_config.tencent.secret_id.clone().unwrap_or_default();
    let secret_key = global_config.tencent.secret_key.clone().unwrap_or_default();

    if !super::is_configured(&secret_id) || !super::is_configured(&secret_key) {
        println!("Skipping: No Tencent Cloud credentials configured");
        return;
    }

    let config = TencentConfig {
        secret_id,
        secret_key,
        region: global_config.tencent.region.clone(),
        project_id: 0,
        proxy_url: None,
        timeout: global_config.tencent.max_retries as u64,
        max_retries: global_config.tencent.max_retries as usize,
        untranslated_text: Vec::new(),
        term_repo_id_list: Vec::new(),
        sent_repo_id_list: Vec::new(),
    };

    let translator = TencentTranslator::new(config).expect("Failed to create translator");

    let texts = vec!["".to_string()];
    let result = translator.translate(&texts, "", "zh").await;

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

/// Test error handling with invalid credentials
#[tokio::test]
async fn test_tencent_invalid_credentials() {
    let config = TencentConfig {
        secret_id: "invalid_id".to_string(),
        secret_key: "invalid_key".to_string(),
        region: "ap-beijing".to_string(),
        project_id: 0,
        proxy_url: None,
        timeout: 30,
        max_retries: 1,
        untranslated_text: Vec::new(),
        term_repo_id_list: Vec::new(),
        sent_repo_id_list: Vec::new(),
    };

    let translator = TencentTranslator::new(config).expect("Failed to create translator");

    let texts = vec!["Hello".to_string()];
    let result = translator.translate(&texts, "en", "zh").await;

    // Should fail with invalid credentials
    assert!(result.is_err(), "Expected error with invalid credentials");
    println!("Expected error occurred: {:?}", result.err());
}

/// Test API signature generation
#[test]
fn test_tencent_signature_generation() {
    let config = TencentConfig {
        secret_id: "test_secret_id".to_string(),
        secret_key: "test_secret_key".to_string(),
        region: "ap-beijing".to_string(),
        project_id: 0,
        proxy_url: None,
        timeout: 30,
        max_retries: 3,
        untranslated_text: Vec::new(),
        term_repo_id_list: Vec::new(),
        sent_repo_id_list: Vec::new(),
    };

    let translator = TencentTranslator::new(config).expect("Failed to create translator");

    // Test that the translator was created successfully
    // The signature generation is tested indirectly through the translate method
    assert_eq!(translator.name(), "tencent");
}
