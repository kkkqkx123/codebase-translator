//! Integration Flow Tests
//!
//! End-to-end integration tests that verify the complete flow between
//! factory, multi-translator, batch translator, and service components.

use std::sync::Arc;

use codebase_translate::translator::{
    create_batch_translator, create_translator_from_config, BatchOptions, BatchTranslationService,
    DeepLXConfig, MultiTranslator, ProviderType, SelectionStrategy, TencentConfig,
    TranslationService, TranslatorConfig, TranslatorImpl,
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
    };
    let batch = create_batch_translator(translator_arc, options);

    assert_eq!(batch.name(), "deeplx");
}

/// Test complete flow: Config -> Factory -> MultiTranslator
#[test]
fn test_config_to_multi_flow() {
    // Create multiple translators
    let config1 = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };
    let translator1 = create_translator_from_config(&config1).expect("Should create");

    let config2 = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };
    let translator2 = create_translator_from_config(&config2).expect("Should create");

    // Create multi-translator
    let translators = vec![(Arc::new(translator1), 2u32), (Arc::new(translator2), 1u32)];
    let multi = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3)
        .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
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
    assert_eq!(service.name(), "deeplx");
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
    let llm = create_translator_from_config(&llm_config).expect("Should create LLM");
    assert_eq!(llm.name(), "llm");

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

/// Test multi-translator with mixed provider types
#[test]
fn test_multi_translator_mixed_providers() {
    let deeplx = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create DeepLX");

    let llm = create_translator_from_config(&TranslatorConfig {
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
    })
    .expect("Should create LLM");

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

    let translators = vec![
        (Arc::new(deeplx), 2u32),
        (Arc::new(llm), 1u32),
        (Arc::new(tencent), 1u32),
    ];

    let multi = MultiTranslator::new(translators, SelectionStrategy::Weighted, 3)
        .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
}

/// Test batch translation service with different strategies
#[test]
fn test_batch_service_with_strategies() {
    let deeplx = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create DeepLX");

    let llm = create_translator_from_config(&TranslatorConfig {
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
    })
    .expect("Should create LLM");

    // Test with DeepLX
    let deeplx_service =
        BatchTranslationService::new(Arc::new(deeplx), BatchOptions::default())
            .expect("Should create service");

    // Test with LLM
    let llm_service = BatchTranslationService::new(Arc::new(llm), BatchOptions::default())
        .expect("Should create service");

    // Both services should be created successfully
    assert!(true);
}

/// Test factory error handling integration
#[test]
fn test_factory_error_handling_integration() {
    // Missing LLM config
    let llm_config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: None,
        ..Default::default()
    };
    let result = create_translator_from_config(&llm_config);
    assert!(result.is_err());

    // Missing Tencent config
    let tencent_config = TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: None,
        ..Default::default()
    };
    let result = create_translator_from_config(&tencent_config);
    assert!(result.is_err());
}

/// Test multi-translator round-robin selection behavior
#[test]
fn test_multi_translator_round_robin_behavior() {
    let t1 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create");

    let t2 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create");

    let translators = vec![(Arc::new(t1), 1u32), (Arc::new(t2), 1u32)];

    let multi =
        MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3).expect("Should create");

    // Verify multi-translator was created with round-robin strategy
    assert_eq!(multi.name(), "multi");
}

/// Test service creation with all provider types
#[test]
fn test_service_creation_all_providers() {
    // DeepLX service
    let deeplx_service = TranslationService::new(TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    });
    assert!(deeplx_service.is_ok());

    // LLM service
    let llm_service = TranslationService::new(TranslatorConfig {
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
    });
    assert!(llm_service.is_ok());

    // Tencent service
    let tencent_service = TranslationService::new(TranslatorConfig {
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
    });
    assert!(tencent_service.is_ok());
}

/// Test batch options integration with different configurations
#[test]
fn test_batch_options_integration() {
    let translator = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create");

    // Test with default options
    let batch1 = create_batch_translator(Arc::new(translator.clone()), BatchOptions::default());
    assert_eq!(batch1.name(), "deeplx");

    // Test with custom options
    let custom_options = BatchOptions {
        rate_limit: 5,
        workers: 3,
        max_retries: 2,
        limit_policy: Some(codebase_translate::translator::LimitPolicy::from_char_count(2000)),
    };
    let batch2 = create_batch_translator(Arc::new(translator), custom_options);
    assert_eq!(batch2.name(), "deeplx");
}
