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
pub use llm::{MultiProviderTranslator, ProviderPool, ProviderPoolConfig, RotationStrategy};
pub use multi::MultiTranslator;
pub use tencent::TencentTranslator;

// Re-export factory
pub use factory::{create_translator_from_config, TranslatorConfig};

// Re-export batch translator
pub use batch::{create_batch_translator, BatchTranslator};

// Re-export sync translation service
pub use service::{BatchTranslationService, TranslationService};
