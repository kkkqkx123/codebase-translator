use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::llm::provider::LLMProvider;

/// Rotation strategy for provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationStrategy {
    RoundRobin,
    Weighted,
}

impl std::str::FromStr for RotationStrategy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "round_robin" | "roundrobin" => Ok(RotationStrategy::RoundRobin),
            "weighted" => Ok(RotationStrategy::Weighted),
            _ => Err(format!("Unknown rotation strategy: {}", s)),
        }
    }
}

/// Provider pool configuration
#[derive(Debug, Clone)]
pub struct ProviderPoolConfig {
    pub strategy: RotationStrategy,
    pub health_check_enabled: bool,
    pub health_check_interval: Duration,
    pub health_check_timeout: Duration,
    pub failure_threshold: u32,
    pub recovery_interval: Duration,
}

impl Default for ProviderPoolConfig {
    fn default() -> Self {
        Self {
            strategy: RotationStrategy::RoundRobin,
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_interval: Duration::from_secs(300),
        }
    }
}

/// Provider pool for managing multiple LLM providers
pub struct ProviderPool {
    providers: Arc<RwLock<Vec<Arc<LLMProvider>>>>,
    strategy: RotationStrategy,
    current_index: Arc<std::sync::atomic::AtomicU64>,
    total_weight: Arc<std::sync::atomic::AtomicU32>,
    config: ProviderPoolConfig,
    health_check_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    stop_signal: Arc<tokio::sync::Notify>,
}

impl ProviderPool {
    /// Create a new provider pool
    pub async fn new(configs: &[LLMProviderConfig], config: ProviderPoolConfig) -> Result<Self> {
        let mut providers: Vec<Arc<LLMProvider>> = Vec::new();
        let mut total_weight = 0u32;

        for provider_config in configs {
            let provider = Arc::new(LLMProvider::new(provider_config)?);
            total_weight += provider.weight();
            providers.push(provider);
        }

        if providers.is_empty() {
            return Err(TranslateError::Config(
                "No valid LLM providers configured".to_string(),
            ));
        }

        let pool = Self {
            providers: Arc::new(RwLock::new(providers)),
            strategy: config.strategy,
            current_index: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_weight: Arc::new(std::sync::atomic::AtomicU32::new(total_weight)),
            config,
            health_check_handle: Arc::new(RwLock::new(None)),
            stop_signal: Arc::new(tokio::sync::Notify::new()),
        };

        if pool.config.health_check_enabled {
            pool.start_health_check().await;
        }

        info!(
            "Provider pool created with {} providers, strategy: {:?}",
            pool.providers.read().await.len(),
            pool.strategy
        );

        Ok(pool)
    }

    /// Start health check routine
    async fn start_health_check(&self) {
        let providers = self.providers.clone();
        let config = self.config.clone();
        let stop_signal = self.stop_signal.clone();

        let handle = tokio::spawn(async move {
            let mut health_check_interval = tokio::time::interval(config.health_check_interval);
            let mut recovery_interval = tokio::time::interval(config.recovery_interval);

            loop {
                tokio::select! {
                    _ = stop_signal.notified() => {
                        info!("Health check stopped");
                        break;
                    }
                    _ = health_check_interval.tick() => {
                        Self::check_all_providers(&providers, config.health_check_timeout).await;
                    }
                    _ = recovery_interval.tick() => {
                        Self::try_recover_unhealthy_providers(&providers, config.health_check_timeout).await;
                    }
                }
            }
        });

        *self.health_check_handle.write().await = Some(handle);
    }

    /// Check all providers health
    async fn check_all_providers(
        providers: &Arc<RwLock<Vec<Arc<LLMProvider>>>>,
        timeout: Duration,
    ) {
        let providers_snapshot = providers.read().await.clone();

        for provider in providers_snapshot {
            let provider_clone = provider.clone();
            tokio::spawn(async move {
                let check_result =
                    tokio::time::timeout(timeout, provider_clone.health_check()).await;

                match check_result {
                    Ok(Ok(_)) => {
                        provider_clone.mark_healthy().await;
                    }
                    Ok(Err(_)) | Err(_) => {
                        provider_clone.mark_unhealthy().await;
                    }
                }
            });
        }
    }

    /// Try to recover unhealthy providers
    async fn try_recover_unhealthy_providers(
        providers: &Arc<RwLock<Vec<Arc<LLMProvider>>>>,
        timeout: Duration,
    ) {
        let providers_snapshot = providers.read().await.clone();

        for provider in providers_snapshot {
            if !provider.is_healthy().await {
                let provider_clone = provider.clone();
                tokio::spawn(async move {
                    let check_result =
                        tokio::time::timeout(timeout, provider_clone.health_check()).await;

                    if check_result.is_ok() && check_result.unwrap().is_ok() {
                        provider_clone.mark_healthy().await;
                    }
                });
            }
        }
    }

    /// Get next provider based on strategy
    pub async fn get_provider(&self) -> Result<Arc<LLMProvider>> {
        match self.strategy {
            RotationStrategy::RoundRobin => self.get_round_robin_provider().await,
            RotationStrategy::Weighted => self.get_weighted_provider().await,
        }
    }

    /// Get provider by round-robin strategy
    async fn get_round_robin_provider(&self) -> Result<Arc<LLMProvider>> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(TranslateError::Translation(
                "No available providers".to_string(),
            ));
        }

        let total_providers = providers.len();
        let mut attempts = 0;

        while attempts < total_providers {
            let index =
                self.current_index
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed) as usize
                    % total_providers;

            if providers[index].is_healthy().await {
                return Ok(providers[index].clone());
            }

            attempts += 1;
        }

        Err(TranslateError::Translation(
            "No healthy providers available".to_string(),
        ))
    }

    /// Get provider by weighted strategy using random selection
    async fn get_weighted_provider(&self) -> Result<Arc<LLMProvider>> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(TranslateError::Translation(
                "No available providers".to_string(),
            ));
        }

        let total_weight = self.total_weight.load(std::sync::atomic::Ordering::Relaxed);
        if total_weight == 0 {
            return self.get_round_robin_provider().await;
        }

        // Use random selection for weighted distribution
        let target_weight = rand::random::<u32>() % total_weight;

        let mut current_weight = 0u32;
        for provider in providers.iter() {
            if !provider.is_healthy().await {
                continue;
            }

            current_weight += provider.weight();
            if target_weight < current_weight {
                tracing::debug!(
                    "Weighted selected provider {} with weight {}",
                    provider.id(),
                    provider.weight()
                );
                return Ok(provider.clone());
            }
        }

        Err(TranslateError::Translation(
            "No healthy providers available".to_string(),
        ))
    }

    /// Get provider by ID
    pub async fn get_provider_by_id(&self, id: &str) -> Result<Arc<LLMProvider>> {
        let providers = self.providers.read().await;

        for provider in providers.iter() {
            if provider.id() == id {
                return Ok(provider.clone());
            }
        }

        Err(TranslateError::Translation(format!(
            "Provider not found: {}",
            id
        )))
    }

    /// Get all providers
    pub async fn get_all_providers(&self) -> Vec<Arc<LLMProvider>> {
        self.providers.read().await.clone()
    }

    /// Get healthy providers
    pub async fn get_healthy_providers(&self) -> Vec<Arc<LLMProvider>> {
        let providers = self.providers.read().await;
        let mut healthy_providers = Vec::new();

        for provider in providers.iter() {
            if provider.is_healthy().await {
                healthy_providers.push(provider.clone());
            }
        }

        healthy_providers
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let providers = self.providers.read().await;
        let total = providers.len();
        let mut healthy = 0usize;

        for provider in providers.iter() {
            if provider.is_healthy().await {
                healthy += 1;
            }
        }

        let mut stats = HashMap::new();
        stats.insert(
            "total_providers".to_string(),
            serde_json::Value::Number(total.into()),
        );
        stats.insert(
            "healthy_providers".to_string(),
            serde_json::Value::Number(healthy.into()),
        );
        stats.insert(
            "strategy".to_string(),
            serde_json::Value::String(format!("{:?}", self.strategy)),
        );
        stats.insert(
            "total_weight".to_string(),
            serde_json::Value::Number(
                self.total_weight
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .into(),
            ),
        );

        stats
    }

    /// Stop health check
    pub async fn stop(&self) {
        self.stop_signal.notify_waiters();
        if let Some(handle) = self.health_check_handle.write().await.take() {
            handle.abort();
        }
    }
}

impl Drop for ProviderPool {
    fn drop(&mut self) {
        let stop_signal = self.stop_signal.clone();
        let health_check_handle = self.health_check_handle.clone();

        tokio::spawn(async move {
            stop_signal.notify_waiters();
            if let Some(handle) = health_check_handle.write().await.take() {
                handle.abort();
            }
        });
    }
}
