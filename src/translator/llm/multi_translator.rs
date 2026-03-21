use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::llm::provider::LLMProvider;
use crate::translator::llm::routing::ProviderRouter;
use crate::translator::Translator;

/// Multi-provider LLM translator with weighted capacity routing
///
/// Routing strategy:
/// - Short texts (< threshold): Weighted distribution among all providers
/// - Long texts (>= threshold): Weighted distribution among capable providers
///
/// Each provider represents a single model with a fixed max_tokens limit.
#[derive(Debug)]
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
        let start_time = std::time::Instant::now();
        let text_len = text.len();

        // Select provider based on capacity
        let provider = match self.router.select_provider(text_len) {
            Some(p) => p.clone(),
            None => {
                error!(
                    "No provider can handle text of length {}. Maximum capacity: {}",
                    text_len,
                    self.router.max_capacity()
                );
                return Err(TranslateError::Translation(format!(
                    "No provider can handle text of length {}. Maximum capacity: {}",
                    text_len,
                    self.router.max_capacity()
                )));
            }
        };

        let provider_id = provider.id().to_string();
        debug!(
            "Selected provider {} for text length {} ({} chars)",
            provider_id, text_len, text_len
        );

        // Try translation with the selected provider
        match provider.translate(text, source_lang, target_lang).await {
            Ok(response) => {
                let latency = start_time.elapsed();
                info!(
                    "Translation succeeded with provider {} in {:?} (text: {} chars)",
                    provider_id, latency, text_len
                );
                // Mark provider as healthy on success
                provider.mark_healthy().await;
                Ok(response)
            }
            Err(e) => {
                let latency = start_time.elapsed();
                error!(
                    "Translation failed with provider {} in {:?}: {}",
                    provider_id, latency, e
                );
                // Immediately mark provider as unhealthy on failure
                provider.mark_unhealthy().await;

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
        let start_time = std::time::Instant::now();
        let text_len = text.len();

        // Find other providers that can handle this text
        let other_providers: Vec<Arc<LLMProvider>> = self
            .router
            .providers()
            .iter()
            .filter(|p| p.can_handle(text_len) && p.id() != exclude_provider)
            .cloned()
            .collect();

        info!(
            "Attempting failover with {} alternative providers (excluded: {})",
            other_providers.len(),
            exclude_provider
        );

        if other_providers.is_empty() {
            error!("No alternative providers available for failover");
            return Err(TranslateError::Translation(
                "No alternative providers available".to_string(),
            ));
        }

        // Try each alternative provider (limited by max_retries)
        let max_attempts = self.max_retries.min(other_providers.len());
        for (idx, provider) in other_providers.iter().take(max_attempts).enumerate() {
            let provider_id = provider.id().to_string();
            let attempt_start = std::time::Instant::now();
            debug!(
                "Trying alternative provider {} (attempt {}/{}, capacity: {} chars)",
                provider_id,
                idx + 1,
                max_attempts,
                provider.max_input_chars()
            );

            match provider.translate(text, source_lang, target_lang).await {
                Ok(response) => {
                    let latency = attempt_start.elapsed();
                    info!(
                        "Translation succeeded with alternative provider {} in {:?}",
                        provider_id, latency
                    );
                    // Mark provider as healthy on success
                    provider.mark_healthy().await;
                    return Ok(response);
                }
                Err(e) => {
                    let latency = attempt_start.elapsed();
                    error!(
                        "Alternative provider {} failed in {:?}: {}",
                        provider_id, latency, e
                    );
                    // Immediately mark provider as unhealthy on failure
                    provider.mark_unhealthy().await;
                }
            }
        }

        let total_latency = start_time.elapsed();
        error!(
            "All alternative providers failed after {:?} ({} attempts made)",
            total_latency, max_attempts
        );
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
                    "id": p.id(),
                    "max_chars": p.max_input_chars(),
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
