//! Translator factory
//!
//! This module provides a factory for creating translator instances.

use crate::core::error::Result;
use crate::translator::common::{DeepLXConfig, LLMConfig, TencentConfig};
use crate::translator::{ProviderType, TranslatorImpl};

/// Configuration for creating translators
#[derive(Debug, Clone, Default)]
pub struct TranslatorConfig {
    /// Provider type
    pub provider: ProviderType,
    /// DeepLX configuration
    pub deeplx: Option<DeepLXConfig>,
    /// LLM configuration
    pub llm: Option<LLMConfig>,
    /// Tencent configuration
    pub tencent: Option<TencentConfig>,
}

/// Create a translator from configuration using static dispatch
///
/// This function creates a TranslatorImpl enum variant directly,
pub fn create_translator_from_config(config: &TranslatorConfig) -> Result<TranslatorImpl> {
    TranslatorImpl::from_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_translator_from_config_deeplx() {
        let config = TranslatorConfig {
            provider: ProviderType::DeepLX,
            deeplx: Some(DeepLXConfig::default()),
            ..Default::default()
        };

        let translator = create_translator_from_config(&config);
        assert!(translator.is_ok());
    }
}
