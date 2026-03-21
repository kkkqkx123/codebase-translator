//! Translation Service Integration Tests
//!
//! Tests for the synchronous translation service that manages async operations.
//! Validates service creation, batch processing, and resource management.

use std::sync::Arc;

use codebase_translate::translator::{
    BatchOptions, BatchTranslationService, DeepLXConfig, ProviderType, TencentConfig,
    TranslationService, TranslatorConfig, TranslatorImpl,
};

/// Test TranslationService creation with DeepLX
#[test]
fn test_translation_service_creation_deeplx() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let result = TranslationService::new(config);
    assert!(result.is_ok(), "Should create translation service");

    let service = result.expect("Should get service");
    assert!(service.name().contains("deeplx"));
}

/// Test TranslationService creation with LLM
#[test]
fn test_translation_service_creation_llm() {
    let config = TranslatorConfig {
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

    let result = TranslationService::new(config);
    assert!(result.is_ok(), "Should create translation service with LLM");

    let service = result.expect("Should get service");
    assert!(service.name().contains("llm"));
}

/// Test TranslationService creation with Tencent
#[test]
fn test_translation_service_creation_tencent() {
    let config = TranslatorConfig {
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

    let result = TranslationService::new(config);
    assert!(result.is_ok(), "Should create translation service with Tencent");

    let service = result.expect("Should get service");
    assert!(service.name().contains("tencent"));
}

/// Test BatchTranslationService creation
#[test]
fn test_batch_translation_service_creation() {
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
    let result = BatchTranslationService::new(vec![(translator, 50)], options);

    assert!(result.is_ok(), "Should create batch translation service");
}

/// Test BatchTranslationService with custom options
#[test]
fn test_batch_translation_service_custom_options() {
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
        limit_policy: Some(codebase_translate::translator::LimitPolicy::from_char_count(2000)),
    };
    let result = BatchTranslationService::new(vec![(translator, 50)], options);

    assert!(result.is_ok(), "Should create batch translation service with custom options");
}

/// Test TranslationService with all provider types
#[test]
fn test_translation_service_all_providers() {
    // DeepLX
    let deeplx_config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };
    let deeplx_service = TranslationService::new(deeplx_config);
    assert!(deeplx_service.is_ok());
    assert_eq!(deeplx_service.unwrap().name(), "deeplx");

    // LLM
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
    let llm_service = TranslationService::new(llm_config);
    assert!(llm_service.is_ok());
    assert_eq!(llm_service.unwrap().name(), "llm");

    // Tencent
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
    let tencent_service = TranslationService::new(tencent_config);
    assert!(tencent_service.is_ok());
    assert_eq!(tencent_service.unwrap().name(), "tencent");
}

/// Test TranslationService max_input_chars
#[test]
fn test_translation_service_max_input_chars() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let service = TranslationService::new(config).expect("Should create service");

    // DeepLX returns 0 for max_input_chars (no specific limit)
    assert_eq!(service.max_input_chars(), 0);
}

/// Test TranslationService can_handle
#[test]
fn test_translation_service_can_handle() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let service = TranslationService::new(config).expect("Should create service");

    // DeepLX can handle any text length
    assert!(service.can_handle(100));
    assert!(service.can_handle(10000));
}

/// Test TranslationService is_available (without actual API call)
#[test]
fn test_translation_service_is_available() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let service = TranslationService::new(config).expect("Should create service");

    // Note: This will return false because there's no actual DeepLX server running
    // but it tests that the method works without panicking
    let _available = service.is_available();
}

/// Test BatchTranslationService with different translator types
#[test]
fn test_batch_translation_service_different_types() {
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
    let service = BatchTranslationService::new(deeplx_translator, options);
    assert!(service.is_ok());

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
    let service = BatchTranslationService::new(llm_translator, options);
    assert!(service.is_ok());
}

/// Test TranslationService drop (cleanup)
#[test]
fn test_translation_service_drop() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    {
        let service = TranslationService::new(config).expect("Should create service");
        assert!(service.name().contains("deeplx"));
        // Service will be dropped here
    }

    // If we reach here without panic, drop worked correctly
    assert!(true);
}

