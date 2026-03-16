//! Translator trait definition
//!
//! This module defines the Translator trait for translation services.

use async_trait::async_trait;

use crate::core::error::Result;
use crate::translator::deeplx::DeepLXTranslator;
use crate::translator::llm::LLMTranslator;
use crate::translator::tencent::TencentTranslator;

/// Translator trait for translation services
#[async_trait]
pub trait Translator: Send + Sync {
    /// Translate a batch of texts
    ///
    /// # Arguments
    /// * `texts` - Texts to translate
    /// * `target_lang` - Target language code (e.g., "en", "zh")
    ///
    /// # Returns
    /// Translated texts in the same order as input
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>>;

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

    /// Close and cleanup resources
    async fn close(&self) -> Result<()> {
        Ok(())
    }
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
pub enum TranslatorImpl {
    DeepLX(DeepLXTranslator),
    LLM(LLMTranslator),
    Tencent(TencentTranslator),
}

#[async_trait]
impl Translator for TranslatorImpl {
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        match self {
            Self::DeepLX(t) => t.translate(texts, target_lang).await,
            Self::LLM(t) => t.translate(texts, target_lang).await,
            Self::Tencent(t) => t.translate(texts, target_lang).await,
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

    async fn close(&self) -> Result<()> {
        match self {
            Self::DeepLX(t) => t.close().await,
            Self::LLM(t) => t.close().await,
            Self::Tencent(t) => t.close().await,
        }
    }
}

impl TranslatorImpl {
    /// Create a translator from the given configuration
    pub fn from_config(config: &crate::translator::factory::TranslatorConfig) -> Result<Self> {
        match config.provider {
            ProviderType::DeepLX => {
                let deeplx_config = config.deeplx.clone().unwrap_or_default();
                let translator = DeepLXTranslator::new(deeplx_config)?;
                Ok(Self::DeepLX(translator))
            }
            ProviderType::LLM => {
                let llm_config = config.llm.clone().ok_or_else(|| {
                    crate::core::error::TranslateError::Config(
                        "LLM configuration is required".to_string(),
                    )
                })?;
                let translator = LLMTranslator::new(llm_config)?;
                Ok(Self::LLM(translator))
            }
            ProviderType::Tencent => {
                let tencent_config = config.tencent.clone().ok_or_else(|| {
                    crate::core::error::TranslateError::Config(
                        "Tencent configuration is required".to_string(),
                    )
                })?;
                let translator = TencentTranslator::new(tencent_config)?;
                Ok(Self::Tencent(translator))
            }
        }
    }
}
