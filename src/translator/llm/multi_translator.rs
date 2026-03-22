use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::reporter::Reporter;
use crate::translator::common::TranslateResponse;
use crate::translator::llm::provider::LLMProvider;
use crate::translator::llm::routing::{ProviderRouter, SelectionStrategy};
use crate::translator::Translator;

/// Multi-provider LLM translator with weighted capacity routing
///
/// Routing strategy:
/// - Short texts (< threshold): Weighted distribution among all providers
/// - Long texts (>= threshold): Weighted distribution among capable providers
///
/// Each provider represents a single model with a fixed max_tokens limit.
///
/// Health management:
/// - Uses threshold-based health tracking (not immediate)
/// - Provider marked unhealthy after N consecutive failures
/// - Provider recovers after M consecutive successes
pub struct MultiProviderTranslator {
    router: ProviderRouter,
    max_retries: usize,
    reporter: Option<Arc<dyn Reporter>>,
}

impl std::fmt::Debug for MultiProviderTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiProviderTranslator")
            .field("router", &self.router)
            .field("max_retries", &self.max_retries)
            .field("has_reporter", &self.reporter.is_some())
            .finish()
    }
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
            reporter: None,
        })
    }

    /// Create a new multi-provider translator with specific selection strategy
    pub fn new_with_strategy(
        configs: &[LLMProviderConfig],
        max_retries: usize,
        strategy: SelectionStrategy,
    ) -> Result<Self> {
        let router = ProviderRouter::new_with_strategy(configs, strategy)?;
        let max_retries = if max_retries == 0 { 3 } else { max_retries };

        info!(
            "Created MultiProviderTranslator with {} providers, max_retries: {}, strategy: {:?}",
            router.providers().len(),
            max_retries,
            strategy
        );

        Ok(Self {
            router,
            max_retries,
            reporter: None,
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
                // Note: Health is now managed internally by provider with threshold
                // No need to manually mark as healthy
                Ok(response)
            }
            Err(e) => {
                let latency = start_time.elapsed();
                error!(
                    "Translation failed with provider {} in {:?}: {}",
                    provider_id, latency, e
                );
                // Note: Health is now managed internally by provider with threshold
                // No need to manually mark as unhealthy

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
            .into_iter()
            .filter(|p| p.can_handle(text_len) && p.id() != exclude_provider)
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

            // Check if provider is healthy before trying
            if !provider.is_healthy().await {
                warn!(
                    "Skipping unhealthy provider {} (attempt {}/{})",
                    provider_id,
                    idx + 1,
                    max_attempts
                );
                continue;
            }

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
                    // Note: Health is now managed internally
                    return Ok(response);
                }
                Err(e) => {
                    let latency = attempt_start.elapsed();
                    error!(
                        "Alternative provider {} failed in {:?}: {}",
                        provider_id, latency, e
                    );
                    // Note: Health is now managed internally by provider
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
            "strategy": format!("{:?}", self.router.strategy()),
            "providers": self.router.providers().iter().map(|p| {
                serde_json::json!({
                    "id": p.id(),
                    "max_chars": p.max_input_chars(),
                    "rate_limit": p.rate_limit(),
                })
            }).collect::<Vec<_>>(),
        })
    }

    /// Get selection strategy
    pub fn selection_strategy(&self) -> SelectionStrategy {
        self.router.strategy()
    }
}

#[async_trait]
impl Translator for MultiProviderTranslator {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let response = self
                .translate_with_failover(text, source_lang, target_lang)
                .await?;
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

    fn set_reporter(&mut self, reporter: Arc<dyn Reporter>) {
        self.reporter = Some(reporter.clone());
        // Also set reporter on all providers
        for provider in self.router.providers() {
            if let Some(provider) = Arc::get_mut(&mut provider.clone()) {
                provider.set_reporter(reporter.clone());
            }
        }
    }

    fn reporter(&self) -> Option<Arc<dyn Reporter>> {
        self.reporter.clone()
    }
}
