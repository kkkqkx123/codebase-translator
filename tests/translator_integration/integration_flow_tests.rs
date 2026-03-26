//! Integration Flow Tests
//!
//! End-to-end integration tests that verify the complete flow between
//! factory, batch translator, and service components.

use std::sync::Arc;

use codebase_translate::translator::{
    create_batch_translator, create_translator_from_config, BatchOptions, DeepLXConfig,
    ProviderType, TencentConfig, TranslationService, Translator, TranslatorConfig,
};

/// Test complete flow: Config -> Factory -> TranslatorImpl -> BatchTranslator
#[test]
fn test_config_to_batch_flow() {
    // Step 1: Create config
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    // Step 2: Create translator via factory
    let translator = create_translator_from_config(&config).expect("Factory should work");
    assert_eq!(translator.name(), "deeplx");

    // Step 3: Wrap in Arc for sharing
    let translator_arc = Arc::new(translator);

    // Step 4: Create batch translator
    let options = BatchOptions {
        rate_limit: 10,
        workers: 5,
        max_retries: 3,
        limit_policy: Some(codebase_translate::translator::LimitPolicy::default()),
        batch_size: 50,
    };
    let batch = create_batch_translator(vec![translator_arc], options);

    assert!(batch.name().contains("deeplx"));
}

/// Test complete flow: Config -> TranslationService
#[test]
fn test_config_to_service_flow() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let service = TranslationService::new(config).expect("Should create service");
    assert!(service.name().contains("deeplx"));
}

/// Test all provider types through factory
#[test]
fn test_all_providers_through_factory() {
    // Test DeepLX
    let deeplx_config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };
    let deeplx = create_translator_from_config(&deeplx_config).expect("Should create DeepLX");
    assert_eq!(deeplx.name(), "deeplx");

    // Test LLM
    let llm_config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: Some(codebase_translate::translator::LLMConfig {
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
        }),
        ..Default::default()
    };
    let llm = create_translator_from_config(&llm_config);
    // LLM should fail with current design as it requires MultiProviderTranslator
    assert!(llm.is_err());

    // Test Tencent
    let tencent_config = TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: Some(TencentConfig {
            secret_id: "test".to_string(),
            secret_key: "test".to_string(),
            region: "ap-beijing".to_string(),
            project_id: 0,
            proxy_url: None,
            timeout: 10,
            max_retries: 3,
            untranslated_text: vec![],
            term_repo_id_list: vec![],
            sent_repo_id_list: vec![],
        }),
        ..Default::default()
    };
    let tencent = create_translator_from_config(&tencent_config).expect("Should create Tencent");
    assert_eq!(tencent.name(), "tencent");
}

/// Test batch translator with multiple providers
#[test]
fn test_batch_translator_multiple_providers() {
    let deeplx = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create DeepLX");

    let tencent = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: Some(TencentConfig {
            secret_id: "test".to_string(),
            secret_key: "test".to_string(),
            region: "ap-beijing".to_string(),
            project_id: 0,
            proxy_url: None,
            timeout: 10,
            max_retries: 3,
            untranslated_text: vec![],
            term_repo_id_list: vec![],
            sent_repo_id_list: vec![],
        }),
        ..Default::default()
    })
    .expect("Should create Tencent");

    let translators = vec![Arc::new(deeplx), Arc::new(tencent)];

    let options = BatchOptions {
        rate_limit: 10,
        workers: 5,
        max_retries: 3,
        limit_policy: Some(codebase_translate::translator::LimitPolicy::default()),
        batch_size: 50,
    };

    let batch = create_batch_translator(translators, options);

    let name = batch.name();
    assert!(name.contains("deeplx") || name.contains("tencent"));
}

/// Test TranslationService methods
#[test]
fn test_translation_service_methods() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let service = TranslationService::new(config).expect("Should create service");

    // Test max_input_chars
    let max_chars = service.max_input_chars();
    assert!(max_chars > 0);

    // Test can_handle
    assert!(service.can_handle(100));
    assert!(service.can_handle(max_chars));
    assert!(!service.can_handle(max_chars + 1));
}

/// Test BatchOptions with all fields
#[test]
fn test_batch_options_all_fields() {
    let options = BatchOptions {
        rate_limit: 20,
        workers: 10,
        max_retries: 5,
        limit_policy: Some(codebase_translate::translator::LimitPolicy {
            rate_limit: 20,
            max_char_count: 5000,
            split_max_chars: 4000,
        }),
        batch_size: 100,
    };

    assert_eq!(options.rate_limit, 20);
    assert_eq!(options.workers, 10);
    assert_eq!(options.max_retries, 5);
    assert_eq!(options.batch_size, 100);
    assert!(options.limit_policy.is_some());
}

/// Test BatchOptions default
#[test]
fn test_batch_options_default() {
    let options = BatchOptions::default();

    assert_eq!(options.rate_limit, 10);
    assert_eq!(options.workers, 5);
    assert_eq!(options.max_retries, 3);
    assert_eq!(options.batch_size, 50);
}
