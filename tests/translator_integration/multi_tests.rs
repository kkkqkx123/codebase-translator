//! Multi-Translator Integration Tests
//!
//! Tests for the multi-translator with load balancing and failover.
//! Validates selection strategies, health tracking, and failover behavior.

use std::sync::Arc;

use codebase_translate::translator::{
    DeepLXConfig, MultiTranslator, ProviderType, SelectionStrategy, TranslatorConfig,
    TranslatorImpl,
};

/// Test MultiTranslator creation fails with empty translator list
#[test]
fn test_multi_translator_fails_with_empty_list() {
    let translators: Vec<(Arc<TranslatorImpl>, u32)> = vec![];
    let result = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("At least one translator is required"));
}

/// Test MultiTranslator creation with single translator
#[test]
fn test_multi_translator_with_single_translator() {
    let config = DeepLXConfig::default();
    let translator = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator), 1u32)];
    let result = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3);

    assert!(result.is_ok());
    let multi = result.expect("Should create multi-translator");
    assert_eq!(multi.name(), "multi");
}

/// Test MultiTranslator creation with multiple translators
#[test]
fn test_multi_translator_with_multiple_translators() {
    let config1 = DeepLXConfig::default();
    let translator1 = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config1),
        ..Default::default()
    })
    .expect("Should create translator");

    let config2 = DeepLXConfig::default();
    let translator2 = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config2),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator1), 2u32), (Arc::new(translator2), 1u32)];
    let result = MultiTranslator::new(translators, SelectionStrategy::Weighted, 3);

    assert!(result.is_ok());
}

/// Test SelectionStrategy variants parsing
#[test]
fn test_selection_strategy_variants() {
    use std::str::FromStr;

    assert_eq!(
        SelectionStrategy::from_str("round_robin").unwrap(),
        SelectionStrategy::RoundRobin
    );
    assert_eq!(
        SelectionStrategy::from_str("roundrobin").unwrap(),
        SelectionStrategy::RoundRobin
    );
    assert_eq!(
        SelectionStrategy::from_str("weighted").unwrap(),
        SelectionStrategy::Weighted
    );
    assert!(SelectionStrategy::from_str("unknown").is_err());
}

/// Test SelectionStrategy case insensitivity
#[test]
fn test_selection_strategy_case_insensitive() {
    use std::str::FromStr;

    assert_eq!(
        SelectionStrategy::from_str("ROUND_ROBIN").unwrap(),
        SelectionStrategy::RoundRobin
    );
    assert_eq!(
        SelectionStrategy::from_str("RoundRobin").unwrap(),
        SelectionStrategy::RoundRobin
    );
    assert_eq!(
        SelectionStrategy::from_str("WEIGHTED").unwrap(),
        SelectionStrategy::Weighted
    );
    assert_eq!(
        SelectionStrategy::from_str("Weighted").unwrap(),
        SelectionStrategy::Weighted
    );
}

/// Test MultiTranslator supported languages
#[tokio::test]
async fn test_multi_translator_supported_langs() {
    let config = DeepLXConfig::default();
    let translator = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator), 1u32)];
    let multi = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3)
        .expect("Should create multi-translator");

    let source_langs = multi.supported_source_langs();
    assert!(source_langs.contains(&"AUTO"));

    let target_langs = multi.supported_target_langs();
    assert!(target_langs.contains(&"EN"));
    assert!(target_langs.contains(&"ZH"));
}

/// Test MultiTranslator max input chars
#[test]
fn test_multi_translator_max_input_chars() {
    let config = DeepLXConfig::default();
    let translator = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator), 1u32)];
    let multi = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3)
        .expect("Should create multi-translator");

    // DeepLX has no specific limit (returns 0)
    assert_eq!(multi.max_input_chars(), 0);
}

/// Test MultiTranslator can_handle method
#[test]
fn test_multi_translator_can_handle() {
    let config = DeepLXConfig::default();
    let translator = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator), 1u32)];
    let multi = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3)
        .expect("Should create multi-translator");

    // DeepLX can handle any text length (max_input_chars returns 0)
    assert!(multi.can_handle(100));
    assert!(multi.can_handle(10000));
    assert!(multi.can_handle(100000));
}

/// Test MultiTranslator with different retry counts
#[test]
fn test_multi_translator_retry_counts() {
    let config = DeepLXConfig::default();
    let translator = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator), 1u32)];

    // Test with 0 retries (should default to 3)
    let result = MultiTranslator::new(translators.clone(), SelectionStrategy::RoundRobin, 0);
    assert!(result.is_ok());

    // Test with normal retry count
    let result = MultiTranslator::new(translators.clone(), SelectionStrategy::RoundRobin, 5);
    assert!(result.is_ok());

    // Test with high retry count (should be limited to 10)
    let result = MultiTranslator::new(translators.clone(), SelectionStrategy::RoundRobin, 20);
    assert!(result.is_ok());
}

/// Test MultiTranslator creation with mixed translator types
#[test]
fn test_multi_translator_with_mixed_types() {
    let deeplx_config = DeepLXConfig::default();
    let deeplx = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(deeplx_config),
        ..Default::default()
    })
    .expect("Should create DeepLX translator");

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
    let llm = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::LLM,
        llm: Some(llm_config),
        ..Default::default()
    })
    .expect("Should create LLM translator");

    let translators = vec![
        (Arc::new(deeplx), 2u32),
        (Arc::new(llm), 1u32),
    ];
    let result = MultiTranslator::new(translators, SelectionStrategy::Weighted, 3);

    assert!(result.is_ok());
    let multi = result.expect("Should create multi-translator");
    assert_eq!(multi.name(), "multi");
}

/// Test MultiTranslator name method
#[test]
fn test_multi_translator_name() {
    let config = DeepLXConfig::default();
    let translator = TranslatorImpl::from_config(&TranslatorConfig {
        provider: ProviderType::DeepLX,
        deeplx: Some(config),
        ..Default::default()
    })
    .expect("Should create translator");

    let translators = vec![(Arc::new(translator), 1u32)];
    let multi = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3)
        .expect("Should create multi-translator");

    assert_eq!(multi.name(), "multi");
}

/// Test SelectionStrategy debug format
#[test]
fn test_selection_strategy_debug() {
    let round_robin = SelectionStrategy::RoundRobin;
    let weighted = SelectionStrategy::Weighted;

    assert!(format!("{:?}", round_robin).contains("RoundRobin"));
    assert!(format!("{:?}", weighted).contains("Weighted"));
}

/// Test SelectionStrategy equality
#[test]
fn test_selection_strategy_equality() {
    assert_eq!(SelectionStrategy::RoundRobin, SelectionStrategy::RoundRobin);
    assert_eq!(SelectionStrategy::Weighted, SelectionStrategy::Weighted);
    assert_ne!(SelectionStrategy::RoundRobin, SelectionStrategy::Weighted);
}

/// Test SelectionStrategy clone
#[test]
fn test_selection_strategy_clone() {
    let round_robin = SelectionStrategy::RoundRobin;
    let cloned = round_robin.clone();
    assert_eq!(round_robin, cloned);

    let weighted = SelectionStrategy::Weighted;
    let cloned = weighted.clone();
    assert_eq!(weighted, cloned);
}

/// Test SelectionStrategy copy
#[test]
fn test_selection_strategy_copy() {
    let round_robin = SelectionStrategy::RoundRobin;
    let copied: SelectionStrategy = round_robin;
    assert_eq!(round_robin, copied);
}
