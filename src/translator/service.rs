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
    batch_translator: Option<Arc<BatchTranslator>>,
    translator: Option<Arc<TranslatorImpl>>,
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
            batch_translator: None,
            translator: Some(Arc::new(translator)),
        })
    }

    /// Create a new translation service with batch translator
    ///
    /// # Arguments
    /// * `batch_translator` - Batch translator with rate limiting and retry logic
    ///
    /// # Returns
    /// A new TranslationService instance
    pub fn with_batch_translator(batch_translator: Arc<BatchTranslator>) -> Result<Self> {
        debug!("Creating translation service with batch translator");
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            TranslateError::Translation(format!("Failed to create Tokio runtime: {}", e))
        })?;

        info!("Translation service created with batch translator");
        Ok(Self {
            runtime,
            batch_translator: Some(batch_translator),
            translator: None,
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

        if let Some(ref batch_translator) = self.batch_translator {
            let texts = texts.to_vec();
            let target_lang = target_lang.to_string();
            let batch_translator = batch_translator.clone();

            let result = self.runtime.block_on(async move {
                batch_translator
                    .translate_batch(&texts, &target_lang)
                    .await
            })?;

            debug!(
                translated_count = result.results.len(),
                "Batch translation completed with rate limiting"
            );
            Ok(result.results.into_iter().map(|r| r.translated_text).collect())
        } else if let Some(ref translator) = self.translator {
            let texts = texts.to_vec();
            let target_lang = target_lang.to_string();
            let translator = translator.clone();

            let result = self
                .runtime
                .block_on(async move { translator.translate(&texts, &target_lang).await })?;

            debug!(
                translated_count = result.len(),
                "Batch translation completed"
            );
            Ok(result)
        } else {
            Err(TranslateError::Translation(
                "No translator configured".to_string(),
            ))
        }
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

        if let Some(ref batch_translator) = self.batch_translator {
            let batch_translator = batch_translator.clone();
            self.runtime.block_on(async move {
                let result = batch_translator
                    .translate_batch(&[text.clone()], &target_lang)
                    .await?;
                if let Some(first) = result.results.first() {
                    Ok(first.translated_text.clone())
                } else {
                    Err(TranslateError::Translation(
                        "No translation result".to_string(),
                    ))
                }
            })
        } else if let Some(ref translator) = self.translator {
            let translator = translator.clone();
            self.runtime.block_on(async move {
                translator
                    .translate_single(&text, &source_lang, &target_lang)
                    .await
            })
        } else {
            Err(TranslateError::Translation(
                "No translator configured".to_string(),
            ))
        }
    }

    /// Check if the translation service is available
    pub fn is_available(&self) -> bool {
        if let Some(ref batch_translator) = self.batch_translator {
            self.runtime
                .block_on(async move { batch_translator.name() != "" })
        } else if let Some(ref translator) = self.translator {
            let translator = translator.clone();
            self.runtime
                .block_on(async move { translator.is_available().await })
        } else {
            false
        }
    }

    /// Get the translator name
    pub fn name(&self) -> String {
        if let Some(ref batch_translator) = self.batch_translator {
            batch_translator.name().to_string()
        } else if let Some(ref translator) = self.translator {
            translator.name().to_string()
        } else {
            "unknown".to_string()
        }
    }
}

impl Drop for TranslationService {
    fn drop(&mut self) {
        // Clean up translator resources
        if let Some(translator) = self.translator.clone() {
            let _ = self.runtime.block_on(async move { translator.close().await });
        }
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
