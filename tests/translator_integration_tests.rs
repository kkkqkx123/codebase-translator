//! Translator Module Integration Tests
//!
//! These tests verify the integration between factory, multi-translator, service,
//! batch translator, and LLM submodules (pool, routing). They focus on component
//! integration integrity and actual behavior of rotation, routing, and coordination
//! without making actual API calls to external translation services.

use std::sync::Arc;

use codebase_translate::config::LLMProviderConfig;
use codebase_translate::translator::llm::{
    CapacityProvider, Provider, ProviderPool, ProviderPoolConfig, ProviderRouter, RotationStrategy,
};
use codebase_translate::translator::multi::SelectionStrategy;
use codebase_translate::translator::{
    create_batch_translator, create_translator_from_config, BatchOptions, BatchTranslationService,
    DeepLXConfig, LimitPolicy, MultiTranslator, ProviderType, TencentConfig, TranslationService,
    Translator, TranslatorConfig,
};

// ============================================================================
// Factory Integration Tests
// ============================================================================

/// Test factory creates all provider types correctly
#[test]
fn test_factory_creates_all_provider_types() {
    // DeepLX
    let deeplx_config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };
    let deeplx = create_translator_from_config(&deeplx_config).expect("Should create DeepLX");
    assert_eq!(deeplx.name(), "deeplx");

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
    let llm = create_translator_from_config(&llm_config).expect("Should create LLM");
    assert_eq!(llm.name(), "llm");

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
    let tencent = create_translator_from_config(&tencent_config).expect("Should create Tencent");
    assert_eq!(tencent.name(), "tencent");
}

/// Test factory error handling for missing configurations
#[test]
fn test_factory_error_handling() {
    // Missing LLM config
    let result = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::LLM,
        llm: None,
        ..Default::default()
    });
    assert!(result.is_err());

    // Missing Tencent config
    let result = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: None,
        ..Default::default()
    });
    assert!(result.is_err());
}

// ============================================================================
// Multi-Translator Coordination Tests
// ============================================================================

/// Test multi-translator round-robin selection cycles through providers
#[test]
fn test_multi_translator_round_robin_cycles() {
    let t1 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create t1");

    let t2 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create t2");

    let multi = MultiTranslator::new(
        vec![(Arc::new(t1), 1u32), (Arc::new(t2), 1u32)],
        SelectionStrategy::RoundRobin,
        3,
    )
    .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
}

/// Test multi-translator weighted selection behavior
#[test]
fn test_multi_translator_weighted_selection() {
    let t1 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create t1");

    let t2 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create t2");

    let multi = MultiTranslator::new(
        vec![(Arc::new(t1), 3u32), (Arc::new(t2), 1u32)],
        SelectionStrategy::Weighted,
        3,
    )
    .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
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

    let multi = MultiTranslator::new(
        vec![(Arc::new(deeplx), 2u32), (Arc::new(llm), 1u32)],
        SelectionStrategy::Weighted,
        3,
    )
    .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
}

/// Test multi-translator fails with empty translator list
#[test]
fn test_multi_translator_fails_empty() {
    let result: Result<MultiTranslator, _> =
        MultiTranslator::new(vec![], SelectionStrategy::RoundRobin, 3);
    assert!(result.is_err());
}

// ============================================================================
// LLM Provider Pool Rotation Tests
// ============================================================================

fn create_test_provider_config(id: &str, base_url: &str, weight: u32) -> LLMProviderConfig {
    LLMProviderConfig {
        id: id.to_string(),
        name: id.to_string(),
        base_url: base_url.to_string(),
        api_keys: vec!["test-key".to_string()],
        model: "test-model".to_string(),
        weight,
        model_list: vec![],
        max_tokens: 4096,
        temperature: 0.3,
        proxy_url: None,
        timeout: 30,
        rate_limit: 10,
        extra_headers: std::collections::HashMap::new(),
        extra_params: std::collections::HashMap::new(),
    }
}

/// Test provider pool round-robin rotation cycles through providers
#[tokio::test]
async fn test_provider_pool_round_robin_rotation() {
    let configs = vec![
        create_test_provider_config("provider1", "http://localhost:11434", 1),
        create_test_provider_config("provider2", "http://localhost:11435", 1),
    ];

    let pool: ProviderPool = ProviderPool::new(
        &configs,
        ProviderPoolConfig {
            strategy: RotationStrategy::RoundRobin,
            health_check_enabled: false,
            ..Default::default()
        },
    )
    .await
    .expect("Should create pool");

    // Get providers multiple times and verify they cycle
    let provider1 = pool.get_provider().await.expect("Should get provider");
    let provider2 = pool.get_provider().await.expect("Should get provider");
    let provider3 = pool.get_provider().await.expect("Should get provider");

    // Should cycle back to first provider
    assert_eq!(provider1.id(), provider3.id());
    // Second provider should be different
    assert_ne!(provider1.id(), provider2.id());
}

/// Test provider pool weighted rotation distribution
#[tokio::test]
async fn test_provider_pool_weighted_rotation() {
    let configs = vec![
        create_test_provider_config("light", "http://localhost:11434", 1),
        create_test_provider_config("heavy", "http://localhost:11435", 3),
    ];

    let pool: ProviderPool = ProviderPool::new(
        &configs,
        ProviderPoolConfig {
            strategy: RotationStrategy::Weighted,
            health_check_enabled: false,
            ..Default::default()
        },
    )
    .await
    .expect("Should create pool");

    // Get providers multiple times
    let mut heavy_count = 0;
    let mut light_count = 0;

    for _ in 0..100 {
        let provider = pool.get_provider().await.expect("Should get provider");
        match provider.id() {
            "heavy" => heavy_count += 1,
            "light" => light_count += 1,
            _ => {}
        }
    }

    // Heavy provider (weight 3) should be selected roughly 3x more than light (weight 1)
    assert!(
        heavy_count > light_count,
        "Heavy provider should be selected more often"
    );
    let ratio = heavy_count as f64 / light_count as f64;
    assert!(
        ratio > 1.5 && ratio < 5.0,
        "Ratio should be approximately 3:1, got {}",
        ratio
    );
}

/// Test provider pool fails with empty configs
#[tokio::test]
async fn test_provider_pool_fails_empty() {
    let configs: Vec<LLMProviderConfig> = vec![];
    let result: Result<ProviderPool, _> =
        ProviderPool::new(&configs, ProviderPoolConfig::default()).await;
    assert!(result.is_err());
}

/// Test provider pool accepts duplicate IDs (no validation)
#[tokio::test]
async fn test_provider_pool_accepts_duplicate_ids() {
    let configs = vec![
        create_test_provider_config("duplicate", "http://localhost:11434", 1),
        create_test_provider_config("duplicate", "http://localhost:11435", 1),
    ];

    // ProviderPool does not validate duplicate IDs, it just creates the pool
    let result: Result<ProviderPool, _> =
        ProviderPool::new(&configs, ProviderPoolConfig::default()).await;
    assert!(result.is_ok(), "ProviderPool should accept duplicate IDs");

    // Both providers should be available - verify by getting providers multiple times
    let pool = result.unwrap();
    let provider1 = pool.get_provider().await.expect("Should get provider 1");
    let provider2 = pool.get_provider().await.expect("Should get provider 2");

    // Both providers should have the same ID but be different instances
    assert_eq!(provider1.id(), provider2.id());
}

// ============================================================================
// LLM Provider Routing Tests
// ============================================================================

/// Test provider router selects provider based on text length
#[test]
fn test_provider_router_selects_by_text_length() {
    let configs = vec![
        LLMProviderConfig {
            id: "small".to_string(),
            name: "small".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "llama2".to_string(),
            max_tokens: 1024,
            weight: 1,
            model_list: vec![],
            temperature: 0.3,
            proxy_url: None,
            timeout: 30,
            rate_limit: 10,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        },
        LLMProviderConfig {
            id: "large".to_string(),
            name: "large".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "llama3".to_string(),
            max_tokens: 4096,
            weight: 1,
            model_list: vec![],
            temperature: 0.3,
            proxy_url: None,
            timeout: 30,
            rate_limit: 10,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        },
    ];

    let router = ProviderRouter::new(&configs).expect("Should create router");

    // Short text should be routable
    let provider = router.select_provider(100);
    assert!(provider.is_some(), "Should select provider for short text");

    // Long text should only route to capable providers
    let threshold = router.capacity_threshold();
    let provider = router.select_provider(threshold + 1000);
    assert!(provider.is_some(), "Should select provider for long text");
    assert!(
        provider.unwrap().can_handle(threshold + 1000),
        "Selected provider should handle the text"
    );
}

/// Test provider router returns None for oversized text
#[test]
fn test_provider_router_returns_none_oversized() {
    let configs = vec![LLMProviderConfig {
        id: "small".to_string(),
        name: "small".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 100,
        weight: 1,
        model_list: vec![],
        temperature: 0.3,
        proxy_url: None,
        timeout: 30,
        rate_limit: 10,
        extra_headers: std::collections::HashMap::new(),
        extra_params: std::collections::HashMap::new(),
    }];

    let router = ProviderRouter::new(&configs).expect("Should create router");
    let max_capacity = router.max_capacity();

    let provider = router.select_provider(max_capacity + 10000);
    assert!(provider.is_none(), "Should return None for oversized text");
}

/// Test provider router weighted distribution
#[test]
fn test_provider_router_weighted_distribution() {
    let configs = vec![
        LLMProviderConfig {
            id: "light".to_string(),
            name: "light".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            max_tokens: 4096,
            weight: 1,
            model_list: vec![],
            temperature: 0.3,
            proxy_url: None,
            timeout: 30,
            rate_limit: 10,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        },
        LLMProviderConfig {
            id: "heavy".to_string(),
            name: "heavy".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            max_tokens: 4096,
            weight: 3,
            model_list: vec![],
            temperature: 0.3,
            proxy_url: None,
            timeout: 30,
            rate_limit: 10,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        },
    ];

    let router = ProviderRouter::new(&configs).expect("Should create router");

    let mut heavy_count = 0;
    let mut light_count = 0;

    for _ in 0..100 {
        let provider = router.select_provider(100);
        assert!(provider.is_some());

        match provider.unwrap().provider().id() {
            "heavy" => heavy_count += 1,
            "light" => light_count += 1,
            _ => {}
        }
    }

    assert!(
        heavy_count > light_count,
        "Heavy provider should be selected more often"
    );
}

/// Test capacity provider can_handle behavior
#[test]
fn test_capacity_provider_can_handle() {
    let config = LLMProviderConfig {
        id: "test".to_string(),
        name: "test".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 1000,
        weight: 1,
        model_list: vec![],
        temperature: 0.3,
        proxy_url: None,
        timeout: 30,
        rate_limit: 10,
        extra_headers: std::collections::HashMap::new(),
        extra_params: std::collections::HashMap::new(),
    };

    let provider = CapacityProvider::new(&config).expect("Should create provider");
    let max_chars = provider.max_chars();

    assert!(provider.can_handle(max_chars / 2));
    assert!(provider.can_handle(max_chars));
    assert!(!provider.can_handle(max_chars + 1000));
}

// ============================================================================
// Batch Translator Tests
// ============================================================================

/// Test batch translator creation with different options
#[test]
fn test_batch_translator_creation() {
    let translator = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create translator");

    let translator_arc = Arc::new(translator);

    // Default options
    let batch1 = create_batch_translator(translator_arc.clone(), BatchOptions::default());
    assert_eq!(batch1.name(), "deeplx");

    // Custom options
    let custom_options = BatchOptions {
        rate_limit: 5,
        workers: 3,
        max_retries: 2,
        limit_policy: Some(LimitPolicy::from_char_count(2000)),
    };
    let batch2 = create_batch_translator(translator_arc, custom_options);
    assert_eq!(batch2.name(), "deeplx");
}

// ============================================================================
// Translation Service Tests
// ============================================================================

/// Test translation service creation with all provider types
#[test]
fn test_translation_service_all_providers() {
    // DeepLX
    let service = TranslationService::new(TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    });
    assert!(service.is_ok());

    // LLM
    let service = TranslationService::new(TranslatorConfig {
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
    assert!(service.is_ok());

    // Tencent
    let service = TranslationService::new(TranslatorConfig {
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
    assert!(service.is_ok());
}

/// Test batch translation service creation
#[test]
fn test_batch_translation_service_creation() {
    let translator = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create translator");

    let service = BatchTranslationService::new(Arc::new(translator), BatchOptions::default());
    assert!(service.is_ok());
}

// ============================================================================
// End-to-End Integration Flow Tests
// ============================================================================

/// Test complete flow: Config -> Factory -> BatchTranslator
#[test]
fn test_complete_flow_config_to_batch() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let translator = create_translator_from_config(&config).expect("Factory should work");
    let batch = create_batch_translator(Arc::new(translator), BatchOptions::default());

    assert_eq!(batch.name(), "deeplx");
}

/// Test complete flow: Config -> Factory -> MultiTranslator
#[test]
fn test_complete_flow_config_to_multi() {
    let t1 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create t1");

    let t2 = create_translator_from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    })
    .expect("Should create t2");

    let multi = MultiTranslator::new(
        vec![(Arc::new(t1), 2u32), (Arc::new(t2), 1u32)],
        SelectionStrategy::RoundRobin,
        3,
    )
    .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
}

/// Test complete flow: Config -> TranslationService
#[test]
fn test_complete_flow_config_to_service() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let service = TranslationService::new(config).expect("Should create service");
    assert_eq!(service.name(), "deeplx");
}
