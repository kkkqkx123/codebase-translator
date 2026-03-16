use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::llm::pool::ProviderPool;
use crate::translator::llm::provider::Provider;
use crate::translator::Translator;

/// Multi-provider LLM translator with automatic failover
pub struct MultiProviderTranslator {
    pool: Arc<ProviderPool>,
    max_retries: usize,
}

impl MultiProviderTranslator {
    /// Create a new multi-provider translator
    pub async fn new(pool: Arc<ProviderPool>, max_retries: usize) -> Self {
        let max_retries = if max_retries == 0 { 3 } else { max_retries };
        Self { pool, max_retries }
    }

    /// Translate with automatic failover
    async fn translate_with_failover(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let mut last_error = None;
        let mut attempted_providers = std::collections::HashSet::new();

        for attempt in 0..=self.max_retries {
            let provider = match self.pool.get_provider().await {
                Ok(p) => p,
                Err(e) => {
                    if attempted_providers.is_empty() {
                        return Err(e);
                    }
                    break;
                }
            };

            let provider_id = provider.id().to_string();

            if attempted_providers.contains(&provider_id) {
                if attempted_providers.len() >= self.pool.get_all_providers().await.len() {
                    break;
                }
                continue;
            }

            attempted_providers.insert(provider_id.clone());

            debug!(
                "Attempting translation with provider {} (attempt {}/{})",
                provider_id,
                attempt + 1,
                self.max_retries + 1
            );

            match provider.translate(text, source_lang, target_lang).await {
                Ok(response) => {
                    info!("Translation succeeded with provider {}", provider_id);
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    error!("Translation failed with provider {}: {}", provider_id, e);

                    if !Self::is_retryable_error(&e) {
                        return Err(e);
                    }

                    provider.mark_unhealthy().await;

                    if attempt < self.max_retries {
                        let delay = Self::calculate_backoff(attempt);
                        debug!("Waiting {:?} before retry", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| TranslateError::Translation("All providers failed".to_string())))
    }

    /// Check if error is retryable
    fn is_retryable_error(error: &TranslateError) -> bool {
        error.is_retryable()
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff(attempt: usize) -> Duration {
        TranslateError::calculate_backoff(attempt)
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.pool.get_stats().await
    }

    /// Force health check on all providers
    pub async fn force_health_check(&self) -> Result<()> {
        let providers = self.pool.get_all_providers().await;

        for provider in providers {
            match provider.health_check().await {
                Ok(_) => provider.mark_healthy().await,
                Err(_) => provider.mark_unhealthy().await,
            }
        }

        Ok(())
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
        match self.pool.get_healthy_providers().await.len() {
            0 => false,
            _ => true,
        }
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        vec!["AUTO"]
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        vec!["EN", "ZH", "JA", "KO", "DE", "FR", "ES", "IT", "PT", "RU"]
    }

    async fn close(&self) -> Result<()> {
        self.pool.stop().await;
        Ok(())
    }
}
