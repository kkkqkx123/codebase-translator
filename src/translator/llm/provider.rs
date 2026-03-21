use async_trait::async_trait;
use regex::Regex;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::LLMProviderConfig;
use crate::core::error::{Result, TranslateError};
use crate::translator::common::TranslateResponse;
use crate::translator::Translator;

// ============================================================================
// Chat API Types
// ============================================================================

/// Chat message
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

/// Chat completion request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Chat completion response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    error: Option<ApiError>,
}

/// Choice in completion response
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// API error
#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

// ============================================================================
// Provider Health and Stats
// ============================================================================

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

// ============================================================================
// LLM Provider
// ============================================================================

/// LLM Provider with integrated HTTP client, health tracking, and routing capabilities
///
/// This struct combines the functionality of the former LLMTranslator and LLMProvider,
/// eliminating one layer of abstraction for a cleaner architecture.
#[derive(Clone)]
pub struct LLMProvider {
    // Identification
    id: String,
    name: String,

    // HTTP Client
    client: Client,

    // Configuration
    base_url: String,
    api_keys: Vec<String>,
    model: String,
    max_tokens: i32,
    temperature: f64,
    proxy_url: Option<String>,
    timeout: u64,
    extra_headers: Option<std::collections::HashMap<String, String>>,
    extra_params: Option<serde_json::Map<String, serde_json::Value>>,

    // Capacity
    max_input_chars: usize,

    // State
    health: Arc<RwLock<ProviderHealth>>,
    failure_count: Arc<AtomicU32>,
    stats: Arc<RwLock<ProviderStats>>,
    weight: u32,
    current_key_index: Arc<AtomicU32>,
}

impl std::fmt::Debug for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMProvider")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("max_input_chars", &self.max_input_chars)
            .field("weight", &self.weight)
            .finish_non_exhaustive()
    }
}

impl LLMProvider {
    /// Create a new LLM provider from configuration
    pub fn new(config: &LLMProviderConfig) -> Result<Self> {
        // Validate required fields
        if config.id.is_empty() {
            return Err(TranslateError::Config(
                "Provider ID cannot be empty".to_string(),
            ));
        }
        if config.base_url.is_empty() {
            return Err(TranslateError::Config(format!(
                "base_url is required for provider {}",
                config.id
            )));
        }
        if config.api_keys.is_empty() {
            return Err(TranslateError::Config(format!(
                "At least one API key is required for provider {}",
                config.id
            )));
        }

        // Determine model to use
        let model = if !config.model.is_empty() {
            config.model.clone()
        } else if !config.model_list.is_empty() {
            config.model_list[0].clone()
        } else {
            return Err(TranslateError::Config(format!(
                "model or model_list is required for provider {}",
                config.id
            )));
        };

        // Build HTTP client
        let timeout = Duration::from_secs(config.timeout.clamp(1, 600));
        let client_builder = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10);

        let client_builder = if let Some(ref proxy_url) = config.proxy_url {
            if !proxy_url.is_empty() {
                match Proxy::all(proxy_url) {
                    Ok(proxy) => client_builder.proxy(proxy),
                    Err(e) => {
                        warn!("Failed to set proxy for provider {}: {}", config.id, e);
                        client_builder
                    }
                }
            } else {
                client_builder
            }
        } else {
            client_builder
        };

        let client = client_builder
            .build()
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        // Calculate max input characters based on max_tokens
        // Reserve 20% of tokens for the response, use 4 chars per token as estimate
        let max_input_chars = ((config.max_tokens as f64 * 0.8) * 4.0) as usize;

        // Convert extra_headers and extra_params
        let extra_headers = if config.extra_headers.is_empty() {
            None
        } else {
            Some(config.extra_headers.clone())
        };

        let extra_params = if config.extra_params.is_empty() {
            None
        } else {
            Some(
                config
                    .extra_params
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        };

        info!(
            provider_id = %config.id,
            base_url = %config.base_url,
            model = %model,
            max_input_chars = max_input_chars,
            "Created LLM provider"
        );

        Ok(Self {
            id: config.id.clone(),
            name: config.name.clone(),
            client,
            base_url: config.base_url.clone(),
            api_keys: config.api_keys.clone(),
            model,
            max_tokens: config.max_tokens as i32,
            temperature: config.temperature as f64,
            proxy_url: config.proxy_url.clone(),
            timeout: config.timeout,
            extra_headers,
            extra_params,
            max_input_chars,
            health: Arc::new(RwLock::new(ProviderHealth::Healthy)),
            failure_count: Arc::new(AtomicU32::new(0)),
            stats: Arc::new(RwLock::new(ProviderStats::default())),
            weight: config.weight,
            current_key_index: Arc::new(AtomicU32::new(0)),
        })
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    /// Get provider ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get provider name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get provider weight
    pub fn weight(&self) -> u32 {
        self.weight
    }

    /// Get maximum input characters
    pub fn max_input_chars(&self) -> usize {
        self.max_input_chars
    }

    /// Check if provider can handle text of given length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.max_input_chars == 0 || text_len <= self.max_input_chars
    }

    // -------------------------------------------------------------------------
    // Health Management
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // API Key Management
    // -------------------------------------------------------------------------

    /// Get next API key (for rotation)
    fn next_api_key(&self) -> Option<String> {
        if self.api_keys.is_empty() {
            return None;
        }

        let index =
            self.current_key_index.fetch_add(1, Ordering::Relaxed) % self.api_keys.len() as u32;
        self.api_keys.get(index as usize).cloned()
    }

    /// Get current API key
    fn current_api_key(&self) -> String {
        self.next_api_key().unwrap_or_default()
    }

    // -------------------------------------------------------------------------
    // Translation
    // -------------------------------------------------------------------------

    /// Build translation prompt
    fn build_prompt(&self, text: &str, target_lang: &str) -> String {
        format!(
            r#"You are a professional translator. Translate the following text to {}.
Only return the translated text, without any explanations or additional content.

Text to translate:
{}"#,
            target_lang, text
        )
    }

    /// Sanitize error message to remove sensitive info
    fn sanitize_error(&self, message: &str) -> String {
        let mut result = message.to_string();

        // Redact all API keys
        for key in &self.api_keys {
            if !key.is_empty() {
                result = result.replace(key, "***REDACTED***");
            }
        }

        // Redact common API key patterns
        let patterns = [
            Regex::new(r"sk-[a-zA-Z0-9]{20,}").expect("Invalid regex pattern for API key"),
            Regex::new(r"Bearer\s+[a-zA-Z0-9\-._~+/]+=*")
                .expect("Invalid regex pattern for Bearer token"),
        ];

        for pattern in &patterns {
            result = pattern.replace_all(&result, "***REDACTED***").to_string();
        }

        result
    }

    /// Perform translation via HTTP API
    async fn translate_via_api(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let prompt = self.build_prompt(text, target_lang);
        let api_key = self.current_api_key();

        let req_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: Some(self.max_tokens.max(1)),
            temperature: Some(self.temperature.clamp(0.0, 2.0)),
            top_p: None,
            stream: Some(false),
        };

        // Construct URL, handling base_url that may or may not end with /v1
        let base = self.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        };

        debug!(
            provider_id = %self.id,
            url = %url,
            model = %self.model,
            "Sending LLM request"
        );

        let mut request_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", api_key));

        if let Some(ref headers) = self.extra_headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        let response = request_builder
            .json(&req_body)
            .send()
            .await
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| TranslateError::Http(e.to_string()))?;

        if !status.is_success() {
            let sanitized = self.sanitize_error(&response_text);
            error!(
                provider_id = %self.id,
                status = %status,
                body = %sanitized,
                "LLM API error"
            );
            return Err(TranslateError::Translation(format!(
                "LLM API error: {} - {}",
                status, sanitized
            )));
        }

        let llm_resp: ChatCompletionResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                TranslateError::Parse(format!(
                    "Failed to parse LLM response: {} - {}",
                    e, response_text
                ))
            })?;

        if let Some(ref api_err) = llm_resp.error {
            let sanitized = self.sanitize_error(&api_err.message);
            error!(provider_id = %self.id, error = %sanitized, "LLM API error");
            return Err(TranslateError::Translation(format!(
                "LLM API error: {}",
                sanitized
            )));
        }

        let translated_text = llm_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text,
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            ..Default::default()
        })
    }

    /// Translate text with statistics tracking
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = Instant::now();

        // Check capacity
        if !text.is_empty() && !self.can_handle(text.len()) {
            return Err(TranslateError::Translation(format!(
                "Text length {} exceeds provider {} maximum capacity {} characters",
                text.len(),
                self.id,
                self.max_input_chars
            )));
        }

        // Handle empty text
        if text.is_empty() {
            return Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text: text.to_string(),
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                ..Default::default()
            });
        }

        // Perform translation
        let result = self
            .translate_via_api(text, source_lang, target_lang)
            .await;

        let latency = start_time.elapsed();

        // Update statistics
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

        result
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<()> {
        let start_time = Instant::now();

        match self.is_available().await {
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
}

#[async_trait]
impl Translator for LLMProvider {
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let resp = self.translate(text, "", target_lang).await?;
            results.push(resp.translated_text);
        }

        Ok(results)
    }

    async fn translate_single(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let resp = self.translate(text, source_lang, target_lang).await?;
        Ok(resp.translated_text)
    }

    fn name(&self) -> &str {
        "llm-provider"
    }

    async fn is_available(&self) -> bool {
        // Simple availability check - try a minimal translation
        match self.translate_via_api("hello", "", "Chinese").await {
            Ok(_) => true,
            Err(e) => {
                warn!("Provider {} availability check failed: {}", self.id, e);
                false
            }
        }
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        vec!["AUTO"]
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        vec!["EN", "ZH", "JA", "KO", "DE", "FR", "ES", "IT", "PT", "RU"]
    }

    fn max_input_chars(&self) -> usize {
        self.max_input_chars
    }
}
