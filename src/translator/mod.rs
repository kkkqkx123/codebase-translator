//! Translation services
//!
//! This module provides translation services for multiple providers:
//! - DeepLX: Free translation service based on DeepL
//! - LLM: OpenAI-compatible API support
//! - Tencent: Tencent Cloud Machine Translation

pub mod batch;
pub mod common;
pub mod deeplx;
pub mod factory;
pub mod llm;
pub mod service;
pub mod tencent;
pub mod r#trait;

use crate::config::{global::GlobalConfig, project::ProjectConfig};
use crate::core::error::Result;
use tracing::{info, warn};

// Re-export traits
pub use r#trait::{ProviderType, Translator, TranslatorImpl};

// Re-export common types
pub use common::{
    chars_to_tokens, tokens_to_chars, BatchOptions, BatchResult, DeepLXConfig, LLMConfig,
    LimitPolicy, TencentConfig, TranslateRequest, TranslateResponse,
};

// Re-export translators
pub use deeplx::DeepLXTranslator;
pub use llm::LLMTranslator;
pub use llm::{MultiProviderTranslator, ProviderPool, ProviderPoolConfig};
pub use tencent::TencentTranslator;

// Re-export factory
pub use factory::{create_translator_from_config, TranslatorConfig};

// Re-export batch translator
pub use batch::{create_batch_translator, BatchTranslator};

// Re-export sync translation service
pub use service::{BatchTranslationService, TranslationService};

/// Create translator instance from global and project configs
/// Always creates all enabled translators for load balancing and failover
pub fn create_translation_service(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    let enabled_providers = global_config.get_enabled_providers();
    info!(
        enabled_providers = ?enabled_providers,
        "Creating translation service with all enabled providers"
    );

    if enabled_providers.is_empty() {
        return Err(crate::core::error::TranslateError::Config(
            "At least one provider must be enabled".to_string(),
        ));
    }

    // Create all enabled translators
    let mut translators: Vec<(std::sync::Arc<TranslatorImpl>, u32)> = Vec::new();

    for provider_str in &enabled_providers {
        let provider_type = match provider_str.parse::<ProviderType>() {
            Ok(pt) => pt,
            Err(e) => {
                warn!(provider = %provider_str, error = %e, "Skipping invalid provider");
                continue;
            }
        };

        let translator_config = create_translator_config_for_provider(provider_type, global_config);

        match create_translator_from_config(&translator_config) {
            Ok(translator_impl) => {
                let weight = get_provider_weight(provider_type, global_config);
                translators.push((std::sync::Arc::new(translator_impl), weight));
                info!(provider = %provider_str, "Translator created successfully");
            }
            Err(e) => {
                warn!(provider = %provider_str, error = %e, "Failed to create translator");
            }
        }
    }

    if translators.is_empty() {
        return Err(crate::core::error::TranslateError::Config(
            "No valid translators could be created".to_string(),
        ));
    }

    // Create BatchTranslator with all enabled translators
    let batch_options = create_batch_options(global_config, project_config);
    let batch_translator = BatchTranslator::new(translators, batch_options);

    let translator =
        TranslationService::with_batch_translator(std::sync::Arc::new(batch_translator))?;
    info!(
        translator_count = enabled_providers.len(),
        "Translation service created successfully with multiple providers"
    );
    Ok(translator)
}

/// Create translator configuration for a specific provider
fn create_translator_config_for_provider(
    provider: ProviderType,
    global_config: &GlobalConfig,
) -> TranslatorConfig {
    match provider {
        ProviderType::DeepLX => TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(common::DeepLXConfig {
                api_url: global_config.deeplx.api_url.clone(),
                api_key: global_config.deeplx.api_key.clone(),
                proxy_url: global_config.deeplx.proxy_url.clone(),
                max_retries: global_config.deeplx.max_retries as usize,
            }),
            llm: None,
            tencent: None,
        },
        ProviderType::LLM => {
            let provider_config = global_config.llm.providers.first().cloned();
            let llm_config = provider_config.map(|p| common::LLMConfig {
                base_url: p.base_url.clone(),
                api_key: p.api_keys.first().cloned().unwrap_or_default(),
                model: if p.model.is_empty() {
                    p.model_list.first().cloned().unwrap_or_default()
                } else {
                    p.model.clone()
                },
                max_tokens: p.max_tokens as i32,
                temperature: p.temperature as f64,
                top_p: None,
                proxy_url: p.proxy_url.clone(),
                timeout: p.timeout,
                max_retries: 3,
                extra_headers: Some(p.extra_headers.clone()),
                extra_params: Some(
                    serde_json::to_value(&p.extra_params).unwrap_or_default(),
                ),
            });

            TranslatorConfig {
                provider: ProviderType::LLM,
                deeplx: None,
                llm: llm_config,
                tencent: None,
            }
        }
        ProviderType::Tencent => TranslatorConfig {
            provider: ProviderType::Tencent,
            deeplx: None,
            llm: None,
            tencent: Some(common::TencentConfig {
                secret_id: global_config.tencent.secret_id.clone().unwrap_or_default(),
                secret_key: global_config.tencent.secret_key.clone().unwrap_or_default(),
                region: global_config.tencent.region.clone(),
                project_id: global_config.tencent.project_id as i64,
                proxy_url: global_config.tencent.proxy_url.clone(),
                timeout: global_config.tencent.timeout,
                max_retries: global_config.tencent.max_retries as usize,
                untranslated_text: global_config.tencent.untranslated_text.clone(),
                term_repo_id_list: global_config.tencent.term_repo_id_list.clone(),
                sent_repo_id_list: global_config.tencent.sent_repo_id_list.clone(),
            }),
        },
    }
}

/// Get weight for a provider (for load balancing)
fn get_provider_weight(provider: ProviderType, global_config: &GlobalConfig) -> u32 {
    match provider {
        ProviderType::DeepLX => 50,
        ProviderType::LLM => global_config
            .llm
            .providers
            .first()
            .map(|p| p.weight)
            .unwrap_or(50),
        ProviderType::Tencent => 50,
    }
}

/// Create batch options for the translation service
fn create_batch_options(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> common::BatchOptions {
    common::BatchOptions {
        rate_limit: global_config.limits.rate_limit,
        workers: project_config.translate.concurrency.max(1),
        max_retries: 3,
        limit_policy: Some(common::LimitPolicy {
            rate_limit: global_config.limits.rate_limit,
            max_char_count: 5000,
            split_max_chars: global_config.limits.split_max_chars as usize,
        }),
    }
}
