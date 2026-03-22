//! Translator trait definition
//!
//! This module defines the Translator trait for translation services.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::error::Result;
use crate::reporter::Reporter;
use crate::translator::deeplx::DeepLXTranslator;
use crate::translator::llm::MultiProviderTranslator;
use crate::translator::tencent::TencentTranslator;
use tracing::{debug, info};

/// Translator trait for translation services
#[async_trait]
pub trait Translator: Send + Sync {
    /// Translate a batch of texts
    ///
    /// # Arguments
    /// * `texts` - Texts to translate
    /// * `source_lang` - Source language code (e.g., "en", "zh", "AUTO")
    /// * `target_lang` - Target language code (e.g., "en", "zh")
    ///
    /// # Returns
    /// Translated texts in the same order as input
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>>;

    /// Translate a single text with source language specification
    ///
    /// # Arguments
    /// * `text` - Text to translate
    /// * `source_lang` - Source language code (e.g., "en", "zh", "AUTO")
    /// * `target_lang` - Target language code (e.g., "en", "zh")
    ///
    /// # Returns
    /// Translated text
    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String>;

    /// Get translator name
    fn name(&self) -> &str;

    /// Check if translator is available (has valid credentials)
    async fn is_available(&self) -> bool;

    /// Get source languages supported
    fn supported_source_langs(&self) -> Vec<&str>;

    /// Get target languages supported
    fn supported_target_langs(&self) -> Vec<&str>;

    /// Get maximum input characters allowed for this translator
    /// Returns 0 if no specific limit is enforced
    fn max_input_chars(&self) -> usize;

    /// Check if the translator can handle text of given length
    fn can_handle(&self, text_len: usize) -> bool {
        let max = self.max_input_chars();
        max == 0 || text_len <= max
    }

    /// Close and cleanup resources
    async fn close(&self) -> Result<()> {
        Ok(())
    }

    /// Set reporter for statistics tracking
    fn set_reporter(&mut self, reporter: Arc<dyn Reporter>);

    /// Get reporter if set
    fn reporter(&self) -> Option<Arc<dyn Reporter>>;
}

/// Provider type for translation services
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum ProviderType {
    /// DeepLX translation service
    #[default]
    DeepLX,
    /// LLM-based translation
    LLM,
    /// Tencent Cloud translation
    Tencent,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::DeepLX => write!(f, "deeplx"),
            ProviderType::LLM => write!(f, "llm"),
            ProviderType::Tencent => write!(f, "tencent"),
        }
    }
}

impl std::str::FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deeplx" => Ok(ProviderType::DeepLX),
            "llm" => Ok(ProviderType::LLM),
            "tencent" => Ok(ProviderType::Tencent),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// Static dispatch translator implementation enum
///
/// This enum provides static dispatch for all translator implementations,
#[derive(Debug)]
pub enum TranslatorImpl {
    DeepLX(DeepLXTranslator),
    LLM(MultiProviderTranslator),
    Tencent(TencentTranslator),
}

#[async_trait]
impl Translator for TranslatorImpl {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        match self {
            Self::DeepLX(t) => t.translate(texts, source_lang, target_lang).await,
            Self::LLM(t) => t.translate(texts, source_lang, target_lang).await,
            Self::Tencent(t) => t.translate(texts, source_lang, target_lang).await,
        }
    }

    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        match self {
            Self::DeepLX(t) => t.translate_single(text, source_lang, target_lang).await,
            Self::LLM(t) => t.translate_single(text, source_lang, target_lang).await,
            Self::Tencent(t) => t.translate_single(text, source_lang, target_lang).await,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::DeepLX(t) => t.name(),
            Self::LLM(t) => t.name(),
            Self::Tencent(t) => t.name(),
        }
    }

    async fn is_available(&self) -> bool {
        match self {
            Self::DeepLX(t) => t.is_available().await,
            Self::LLM(t) => t.is_available().await,
            Self::Tencent(t) => t.is_available().await,
        }
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        match self {
            Self::DeepLX(t) => t.supported_source_langs(),
            Self::LLM(t) => t.supported_source_langs(),
            Self::Tencent(t) => t.supported_source_langs(),
        }
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        match self {
            Self::DeepLX(t) => t.supported_target_langs(),
            Self::LLM(t) => t.supported_target_langs(),
            Self::Tencent(t) => t.supported_target_langs(),
        }
    }

    fn max_input_chars(&self) -> usize {
        match self {
            Self::DeepLX(t) => t.max_input_chars(),
            Self::LLM(t) => t.max_input_chars(),
            Self::Tencent(t) => t.max_input_chars(),
        }
    }

    async fn close(&self) -> Result<()> {
        match self {
            Self::DeepLX(t) => t.close().await,
            Self::LLM(t) => t.close().await,
            Self::Tencent(t) => t.close().await,
        }
    }

    fn set_reporter(&mut self, reporter: Arc<dyn Reporter>) {
        match self {
            Self::DeepLX(t) => t.set_reporter(reporter),
            Self::LLM(t) => t.set_reporter(reporter),
            Self::Tencent(t) => t.set_reporter(reporter),
        }
    }

    fn reporter(&self) -> Option<Arc<dyn Reporter>> {
        match self {
            Self::DeepLX(t) => t.reporter(),
            Self::LLM(t) => t.reporter(),
            Self::Tencent(t) => t.reporter(),
        }
    }
}

impl TranslatorImpl {
    /// Create a translator from the given configuration
    ///
    /// Note: For LLM provider, use `create_llm_multi_provider_translator` in mod.rs instead
    /// to enable multi-provider support with automatic filtering.
    pub fn from_config(config: &crate::translator::factory::TranslatorConfig) -> Result<Self> {
        info!(
            provider = ?config.provider,
            "Creating translator from configuration"
        );

        let translator = match config.provider {
            ProviderType::DeepLX => {
                debug!("Creating DeepLX translator");
                let deeplx_config = config.deeplx.clone().unwrap_or_default();
                let translator = DeepLXTranslator::new(deeplx_config)?;
                Ok(Self::DeepLX(translator))
            }
            ProviderType::LLM => {
                // LLM is now handled by create_llm_multi_provider_translator in mod.rs
                // This branch should not be reached when using the recommended API
                Err(crate::core::error::TranslateError::Config(
                    "LLM provider should be created using create_llm_multi_provider_translator"
                        .to_string(),
                ))
            }
            ProviderType::Tencent => {
                debug!("Creating Tencent translator");
                let tencent_config = config.tencent.clone().ok_or_else(|| {
                    crate::core::error::TranslateError::Config(
                        "Tencent configuration is required".to_string(),
                    )
                })?;
                let translator = TencentTranslator::new(tencent_config)?;
                Ok(Self::Tencent(translator))
            }
        };

        info!(
            provider = ?config.provider,
            "Translator created successfully"
        );

        translator
    }

    /// Get maximum input characters for this translator implementation
    pub fn max_input_chars(&self) -> usize {
        match self {
            Self::DeepLX(_) => 5000,
            Self::LLM(t) => t.max_input_chars(),
            Self::Tencent(_) => 6000,
        }
    }

    /// Check if this translator can handle text of given length
    pub fn can_handle(&self, text_len: usize) -> bool {
        let max = self.max_input_chars();
        max == 0 || text_len <= max
    }
}
