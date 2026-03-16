use async_trait::async_trait;
use tracing::{debug, error, info};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::llm::provider::Provider;
use crate::translator::llm::routing::ProviderRouter;
use crate::translator::Translator;

/// Multi-provider LLM translator with weighted capacity routing
///
/// Routing strategy:
/// - Short texts (< threshold): Weighted distribution among all providers
/// - Long texts (>= threshold): Weighted distribution among capable providers
///
/// Each provider represents a single model with a fixed max_tokens limit.
pub struct MultiProviderTranslator {
    router: ProviderRouter,
    max_retries: usize,
}

impl MultiProviderTranslator {
    /// Create a new multi-provider translator
    pub fn new(configs: &[LLMProviderConfig], max_retries: usize) -> Result<Self> {
        let router = ProviderRouter::new(configs)?;
        let max_retries = if max_retries == 0 { 3 } else { max_retries };

        info!(
            "Created MultiProviderTranslator with {} providers, max_retries: {}",
            router.providers().len(),
            max_retries
        );

        Ok(Self {
            router,
            max_retries,
        })
    }

    /// Translate with automatic failover
    async fn translate_with_failover(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let text_len = text.len();

        // Select provider based on capacity
        let provider = match self.router.select_provider(text_len) {
            Some(p) => p.provider().clone(),
            None => {
                return Err(TranslateError::Translation(format!(
                    "No provider can handle text of length {}. Maximum capacity: {}",
                    text_len,
                    self.router.max_capacity()
                )));
            }
        };

        let provider_id = provider.id().to_string();
        debug!(
            "Selected provider {} for text length {}",
            provider_id, text_len
        );

        // Try translation with the selected provider
        match provider.translate(text, source_lang, target_lang).await {
            Ok(response) => {
                info!("Translation succeeded with provider {}", provider_id);
                Ok(response)
            }
            Err(e) => {
                error!("Translation failed with provider {}: {}", provider_id, e);

                // If retryable, try other providers that can handle this text
                if Self::is_retryable_error(&e) {
                    self.try_other_providers(text, source_lang, target_lang, &provider_id)
                        .await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Try other providers when the primary fails
    async fn try_other_providers(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
        exclude_provider: &str,
    ) -> Result<TranslateResponse> {
        let text_len = text.len();

        // Find other providers that can handle this text
        let other_providers: Vec<_> = self
            .router
            .providers()
            .iter()
            .filter(|p| p.can_handle(text_len) && p.provider().id() != exclude_provider)
            .collect();

        if other_providers.is_empty() {
            return Err(TranslateError::Translation(
                "No alternative providers available".to_string(),
            ));
        }

        // Try each alternative provider
        for (idx, provider) in other_providers.iter().enumerate() {
            let provider_id = provider.provider().id().to_string();
            debug!(
                "Trying alternative provider {} (attempt {})",
                provider_id,
                idx + 1
            );

            match provider
                .provider()
                .translate(text, source_lang, target_lang)
                .await
            {
                Ok(response) => {
                    info!(
                        "Translation succeeded with alternative provider {}",
                        provider_id
                    );
                    return Ok(response);
                }
                Err(e) => {
                    error!("Alternative provider {} failed: {}", provider_id, e);
                }
            }
        }

        Err(TranslateError::Translation(
            "All alternative providers failed".to_string(),
        ))
    }

    /// Check if error is retryable
    fn is_retryable_error(error: &TranslateError) -> bool {
        error.is_retryable()
    }

    /// Get router statistics
    pub fn get_router_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_providers": self.router.providers().len(),
            "max_capacity": self.router.max_capacity(),
            "providers": self.router.providers().iter().map(|p| {
                serde_json::json!({
                    "id": p.provider().id(),
                    "max_chars": p.max_chars(),
                    "weight": p.weight(),
                })
            }).collect::<Vec<_>>(),
        })
    }
}

#[async_trait]
impl Translator for MultiProviderTranslator {
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let response = self.translate_with_failover(text, "", target_lang).await?;
            results.push(response.translated_text);
        }

        Ok(results)
    }

    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let response = self
            .translate_with_failover(text, source_lang, target_lang)
            .await?;
        Ok(response.translated_text)
    }

    fn name(&self) -> &str {
        "llm-multi-provider"
    }

    async fn is_available(&self) -> bool {
        // Check if any provider can handle at least some text
        self.router.max_capacity() > 0
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        vec!["AUTO"]
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        vec!["EN", "ZH", "JA", "KO", "DE", "FR", "ES", "IT", "PT", "RU"]
    }

    fn max_input_chars(&self) -> usize {
        // Return the maximum capacity among all providers
        self.router.max_capacity()
    }

    async fn close(&self) -> Result<()> {
        // No async cleanup needed with the new router-based approach
        Ok(())
    }
}
