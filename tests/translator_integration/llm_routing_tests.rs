//! LLM Provider Routing Integration Tests
//!
//! Tests for the provider router with capacity-aware routing.
//! Validates text length based routing and weighted distribution.

use codebase_translate::config::LLMProviderConfig;
use codebase_translate::translator::llm::routing::{CapacityProvider, ProviderRouter};

/// Helper function to create test provider configs with different capacities
fn create_test_configs_with_capacities() -> Vec<LLMProviderConfig> {
    vec![
        LLMProviderConfig {
            id: "small".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "llama2".to_string(),
            max_tokens: 1024, // Small capacity
            weight: 1,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "large".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "llama3".to_string(),
            max_tokens: 4096, // Large capacity
            weight: 2,
            ..Default::default()
        },
    ]
}

/// Test provider router creation
#[test]
fn test_provider_router_creation() {
    let configs = create_test_configs_with_capacities();
    let router = ProviderRouter::new(&configs);
    assert!(router.is_ok(), "Should create provider router");
}

/// Test provider router fails with empty configs
#[test]
fn test_provider_router_fails_empty_configs() {
    let configs: Vec<LLMProviderConfig> = vec![];
    let router = ProviderRouter::new(&configs);
    assert!(router.is_err(), "Should fail with empty configs");
}

/// Test provider router fails with duplicate IDs
#[test]
fn test_provider_router_fails_duplicate_ids() {
    let configs = vec![
        LLMProviderConfig {
            id: "duplicate".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            ..Default::default()
        },
        LLMProviderConfig {
            id: "duplicate".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            ..Default::default()
        },
    ];
    let router = ProviderRouter::new(&configs);
    assert!(router.is_err(), "Should fail with duplicate IDs");
}

/// Test provider router fails with empty provider ID
#[test]
fn test_provider_router_fails_empty_id() {
    let configs = vec![LLMProviderConfig {
        id: "".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        ..Default::default()
    }];
    let router = ProviderRouter::new(&configs);
    assert!(router.is_err(), "Should fail with empty provider ID");
}

/// Test capacity provider can_handle
#[test]
fn test_capacity_provider_can_handle() {
    let config = LLMProviderConfig {
        id: "test".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 1000,
        ..Default::default()
    };

    let provider = CapacityProvider::new(&config).expect("Should create provider");
    let max_chars = provider.max_chars();

    assert!(provider.can_handle(max_chars / 2));
    assert!(provider.can_handle(max_chars));
    assert!(!provider.can_handle(max_chars + 1000));
}

/// Test router selects provider for short text (below threshold)
#[test]
fn test_router_selects_for_short_text() {
    let configs = create_test_configs_with_capacities();
    let router = ProviderRouter::new(&configs).expect("Should create router");

    // Short text should be routable to any provider
    let provider = router.select_provider(100);
    assert!(provider.is_some(), "Should select provider for short text");
}

/// Test router filters providers for long text (above threshold)
#[test]
fn test_router_filters_for_long_text() {
    let configs = create_test_configs_with_capacities();
    let router = ProviderRouter::new(&configs).expect("Should create router");

    let threshold = router.capacity_threshold();

    // Long text above threshold should only route to capable providers
    let provider = router.select_provider(threshold + 1000);

    // Should still find a provider (the large one)
    assert!(provider.is_some(), "Should select provider for long text");

    // The selected provider should be able to handle the text
    let selected = provider.unwrap();
    assert!(selected.can_handle(threshold + 1000));
}

/// Test router returns None for text too long for all providers
#[test]
fn test_router_returns_none_for_oversized_text() {
    let configs = vec![LLMProviderConfig {
        id: "small".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 100,
        ..Default::default()
    }];
    let router = ProviderRouter::new(&configs).expect("Should create router");

    let max_capacity = router.max_capacity();

    // Text exceeding all provider capacities
    let provider = router.select_provider(max_capacity + 10000);
    assert!(provider.is_none(), "Should return None for oversized text");
}

/// Test router can_handle method
#[test]
fn test_router_can_handle() {
    let configs = create_test_configs_with_capacities();
    let router = ProviderRouter::new(&configs).expect("Should create router");

    let max_capacity = router.max_capacity();

    assert!(router.can_handle(100));
    assert!(router.can_handle(max_capacity));
    assert!(!router.can_handle(max_capacity + 10000));
}

/// Test weighted distribution over multiple selections
#[test]
fn test_router_weighted_distribution() {
    let configs = vec![
        LLMProviderConfig {
            id: "light".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            max_tokens: 4096,
            weight: 1,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "heavy".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            max_tokens: 4096,
            weight: 3,
            ..Default::default()
        },
    ];
    let router = ProviderRouter::new(&configs).expect("Should create router");

    let mut heavy_count = 0;
    let mut light_count = 0;

    // Short text that both providers can handle
    for _ in 0..100 {
        let provider = router.select_provider(100);
        assert!(provider.is_some());

        match provider.unwrap().provider().id() {
            "heavy" => heavy_count += 1,
            "light" => light_count += 1,
            _ => {}
        }
    }

    // Heavy provider should be selected more often
    assert!(heavy_count > light_count, "Heavy provider should be selected more often");
}

/// Test capacity threshold calculation
#[test]
fn test_capacity_threshold_calculation() {
    let configs = vec![
        LLMProviderConfig {
            id: "small".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_keys: vec!["key1".to_string()],
            model: "model1".to_string(),
            max_tokens: 1000,
            ..Default::default()
        },
        LLMProviderConfig {
            id: "large".to_string(),
            base_url: "http://localhost:11435".to_string(),
            api_keys: vec!["key2".to_string()],
            model: "model2".to_string(),
            max_tokens: 4000,
            ..Default::default()
        },
    ];
    let router = ProviderRouter::new(&configs).expect("Should create router");

    // Threshold should be the minimum capacity
    let threshold = router.capacity_threshold();
    assert!(threshold > 0, "Threshold should be positive");
}

/// Test router with single provider
#[test]
fn test_router_single_provider() {
    let configs = vec![LLMProviderConfig {
        id: "only".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 2000,
        ..Default::default()
    }];
    let router = ProviderRouter::new(&configs).expect("Should create router");

    let provider = router.select_provider(100);
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().provider().id(), "only");
}

/// Test CapacityProvider max_chars calculation
#[test]
fn test_capacity_provider_max_chars() {
    let config = LLMProviderConfig {
        id: "test".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 1000,
        ..Default::default()
    };

    let provider = CapacityProvider::new(&config).expect("Should create provider");

    // max_chars should be calculated from max_tokens (70% of tokens * 1.5 chars/token)
    let expected_max_chars = ((1000.0 * 0.7) as usize * 15) / 10;
    assert_eq!(provider.max_chars(), expected_max_chars);
}

/// Test router providers accessor
#[test]
fn test_router_providers_accessor() {
    let configs = create_test_configs_with_capacities();
    let router = ProviderRouter::new(&configs).expect("Should create router");

    let providers = router.providers();
    assert_eq!(providers.len(), 2);
}

/// Test CapacityProvider weight accessor
#[test]
fn test_capacity_provider_weight() {
    let config = LLMProviderConfig {
        id: "test".to_string(),
        base_url: "http://localhost:11434".to_string(),
        api_keys: vec!["key".to_string()],
        model: "model".to_string(),
        max_tokens: 1000,
        weight: 5,
        ..Default::default()
    };

    let provider = CapacityProvider::new(&config).expect("Should create provider");
    assert_eq!(provider.weight(), 5);
}
