//! Translation Service Integration Tests
//!
//! Tests for the synchronous translation service that manages async operations.
//! Validates service creation, batch processing, and resource management.

use std::sync::Arc;

use codebase_translate::config::global::{GlobalConfig, LLMProviderConfig};
use codebase_translate::translator::{
    create_llm_multi_provider_translator, BatchOptions, BatchTranslationService, DeepLXConfig,
    ProviderType, TencentConfig, TranslationService, TranslatorConfig, TranslatorImpl,
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

/// Test TranslationService creation with LLM - using multi-provider API
#[test]
fn test_translation_service_creation_llm() {
    use codebase_translate::config::global::LLMGlobalConfig;
    let global_config = GlobalConfig {
        llm: LLMGlobalConfig {
            providers: vec![LLMProviderConfig {
                id: "test-provider".to_string(),
                name: "Test Provider".to_string(),
                base_url: "http://localhost".to_string(),
                api_keys: vec!["test".to_string()],
                model: "test".to_string(),
                model_list: vec![],
                max_tokens: 100,
                temperature: 0.5,
                proxy_url: None,
                timeout: 10,
                rate_limit: 10,
                extra_headers: std::collections::HashMap::new(),
                extra_params: std::collections::HashMap::new(),
                custom_system_prompt: None,
                custom_user_prompt: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let translator =
        create_llm_multi_provider_translator(&global_config).expect("Should create LLM translator");
    let translator_arc = Arc::new(translator);

    let options = BatchOptions::default();
    let result = BatchTranslationService::new(vec![translator_arc], options);
    assert!(result.is_ok(), "Should create translation service with LLM");
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
    assert!(
        result.is_ok(),
        "Should create translation service with Tencent"
    );

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
    let result = BatchTranslationService::new(vec![translator], options);

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
        batch_size: 50,
    };
    let result = BatchTranslationService::new(vec![translator], options);

    assert!(
        result.is_ok(),
        "Should create batch translation service with custom options"
    );
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

    // LLM - using multi-provider API
    use codebase_translate::config::global::LLMGlobalConfig;
    let global_config = GlobalConfig {
        llm: LLMGlobalConfig {
            providers: vec![LLMProviderConfig {
                id: "test-provider".to_string(),
                name: "Test Provider".to_string(),
                base_url: "http://localhost".to_string(),
                api_keys: vec!["test".to_string()],
                model: "test".to_string(),
                model_list: vec![],
                max_tokens: 100,
                temperature: 0.5,
                proxy_url: None,
                timeout: 10,
                rate_limit: 10,
                extra_headers: std::collections::HashMap::new(),
                extra_params: std::collections::HashMap::new(),
                custom_system_prompt: None,
                custom_user_prompt: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let llm_translator =
        create_llm_multi_provider_translator(&global_config).expect("Should create LLM translator");
    let llm_translator_arc = Arc::new(llm_translator);
    let options = BatchOptions::default();
    let llm_service = BatchTranslationService::new(vec![llm_translator_arc], options);
    assert!(llm_service.is_ok());

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

    // DeepLX returns 5000 for max_input_chars
    assert_eq!(service.max_input_chars(), 5000);
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

    // DeepLX can handle any text length (returns true for any input)
    assert!(service.can_handle(100));
    // Note: DeepLX service returns 0 for max_input_chars, meaning no specific limit
    // so can_handle always returns true
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
    let service = BatchTranslationService::new(vec![deeplx_translator], options);
    assert!(service.is_ok());

    // LLM - using multi-provider API
    use codebase_translate::config::global::LLMGlobalConfig;
    let global_config = GlobalConfig {
        llm: LLMGlobalConfig {
            providers: vec![LLMProviderConfig {
                id: "test-provider".to_string(),
                name: "Test Provider".to_string(),
                base_url: "http://localhost".to_string(),
                api_keys: vec!["test".to_string()],
                model: "test".to_string(),
                model_list: vec![],
                max_tokens: 100,
                temperature: 0.5,
                proxy_url: None,
                timeout: 10,
                rate_limit: 10,
                extra_headers: std::collections::HashMap::new(),
                extra_params: std::collections::HashMap::new(),
                custom_system_prompt: None,
                custom_user_prompt: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let llm_translator = Arc::new(
        create_llm_multi_provider_translator(&global_config).expect("Should create LLM translator"),
    );

    let options = BatchOptions::default();
    let service = BatchTranslationService::new(vec![llm_translator], options);
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
}
