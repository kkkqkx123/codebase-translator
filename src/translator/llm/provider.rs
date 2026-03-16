use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::llm::LLMTranslator;
use crate::translator::Translator;

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealth {
    Healthy,
    Unhealthy,
}

/// Provider statistics
#[derive(Debug, Clone)]
pub struct ProviderStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens: u64,
    pub average_latency_ms: f64,
    pub last_request_time: Option<Instant>,
}

impl Default for ProviderStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            average_latency_ms: 0.0,
            last_request_time: None,
        }
    }
}

/// LLM provider wrapper with health tracking
#[derive(Clone)]
pub struct LLMProvider {
    id: String,
    translator: Arc<LLMTranslator>,
    health: Arc<RwLock<ProviderHealth>>,
    failure_count: Arc<AtomicU32>,
    stats: Arc<RwLock<ProviderStats>>,
    weight: u32,
    api_keys: Vec<String>,
    current_key_index: Arc<AtomicU32>,
}

impl LLMProvider {
    /// Create a new LLM provider
    pub fn new(config: &LLMProviderConfig) -> Result<Self> {
        let translator_config = crate::translator::common::LLMConfig {
            base_url: config.base_url.clone(),
            api_key: config.api_keys.first().cloned().unwrap_or_default(),
            model: config
                .models
                .first()
                .map(|m| m.name.clone())
                .unwrap_or_default(),
            proxy_url: config.proxy_url.clone(),
            timeout: config.timeout,
            max_tokens: config
                .models
                .first()
                .and_then(|m| m.max_tokens)
                .unwrap_or(4096) as i32,
            temperature: config
                .models
                .first()
                .and_then(|m| m.temperature)
                .unwrap_or(0.3) as f64,
            top_p: None,
            extra_headers: if config.extra_headers.is_empty() {
                None
            } else {
                Some(config.extra_headers.clone())
            },
            extra_params: if config.extra_params.is_empty() {
                None
            } else {
                Some(serde_json::Value::Object(
                    config
                        .extra_params
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                ))
            },
            max_retries: 3,
        };

        let translator = Arc::new(LLMTranslator::new(translator_config)?);

        Ok(Self {
            id: config.id.clone(),
            translator,
            health: Arc::new(RwLock::new(ProviderHealth::Healthy)),
            failure_count: Arc::new(AtomicU32::new(0)),
            stats: Arc::new(RwLock::new(ProviderStats::default())),
            weight: config.weight,
            api_keys: config.api_keys.clone(),
            current_key_index: Arc::new(AtomicU32::new(0)),
        })
    }

    /// Get provider ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get provider weight
    pub fn weight(&self) -> u32 {
        self.weight
    }

    /// Check if provider is healthy
    pub async fn is_healthy(&self) -> bool {
        *self.health.read().await == ProviderHealth::Healthy
    }

    /// Mark provider as healthy
    pub async fn mark_healthy(&self) {
        *self.health.write().await = ProviderHealth::Healthy;
        self.failure_count.store(0, Ordering::Relaxed);
        debug!("Provider {} marked as healthy", self.id);
    }

    /// Mark provider as unhealthy
    pub async fn mark_unhealthy(&self) {
        *self.health.write().await = ProviderHealth::Unhealthy;
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        warn!(
            "Provider {} marked as unhealthy (failure count: {})",
            self.id, count
        );
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get provider statistics
    pub async fn stats(&self) -> ProviderStats {
        self.stats.read().await.clone()
    }

    /// Get next API key (for rotation)
    pub fn next_api_key(&self) -> Option<String> {
        if self.api_keys.is_empty() {
            return None;
        }

        let index =
            self.current_key_index.fetch_add(1, Ordering::Relaxed) % self.api_keys.len() as u32;
        self.api_keys.get(index as usize).cloned()
    }

    /// Translate text
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = Instant::now();

        let result = self
            .translator
            .translate_single(text, source_lang, target_lang)
            .await;

        let latency = start_time.elapsed();

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.last_request_time = Some(Instant::now());

        match &result {
            Ok(_) => {
                stats.successful_requests += 1;
                let total_latency = stats.average_latency_ms * (stats.total_requests - 1) as f64;
                stats.average_latency_ms =
                    (total_latency + latency.as_millis() as f64) / stats.total_requests as f64;
            }
            Err(_) => {
                stats.failed_requests += 1;
            }
        }

        match result {
            Ok(translated_text) => Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text,
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                ..Default::default()
            }),
            Err(e) => Err(e),
        }
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<()> {
        let start_time = Instant::now();

        match self.translator.is_available().await {
            true => {
                let latency = start_time.elapsed();
                debug!(
                    "Provider {} health check passed (latency: {:?})",
                    self.id, latency
                );
                Ok(())
            }
            false => {
                let latency = start_time.elapsed();
                error!(
                    "Provider {} health check failed (latency: {:?})",
                    self.id, latency
                );
                Err(TranslateError::Translation(format!(
                    "Provider {} health check failed",
                    self.id
                )))
            }
        }
    }

    /// Get underlying translator
    pub fn translator(&self) -> &Arc<LLMTranslator> {
        &self.translator
    }

    /// Close provider and cleanup resources
    pub async fn close(&self) -> Result<()> {
        // Currently no resources to cleanup, but this allows for future extensions
        Ok(())
    }
}

/// Provider trait for LLM providers
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get provider ID
    fn id(&self) -> &str;

    /// Get provider weight
    fn weight(&self) -> u32;

    /// Check if provider is healthy
    async fn is_healthy(&self) -> bool;

    /// Mark provider as healthy
    async fn mark_healthy(&self);

    /// Mark provider as unhealthy
    async fn mark_unhealthy(&self);

    /// Translate text
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse>;

    /// Perform health check
    async fn health_check(&self) -> Result<()>;

    /// Close provider
    async fn close(&self) -> Result<()>;
}

/// Static dispatch provider implementation enum
///
/// This enum provides static dispatch for all provider implementations,
#[derive(Clone)]
pub enum ProviderImpl {
    LLM(LLMProvider),
}

#[async_trait]
impl Provider for ProviderImpl {
    fn id(&self) -> &str {
        match self {
            Self::LLM(p) => p.id(),
        }
    }

    fn weight(&self) -> u32 {
        match self {
            Self::LLM(p) => p.weight(),
        }
    }

    async fn is_healthy(&self) -> bool {
        match self {
            Self::LLM(p) => p.is_healthy().await,
        }
    }

    async fn mark_healthy(&self) {
        match self {
            Self::LLM(p) => p.mark_healthy().await,
        }
    }

    async fn mark_unhealthy(&self) {
        match self {
            Self::LLM(p) => p.mark_unhealthy().await,
        }
    }

    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        match self {
            Self::LLM(p) => p.translate(text, source_lang, target_lang).await,
        }
    }

    async fn health_check(&self) -> Result<()> {
        match self {
            Self::LLM(p) => p.health_check().await,
        }
    }

    async fn close(&self) -> Result<()> {
        match self {
            Self::LLM(p) => p.close().await,
        }
    }
}
