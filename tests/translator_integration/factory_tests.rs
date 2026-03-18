//! Factory Module Integration Tests
//!
//! Tests for the translator factory that creates translator instances
//! based on configuration. Validates proper creation of all provider types
//! and error handling for invalid configurations.

use codebase_translate::translator::{
    create_translator_from_config, DeepLXConfig, LLMConfig, ProviderType, TencentConfig,
    TranslatorConfig, TranslatorImpl,
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
    assert!(result.is_ok(), "Factory should create DeepLX translator with custom config");

    let translator = result.expect("Should get translator");
    assert_eq!(translator.name(), "deeplx");
}

/// Test factory creates LLM translator correctly
#[test]
fn test_factory_creates_llm_translator() {
    let config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: Some(LLMConfig {
            base_url: "http://localhost:11434".to_string(),
            api_key: "test-api-key".to_string(),
            model: "llama2".to_string(),
            max_tokens: 2048,
            temperature: 0.3,
            top_p: None,
            proxy_url: None,
            timeout: 30,
            max_retries: 3,
            extra_headers: None,
            extra_params: None,
        }),
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(result.is_ok(), "Factory should create LLM translator");

    let translator = result.expect("Should get translator");
    assert_eq!(translator.name(), "llm");
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

/// Test factory returns error for missing LLM config
#[test]
fn test_factory_fails_without_llm_config() {
    let config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: None,
        ..Default::default()
    };

    let result = create_translator_from_config(&config);
    assert!(result.is_err(), "Factory should fail without LLM config");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("LLM configuration is required"));
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
    assert!(result.is_err(), "Factory should fail without Tencent config");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Tencent configuration is required"));
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

    // LLM (with config)
    let llm_config = TranslatorConfig {
        provider: ProviderType::LLM,
        llm: Some(LLMConfig {
            base_url: "http://localhost:11434".to_string(),
            api_key: "test".to_string(),
            model: "llama2".to_string(),
            max_tokens: 2048,
            temperature: 0.3,
            top_p: None,
            proxy_url: None,
            timeout: 30,
            max_retries: 3,
            extra_headers: None,
            extra_params: None,
        }),
        ..Default::default()
    };
    assert!(create_translator_from_config(&llm_config).is_ok());

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

/// Test TranslatorImpl from_config for LLM
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

    let result = TranslatorImpl::from_config(&config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "llm");
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
