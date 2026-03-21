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
pub mod multi;
pub mod service;
pub mod tencent;
pub mod r#trait;

use crate::config::{global::GlobalConfig, project::ProjectConfig};
use crate::core::error::Result;
use tracing::{debug, info};

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
pub use multi::MultiTranslator;
pub use tencent::TencentTranslator;

// Re-export factory
pub use factory::{create_translator_from_config, TranslatorConfig};

// Re-export batch translator
pub use batch::{create_batch_translator, BatchTranslator};

// Re-export sync translation service
pub use service::{BatchTranslationService, TranslationService};

/// Create translator instance from global and project configs
pub fn create_translation_service(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    info!(
        provider = %project_config.translate.provider,
        "Creating translator instance"
    );

    let translator_config = match project_config.translate.provider {
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
            if global_config.llm.providers.is_empty() {
                return Err(crate::core::error::TranslateError::Config(
                    "At least one LLM provider is required".to_string(),
                ));
            }

            let provider = &global_config.llm.providers[0];
            let api_key = provider.api_keys.first().cloned().unwrap_or_default();
            let model = if provider.model.is_empty() {
                provider.model_list.first().cloned().unwrap_or_default()
            } else {
                provider.model.clone()
            };

            TranslatorConfig {
                provider: ProviderType::LLM,
                deeplx: None,
                llm: Some(common::LLMConfig {
                    base_url: provider.base_url.clone(),
                    api_key,
                    model,
                    max_tokens: provider.max_tokens as i32,
                    temperature: provider.temperature as f64,
                    top_p: None,
                    proxy_url: provider.proxy_url.clone(),
                    timeout: provider.timeout,
                    max_retries: 3,
                    extra_headers: Some(provider.extra_headers.clone()),
                    extra_params: Some(
                        serde_json::to_value(&provider.extra_params).unwrap_or_default(),
                    ),
                }),
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
    };

    let translator_impl = create_translator_from_config(&translator_config)?;

    let batch_options = common::BatchOptions {
        rate_limit: match project_config.translate.provider {
            ProviderType::DeepLX => global_config.deeplx.rate_limit,
            ProviderType::LLM => global_config.limits.rate_limit,
            ProviderType::Tencent => global_config.tencent.rate_limit,
        },
        workers: 5,
        max_retries: match project_config.translate.provider {
            ProviderType::DeepLX => global_config.deeplx.max_retries as usize,
            ProviderType::LLM => 3,
            ProviderType::Tencent => global_config.tencent.max_retries as usize,
        },
        limit_policy: Some(match project_config.translate.provider {
            ProviderType::DeepLX => common::LimitPolicy {
                rate_limit: global_config.deeplx.rate_limit,
                max_char_count: 5000,
                split_max_chars: 4000,
            },
            ProviderType::LLM => common::LimitPolicy {
                rate_limit: global_config.limits.rate_limit,
                max_char_count: global_config
                    .llm
                    .providers
                    .first()
                    .map(|p| p.max_tokens as usize)
                    .unwrap_or(4096),
                split_max_chars: global_config.limits.split_max_chars as usize,
            },
            ProviderType::Tencent => common::LimitPolicy {
                rate_limit: global_config.tencent.rate_limit,
                max_char_count: 6000,
                split_max_chars: 5000,
            },
        }),
    };

    let batch_translator =
        BatchTranslator::new(std::sync::Arc::new(translator_impl), batch_options);

    let translator =
        TranslationService::with_batch_translator(std::sync::Arc::new(batch_translator))?;
    debug!("Translator instance created successfully with batch translator");
    Ok(translator)
}
