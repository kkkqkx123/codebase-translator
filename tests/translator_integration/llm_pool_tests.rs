//! LLM Provider Pool Integration Tests
//!
//! Tests for the provider pool with round-robin and weighted rotation strategies.
//! Validates provider selection behavior and health check functionality.

use std::time::Duration;

use codebase_translate::config::LLMProviderConfig;
use codebase_translate::translator::llm::pool::{ProviderPool, ProviderPoolConfig, RotationStrategy};

/// Helper function to create test provider configs
fn create_test_provider_configs() -> Vec<LLMProviderConfig> {
    vec![
        LLMProviderConfig {
            id: "provider1".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "llama2".to_string(),
            max_tokens: 2048,
            weight: 1,
            timeout: 30,
            max_retries: 3,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "provider2".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "llama3".to_string(),
            max_tokens: 4096,
            weight: 2,
            timeout: 30,
            max_retries: 3,
            ..Default::default()
        },
    ]
}

/// Test provider pool creation with round-robin strategy
#[tokio::test]
async fn test_provider_pool_round_robin_creation() {
    let configs = create_test_provider_configs();
    let pool_config = ProviderPoolConfig {
        strategy: RotationStrategy::RoundRobin,
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config).await;
    assert!(pool.is_ok(), "Should create provider pool with round-robin");
}

/// Test provider pool creation with weighted strategy
#[tokio::test]
async fn test_provider_pool_weighted_creation() {
    let configs = create_test_provider_configs();
    let pool_config = ProviderPoolConfig {
        strategy: RotationStrategy::Weighted,
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config).await;
    assert!(pool.is_ok(), "Should create provider pool with weighted");
}

/// Test provider pool fails with empty configs
#[tokio::test]
async fn test_provider_pool_fails_with_empty_configs() {
    let configs: Vec<LLMProviderConfig> = vec![];
    let pool_config = ProviderPoolConfig::default();

    let pool = ProviderPool::new(&configs, pool_config).await;
    assert!(pool.is_err(), "Should fail with empty configs");
}

/// Test provider pool fails with invalid provider (empty base_url)
#[tokio::test]
async fn test_provider_pool_skips_invalid_providers() {
    let configs = vec![
        LLMProviderConfig {
            id: "invalid".to_string(),
            base_url: "".to_string(), // Empty base_url
            api_keys: vec!["key".to_string()],
            model: "test".to_string(),
            ..Default::default()
        },
    ];
    let pool_config = ProviderPoolConfig::default();

    let pool = ProviderPool::new(&configs, pool_config).await;
    assert!(pool.is_err(), "Should fail when all providers are invalid");
}

/// Test provider pool fails with duplicate provider IDs
#[tokio::test]
async fn test_provider_pool_duplicate_ids() {
    let configs = vec![
        LLMProviderConfig {
            id: "duplicate".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            ..Default::default()
        },
        LLMProviderConfig {
            id: "duplicate".to_string(), // Same ID
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            ..Default::default()
        },
    ];
    let pool_config = ProviderPoolConfig::default();

    let pool = ProviderPool::new(&configs, pool_config).await;
    assert!(pool.is_err(), "Should fail with duplicate provider IDs");
}

/// Test round-robin provider selection cycles through providers
#[tokio::test]
async fn test_round_robin_selection_cycles() {
    let configs = create_test_provider_configs();
    let pool_config = ProviderPoolConfig {
        strategy: RotationStrategy::RoundRobin,
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config)
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

/// Test weighted provider selection respects weights
#[tokio::test]
async fn test_weighted_selection_distribution() {
    let configs = vec![
        LLMProviderConfig {
            id: "light".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            weight: 1,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "heavy".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            weight: 3,
            ..Default::default()
        },
    ];
    let pool_config = ProviderPoolConfig {
        strategy: RotationStrategy::Weighted,
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config)
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
    assert!(heavy_count > light_count, "Heavy provider should be selected more often");
    let ratio = heavy_count as f64 / light_count as f64;
    assert!(ratio > 1.5 && ratio < 5.0, "Ratio should be approximately 3:1, got {}", ratio);
}

/// Test provider pool get_all_providers returns all providers
#[tokio::test]
async fn test_get_all_providers() {
    let configs = create_test_provider_configs();
    let pool_config = ProviderPoolConfig {
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config)
        .await
        .expect("Should create pool");

    let providers = pool.get_all_providers().await;
    assert_eq!(providers.len(), 2);
}

/// Test provider pool get_provider_by_id
#[tokio::test]
async fn test_get_provider_by_id() {
    let configs = create_test_provider_configs();
    let pool_config = ProviderPoolConfig {
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config)
        .await
        .expect("Should create pool");

    let provider = pool.get_provider_by_id("provider1").await;
    assert!(provider.is_ok());
    assert_eq!(provider.unwrap().id(), "provider1");

    let not_found = pool.get_provider_by_id("nonexistent").await;
    assert!(not_found.is_err());
}

/// Test RotationStrategy parsing
#[test]
fn test_rotation_strategy_from_str() {
    use std::str::FromStr;

    assert_eq!(
        RotationStrategy::from_str("round_robin").unwrap(),
        RotationStrategy::RoundRobin
    );
    assert_eq!(
        RotationStrategy::from_str("roundrobin").unwrap(),
        RotationStrategy::RoundRobin
    );
    assert_eq!(
        RotationStrategy::from_str("weighted").unwrap(),
        RotationStrategy::Weighted
    );
    assert!(RotationStrategy::from_str("unknown").is_err());
}

/// Test RotationStrategy case insensitivity
#[test]
fn test_rotation_strategy_case_insensitive() {
    use std::str::FromStr;

    assert_eq!(
        RotationStrategy::from_str("ROUND_ROBIN").unwrap(),
        RotationStrategy::RoundRobin
    );
    assert_eq!(
        RotationStrategy::from_str("RoundRobin").unwrap(),
        RotationStrategy::RoundRobin
    );
    assert_eq!(
        RotationStrategy::from_str("WEIGHTED").unwrap(),
        RotationStrategy::Weighted
    );
}

/// Test ProviderPoolConfig default values
#[test]
fn test_provider_pool_config_default() {
    let config = ProviderPoolConfig::default();
    assert_eq!(config.strategy, RotationStrategy::RoundRobin);
    assert!(config.health_check_enabled);
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.health_check_timeout, Duration::from_secs(5));
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.recovery_interval, Duration::from_secs(300));
}

/// Test provider pool with all zero weights falls back to round-robin
#[tokio::test]
async fn test_weighted_with_zero_weights_fallback() {
    let configs = vec![
        LLMProviderConfig {
            id: "p1".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            weight: 0,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "p2".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            weight: 0,
            ..Default::default()
        },
    ];
    let pool_config = ProviderPoolConfig {
        strategy: RotationStrategy::Weighted,
        health_check_enabled: false,
        ..Default::default()
    };

    let pool = ProviderPool::new(&configs, pool_config)
        .await
        .expect("Should create pool");

    // Should still work even with all zero weights
    let provider = pool.get_provider().await;
    assert!(provider.is_ok());
}

