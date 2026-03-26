//! Multi-Translator Integration Tests
//!
//! Tests for the multi-translator with load balancing and failover.
//! Validates MultiProviderTranslator functionality for LLM providers.

use codebase_translate::config::LLMProviderConfig;
use codebase_translate::translator::{MultiProviderTranslator, SelectionStrategy, Translator};

/// Helper function to create test LLM provider configs
fn create_test_llm_configs() -> Vec<LLMProviderConfig> {
    vec![
        LLMProviderConfig {
            id: "provider1".to_string(),
            name: "Provider 1".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "llama2".to_string(),
            max_tokens: 2048,
            rate_limit: 1,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "provider2".to_string(),
            name: "Provider 2".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "llama3".to_string(),
            max_tokens: 4096,
            rate_limit: 2,
            ..Default::default()
        },
    ]
}

/// Test MultiProviderTranslator creation fails with empty config list
#[test]
fn test_multi_translator_fails_with_empty_configs() {
    let configs: Vec<LLMProviderConfig> = vec![];
    let result = MultiProviderTranslator::new(&configs, 3);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("At least one LLM provider configuration is required"));
}

/// Test MultiProviderTranslator creation with single provider
#[test]
fn test_multi_translator_with_single_provider() {
    let configs = vec![LLMProviderConfig {
        id: "provider1".to_string(),
        name: "Provider 1".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key1".to_string()],
        model: "llama2".to_string(),
        max_tokens: 2048,
        rate_limit: 1,
        ..Default::default()
    }];

    let result = MultiProviderTranslator::new(&configs, 3);
    assert!(result.is_ok());

    let multi = result.expect("Should create multi-translator");
    assert_eq!(multi.name(), "llm-multi-provider");
}

/// Test MultiProviderTranslator creation with multiple providers
#[test]
fn test_multi_translator_with_multiple_providers() {
    let configs = create_test_llm_configs();
    let result = MultiProviderTranslator::new(&configs, 3);

    assert!(result.is_ok());
    let multi = result.expect("Should create multi-translator");
    assert_eq!(multi.name(), "llm-multi-provider");
}

/// Test MultiProviderTranslator creation skips invalid providers
#[test]
fn test_multi_translator_skips_invalid_providers() {
    let configs = vec![
        LLMProviderConfig {
            id: "invalid".to_string(),
            name: "Invalid Provider".to_string(),
            base_url: "".to_string(), // Empty base_url
            api_keys: vec!["key".to_string()],
            model: "test".to_string(),
            ..Default::default()
        },
        LLMProviderConfig {
            id: "valid".to_string(),
            name: "Valid Provider".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key".to_string()],
            model: "llama2".to_string(),
            max_tokens: 2048,
            rate_limit: 1,
            ..Default::default()
        },
    ];

    let result = MultiProviderTranslator::new(&configs, 3);
    assert!(result.is_ok());
    let multi = result.expect("Should create multi-translator skipping invalid provider");
    assert_eq!(multi.name(), "llm-multi-provider");
}

/// Test MultiProviderTranslator fails when all providers are invalid
#[test]
fn test_multi_translator_fails_all_invalid() {
    let configs = vec![
        LLMProviderConfig {
            id: "invalid1".to_string(),
            name: "Invalid Provider 1".to_string(),
            base_url: "".to_string(),
            api_keys: vec!["key".to_string()],
            model: "test".to_string(),
            ..Default::default()
        },
        LLMProviderConfig {
            id: "invalid2".to_string(),
            name: "Invalid Provider 2".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec![], // Empty api_keys
            model: "test".to_string(),
            ..Default::default()
        },
    ];

    let result = MultiProviderTranslator::new(&configs, 3);
    assert!(result.is_err());
}

/// Test MultiProviderTranslator max_retries validation
#[test]
fn test_multi_translator_max_retries_zero() {
    let configs = vec![LLMProviderConfig {
        id: "provider1".to_string(),
        name: "Provider 1".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key1".to_string()],
        model: "llama2".to_string(),
        max_tokens: 2048,
        rate_limit: 1,
        ..Default::default()
    }];

    // max_retries = 0 should be treated as 3 (default)
    let result = MultiProviderTranslator::new(&configs, 0);
    assert!(result.is_ok());
}

/// Test SelectionStrategy variants
#[test]
fn test_selection_strategy_variants() {
    // Test RoundRobin variant exists
    let _ = SelectionStrategy::RoundRobin;
    // Test RateBasedRandom variant exists
    let _ = SelectionStrategy::RateBasedRandom;
    // Test SmoothRateBasedRoundRobin variant exists
    let _ = SelectionStrategy::SmoothRateBasedRoundRobin;
}

/// Test SelectionStrategy default
#[test]
fn test_selection_strategy_default() {
    let default = SelectionStrategy::default();
    assert_eq!(default, SelectionStrategy::SmoothRateBasedRoundRobin);
}
