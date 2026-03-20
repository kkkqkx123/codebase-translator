//! Translator Module Integration Tests
//!
//! These tests verify the integration between factory, multi, service, common, and batch components
//! with translator implementations. They focus on component integration integrity without making
//! actual API calls to external translation services.

use std::sync::Arc;

use codebase_translate::core::error::TranslateError;
use codebase_translate::translator::multi::SelectionStrategy;
use codebase_translate::translator::{
    create_batch_translator, create_translator_from_config, BatchOptions, BatchResult,
    BatchTranslationService, BatchTranslator, DeepLXConfig, LimitPolicy, MultiTranslator,
    ProviderType, TencentConfig, TranslationService, Translator, TranslatorConfig, TranslatorImpl,
};

// ============================================================================
// Factory Integration Tests
// ============================================================================

mod factory_tests {
    use super::*;

    /// Test factory creates DeepLX translator correctly
    #[test]
    fn test_factory_creates_deeplx_translator() {
        let config = TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(DeepLXConfig {
                api_url: "http://localhost:1188".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = create_translator_from_config(&config);
        assert!(result.is_ok(), "Factory should create DeepLX translator");

        let translator = result.expect("Should get translator");
        assert_eq!(translator.name(), "deeplx");
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

        // LLM (with config)
        let llm_config = TranslatorConfig {
            provider: ProviderType::LLM,
            llm: Some(codebase_translate::translator::LLMConfig {
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
}

// ============================================================================
// Common Module Integration Tests
// ============================================================================

mod common_tests {
    use codebase_translate::translator::common::{
        chars_to_tokens, tokens_to_chars, BatchOptions, LimitPolicy,
    };

    /// Test character to token conversion
    #[test]
    fn test_chars_to_tokens_conversion() {
        // 1 token ≈ 1.5 characters
        assert_eq!(chars_to_tokens(0), 0);
        assert_eq!(chars_to_tokens(1), 1);
        assert_eq!(chars_to_tokens(15), 10);
        assert_eq!(chars_to_tokens(150), 100);
    }

    /// Test token to character conversion
    #[test]
    fn test_tokens_to_chars_conversion() {
        // 1 token ≈ 1.5 characters
        assert_eq!(tokens_to_chars(0), 0);
        assert_eq!(tokens_to_chars(10), 15);
        assert_eq!(tokens_to_chars(100), 150);
    }

    /// Test LimitPolicy creation from character count
    #[test]
    fn test_limit_policy_from_char_count() {
        let policy = LimitPolicy::from_char_count(5000);
        assert_eq!(policy.max_char_count, 5000);
        assert_eq!(policy.split_max_chars, 4000); // 80% of max
        assert_eq!(policy.rate_limit, 10);
    }

    /// Test LimitPolicy creation from token count
    #[test]
    fn test_limit_policy_from_token_count() {
        let policy = LimitPolicy::from_token_count(1000);
        assert_eq!(policy.max_char_count, 1500); // 1000 * 1.5
        assert_eq!(policy.split_max_chars, 1200); // 80% of max
    }

    /// Test BatchOptions default values
    #[test]
    fn test_batch_options_default() {
        let options = BatchOptions::default();
        assert_eq!(options.rate_limit, 10);
        assert_eq!(options.workers, 5);
        assert_eq!(options.max_retries, 3);
        assert!(options.limit_policy.is_none());
    }

    /// Test BatchOptions custom values
    #[test]
    fn test_batch_options_custom() {
        let options = BatchOptions {
            rate_limit: 20,
            workers: 10,
            max_retries: 5,
            limit_policy: Some(LimitPolicy::default()),
        };
        assert_eq!(options.rate_limit, 20);
        assert_eq!(options.workers, 10);
        assert_eq!(options.max_retries, 5);
        assert!(options.limit_policy.is_some());
    }
}

// ============================================================================
// Multi-Translator Integration Tests
// ============================================================================

mod multi_tests {
    use super::*;

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

    /// Test SelectionStrategy variants
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

    }

// ============================================================================
// Batch Translator Integration Tests
// ============================================================================

mod batch_tests {
    use super::*;

    /// Test BatchTranslator creation
    #[test]
    fn test_batch_translator_creation() {
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
        let batch = BatchTranslator::new(translator, options);

        assert_eq!(batch.name(), "deeplx");
    }

    /// Test create_batch_translator helper function
    #[test]
    fn test_create_batch_translator_helper() {
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
        let batch = create_batch_translator(translator, options);

        assert_eq!(batch.name(), "deeplx");
    }

    /// Test BatchTranslator with custom options
    #[test]
    fn test_batch_translator_custom_options() {
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
            limit_policy: Some(LimitPolicy::from_char_count(3000)),
        };
        let batch = BatchTranslator::new(translator, options);

        assert_eq!(batch.name(), "deeplx");
    }

    /// Test BatchResult default values
    #[test]
    fn test_batch_result_default() {
        let result = BatchResult::default();
        assert_eq!(result.total_count, 0);
        assert_eq!(result.success_count, 0);
        assert_eq!(result.failed_count, 0);
        assert!(result.results.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.processing_time, 0);
        assert_eq!(result.total_chars, 0);
        assert_eq!(result.total_tokens, 0);
        assert_eq!(result.average_latency_ms, 0.0);
    }
}

// ============================================================================
// Translation Service Integration Tests
// ============================================================================

mod service_tests {
    use super::*;

    /// Test TranslationService creation
    #[test]
    fn test_translation_service_creation() {
        let config = TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(DeepLXConfig::default()),
            ..Default::default()
        };

        let result = TranslationService::new(config);
        assert!(result.is_ok(), "Should create translation service");

        let service = result.expect("Should get service");
        assert_eq!(service.name(), "deeplx");
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
        let result = BatchTranslationService::new(translator, options);

        assert!(result.is_ok(), "Should create batch translation service");
    }
}

// ============================================================================
// End-to-End Component Integration Tests
// ============================================================================

mod e2e_component_tests {
    use super::*;

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
            limit_policy: Some(LimitPolicy::default()),
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
        let tencent =
            create_translator_from_config(&tencent_config).expect("Should create Tencent");
        assert_eq!(tencent.name(), "tencent");
    }
}

// ============================================================================
// Error Handling Integration Tests
// ============================================================================

mod error_handling_tests {
    use super::*;

    /// Test error propagation through factory
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
        assert!(matches!(err, TranslateError::Config(_)));
    }

    /// Test MultiTranslator creation error
    #[test]
    fn test_multi_translator_creation_error() {
        let translators: Vec<(Arc<TranslatorImpl>, u32)> = vec![];
        let result = MultiTranslator::new(translators, SelectionStrategy::RoundRobin, 3);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TranslateError::Config(_)));
    }

    /// Test BatchTranslationService creation error with invalid runtime
    /// Note: This test may not trigger error in normal conditions
    #[test]
    fn test_batch_service_creation() {
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
        let result = BatchTranslationService::new(translator, options);

        assert!(result.is_ok());
    }
}

// ============================================================================
// Trait Implementation Integration Tests
// ============================================================================

mod trait_tests {
    use super::*;

    /// Test ProviderType display implementation
    #[test]
    fn test_provider_type_display() {
        assert_eq!(format!("{}", ProviderType::DeepLX), "deeplx");
        assert_eq!(format!("{}", ProviderType::LLM), "llm");
        assert_eq!(format!("{}", ProviderType::Tencent), "tencent");
    }

    /// Test ProviderType from_str implementation
    #[test]
    fn test_provider_type_from_str() {
        use std::str::FromStr;

        assert_eq!(
            ProviderType::from_str("deeplx").unwrap(),
            ProviderType::DeepLX
        );
        assert_eq!(ProviderType::from_str("llm").unwrap(), ProviderType::LLM);
        assert_eq!(
            ProviderType::from_str("tencent").unwrap(),
            ProviderType::Tencent
        );

        // Case insensitive
        assert_eq!(
            ProviderType::from_str("DEEPLX").unwrap(),
            ProviderType::DeepLX
        );
        assert_eq!(ProviderType::from_str("LLM").unwrap(), ProviderType::LLM);

        // Invalid provider
        assert!(ProviderType::from_str("invalid").is_err());
    }

    /// Test ProviderType default
    #[test]
    fn test_provider_type_default() {
        let default = ProviderType::default();
        assert_eq!(default, ProviderType::DeepLX);
    }

    /// Test TranslatorImpl name method for all variants
    #[test]
    fn test_translator_impl_name() {
        // DeepLX
        let deeplx_config = TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(DeepLXConfig::default()),
            ..Default::default()
        };
        let deeplx = create_translator_from_config(&deeplx_config).unwrap();
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
        let llm = create_translator_from_config(&llm_config).unwrap();
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
        let tencent = create_translator_from_config(&tencent_config).unwrap();
        assert_eq!(tencent.name(), "tencent");
    }
}
