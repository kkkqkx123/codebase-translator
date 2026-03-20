//! Translation Service - Async isolation layer
//!
//! This module provides a synchronous interface to the asynchronous translation
//! functionality. It encapsulates the async runtime and provides a clean boundary
//! between the sync and async parts of the codebase.
//!
//! Uses static dispatch via TranslatorImpl for better performance.

use std::sync::Arc;
use tracing::{debug, info};

use crate::core::error::{Result, TranslateError};
use crate::translator::batch::BatchTranslator;
use crate::translator::common::{BatchOptions, BatchResult};
use crate::translator::factory::{create_translator_from_config, TranslatorConfig};
use crate::translator::{Translator, TranslatorImpl};

/// Synchronous translation service that internally manages async operations
///
/// This struct provides a sync interface while handling all async translation
/// operations internally using a dedicated Tokio runtime.
/// Uses static dispatch via TranslatorImpl for better performance.
pub struct TranslationService {
    runtime: tokio::runtime::Runtime,
    translator: Arc<TranslatorImpl>,
}

impl TranslationService {
    /// Create a new translation service with the given configuration
    ///
    /// # Arguments
    /// * `config` - Translator configuration
    ///
    /// # Returns
    /// A new TranslationService instance
    pub fn new(config: TranslatorConfig) -> Result<Self> {
        debug!("Creating translation service");
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            TranslateError::Translation(format!("Failed to create Tokio runtime: {}", e))
        })?;

        debug!("Creating translator from config");
        let translator = runtime.block_on(async {
            // Note: factory creation is sync, but we prepare for async initialization
            create_translator_from_config(&config)
        })?;

        info!("Translation service created successfully");
        Ok(Self {
            runtime,
            translator: Arc::new(translator),
        })
    }

    /// Translate a batch of texts
    ///
    /// # Arguments
    /// * `texts` - Texts to translate
    /// * `target_lang` - Target language code
    ///
    /// # Returns
    /// Translated texts in the same order as input
    pub fn translate_batch(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        debug!(
            texts_count = texts.len(),
            target_lang = %target_lang,
            "Translating batch of texts"
        );
        let texts = texts.to_vec();
        let target_lang = target_lang.to_string();
        let translator = self.translator.clone();

        let result = self
            .runtime
            .block_on(async move { translator.translate(&texts, &target_lang).await })?;

        debug!(
            translated_count = result.len(),
            "Batch translation completed"
        );
        Ok(result)
    }

    /// Translate a single text
    ///
    /// # Arguments
    /// * `text` - Text to translate
    /// * `source_lang` - Source language code (or "AUTO")
    /// * `target_lang` - Target language code
    ///
    /// # Returns
    /// Translated text
    pub fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let text = text.to_string();
        let source_lang = source_lang.to_string();
        let target_lang = target_lang.to_string();
        let translator = self.translator.clone();

        self.runtime.block_on(async move {
            translator
                .translate_single(&text, &source_lang, &target_lang)
                .await
        })
    }

    /// Check if the translation service is available
    pub fn is_available(&self) -> bool {
        let translator = self.translator.clone();
        self.runtime
            .block_on(async move { translator.is_available().await })
    }

    /// Get the translator name
    pub fn name(&self) -> String {
        self.translator.name().to_string()
    }

    /// Get supported source languages
    pub fn supported_source_langs(&self) -> Vec<String> {
        self.translator
            .supported_source_langs()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get supported target languages
    pub fn supported_target_langs(&self) -> Vec<String> {
        self.translator
            .supported_target_langs()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Drop for TranslationService {
    fn drop(&mut self) {
        // Clean up translator resources
        let translator = self.translator.clone();
        let _ = self
            .runtime
            .block_on(async move { translator.close().await });
    }
}

/// Batch translation service with rate limiting
///
/// This struct provides synchronous batch translation with built-in
/// rate limiting and retry logic.
/// Uses static dispatch via TranslatorImpl for better performance.
pub struct BatchTranslationService {
    runtime: tokio::runtime::Runtime,
    batch_translator: BatchTranslator,
}

impl BatchTranslationService {
    /// Create a new batch translation service
    pub fn new(translator: Arc<TranslatorImpl>, options: BatchOptions) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            TranslateError::Translation(format!("Failed to create Tokio runtime: {}", e))
        })?;

        let batch_translator = BatchTranslator::new(translator, options);

        Ok(Self {
            runtime,
            batch_translator,
        })
    }

    /// Translate a batch of texts with rate limiting
    pub fn translate_batch(&self, texts: &[String], target_lang: &str) -> Result<BatchResult> {
        let texts = texts.to_vec();
        let target_lang = target_lang.to_string();

        self.runtime.block_on(async move {
            self.batch_translator
                .translate_batch(&texts, &target_lang)
                .await
        })
    }

    /// Set rate limit dynamically
    pub fn set_rate_limit(&self, requests_per_second: u32) {
        self.runtime.block_on(async move {
            self.batch_translator
                .set_rate_limit(requests_per_second)
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_translation_service_creation() {
        // This test would require a valid DeepLX endpoint
        // Skipping for unit tests
    }

    #[test]
    fn test_batch_translation_service() {
        // This test would require a valid translator
        // Skipping for unit tests
    }
}
