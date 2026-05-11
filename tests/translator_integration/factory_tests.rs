//! Factory Module Integration Tests
//!
//! Tests for the translator factory that creates translator instances
//! based on configuration. Validates proper creation of all provider types
//! and error handling for invalid configurations.

use codebase_translate::config::global::{GlobalConfig, LLMProviderConfig};
use codebase_translate::translator::{
    create_llm_multi_provider_translator, create_translator_from_config, DeepLXConfig, LLMConfig,
    ProviderType, TencentConfig, Translator, TranslatorConfig, TranslatorImpl,
};

/// Test factory creates DeepLX translator correctly with default config
#[test]
fn test_factory_creates_deeplx_translator() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(result.is_ok(), "Factory should create DeepLX translator");

    let translator = result.expect("Should get translator");
    assert_eq!(translator.name(), "deeplx");
}

/// Test factory creates DeepLX translator with custom configuration
#[test]
fn test_factory_creates_deeplx_with_custom_config() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig {
            api_url: "http://custom:1188".to_string(),
            api_key: Some("test-key".to_string()),
            proxy_url: Some("http://proxy:8080".to_string()),
            max_retries: 5,
        }),
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(
        result.is_ok(),
        "Factory should create DeepLX translator with custom config"
    );

    let translator = result.expect("Should get translator");
    assert_eq!(translator.name(), "deeplx");
}

/// Test factory creates LLM translator correctly using multi-provider API
#[test]
fn test_factory_creates_llm_translator() {
    use codebase_translate::config::global::LLMGlobalConfig;
    use std::collections::HashMap;

    let global_config = GlobalConfig {
        llm: LLMGlobalConfig {
            providers: vec![LLMProviderConfig {
                id: "test-provider".to_string(),
                name: "Test Provider".to_string(),
                base_url: "http://localhost:11434".to_string(),
                api_keys: vec!["test-api-key".to_string()],
                model: "llama2".to_string(),
                model_list: vec![],
                max_tokens: 2048,
                temperature: 0.3,
                proxy_url: None,
                timeout: 30,
                rate_limit: 10,
                extra_headers: HashMap::new(),
                extra_params: HashMap::new(),
                custom_system_prompt: None,
                custom_user_prompt: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let result = create_llm_multi_provider_translator(&global_config);
    assert!(result.is_ok(), "Factory should create LLM translator");

    let translator = result.expect("Should get translator");
    assert_eq!(translator.name(), "llm-multi-provider");
}

/// Test factory creates Tencent translator correctly
#[test]
fn test_factory_creates_tencent_translator() {
    let config = TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: Some(TencentConfig {
            secret_id: "test-secret-id".to_string(),
            secret_key: "test-secret-key".to_string(),
            region: "ap-beijing".to_string(),
            project_id: 12345,
            proxy_url: None,
            timeout: 30,
            max_retries: 3,
            untranslated_text: vec![],
            term_repo_id_list: vec![],
            sent_repo_id_list: vec![],
        }),
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(result.is_ok(), "Factory should create Tencent translator");

    let translator = result.expect("Should get translator");
    assert_eq!(translator.name(), "tencent");
}

/// Test factory returns error for missing LLM providers
#[test]
fn test_factory_fails_without_llm_config() {
    use codebase_translate::config::global::LLMGlobalConfig;
    let global_config = GlobalConfig {
        llm: LLMGlobalConfig {
            providers: vec![], // Empty providers list
            ..Default::default()
        },
        ..Default::default()
    };

    let result = create_llm_multi_provider_translator(&global_config);
    assert!(result.is_err(), "Factory should fail without LLM providers");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("No valid LLM providers found"));
}

/// Test factory returns error for missing Tencent config
#[test]
fn test_factory_fails_without_tencent_config() {
    let config = TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: None,
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(
        result.is_err(),
        "Factory should fail without Tencent config"
    );

    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("Tencent configuration is required"));
}

/// Test factory creates all provider types
#[test]
fn test_factory_all_provider_types() {
    // DeepLX
    let deeplx_config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };
    assert!(create_translator_from_config(&deeplx_config).is_ok());

    // LLM (with config) - using multi-provider API
    use codebase_translate::config::global::LLMGlobalConfig;
    let global_config = GlobalConfig {
        llm: LLMGlobalConfig {
            providers: vec![LLMProviderConfig {
                id: "test-provider".to_string(),
                name: "Test Provider".to_string(),
                base_url: "http://localhost:11434".to_string(),
                api_keys: vec!["test".to_string()],
                model: "llama2".to_string(),
                model_list: vec![],
                max_tokens: 2048,
                temperature: 0.3,
                proxy_url: None,
                timeout: 30,
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
    assert!(create_llm_multi_provider_translator(&global_config).is_ok());

    // Tencent (with config)
    let tencent_config = TranslatorConfig {
        provider: ProviderType::Tencent,
        tencent: Some(TencentConfig {
            secret_id: "test_id".to_string(),
            secret_key: "test_key".to_string(),
            region: "ap-beijing".to_string(),
            project_id: 0,
            proxy_url: None,
            timeout: 30,
            max_retries: 3,
            untranslated_text: vec![],
            term_repo_id_list: vec![],
            sent_repo_id_list: vec![],
        }),
        ..Default::default()
    };
    assert!(create_translator_from_config(&tencent_config).is_ok());
}

/// Test TranslatorImpl from_config for DeepLX
#[test]
fn test_translator_impl_from_config_deeplx() {
    let config = TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(DeepLXConfig::default()),
        ..Default::default()
    };

    let result = TranslatorImpl::from_config(&config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "deeplx");
}

/// Test TranslatorImpl from_config for LLM - now returns error directing to use multi-provider API
#[test]
fn test_translator_impl_from_config_llm() {
    let config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: Some(LLMConfig {
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

    // TranslatorImpl::from_config now returns error for LLM, directing to use create_llm_multi_provider_translator
    let result = TranslatorImpl::from_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("create_llm_multi_provider_translator"));
}

/// Test TranslatorImpl from_config for Tencent
#[test]
fn test_translator_impl_from_config_tencent() {
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

    let result = TranslatorImpl::from_config(&config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "tencent");
}

/// Test factory error propagation returns correct error type
#[test]
fn test_factory_error_propagation() {
    let config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: None,
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_string = err.to_string();
    assert!(err_string.contains("LLM") || err_string.contains("configuration"));
}
