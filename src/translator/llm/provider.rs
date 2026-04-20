use async_trait::async_trait;
use once_cell::sync::Lazy;
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
use crate::reporter::Reporter;
use crate::translator::common::TranslateResponse;
use crate::translator::Translator;

// ============================================================================
// Static Regex Patterns (compiled once)
// ============================================================================

static API_KEY_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"sk-[a-zA-Z0-9]{20,}").expect("Invalid regex pattern for API key"));

static BEARER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Bearer\s+[a-zA-Z0-9\-._~+/]+=*").expect("Invalid regex pattern for Bearer token")
});

static MARKDOWN_CODE_BLOCK_WRAPPER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^```(?:\w*)\n?([\s\S]*?)\n?```$")
        .expect("Invalid regex pattern for code block wrapper")
});

static MARKDOWN_INLINE_WRAPPER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^`([^`]+)`$").expect("Invalid regex pattern for inline wrapper"));

// ============================================================================
// Token Estimation
// ============================================================================

/// Token estimation configuration for different languages
#[derive(Debug, Clone, Copy)]
pub struct TokenEstimationConfig {
    /// Characters per token for CJK languages (Chinese, Japanese, Korean)
    pub cjk_chars_per_token: f64,
    /// Characters per token for non-CJK languages
    pub non_cjk_chars_per_token: f64,
    /// Ratio of tokens reserved for output (translation may be longer than source)
    pub output_reserve_ratio: f64,
    /// System prompt overhead in tokens
    pub system_prompt_tokens: usize,
}

impl Default for TokenEstimationConfig {
    fn default() -> Self {
        Self {
            // CJK: ~1.5 chars/token (conservative for Chinese)
            cjk_chars_per_token: 1.5,
            // Non-CJK: ~4 chars/token (English, etc.)
            non_cjk_chars_per_token: 4.0,
            // Reserve 40% for output (translation can be 20-30% longer)
            output_reserve_ratio: 0.4,
            // System prompt + user prompt overhead (increased due to separate system message)
            system_prompt_tokens: 150,
        }
    }
}

impl TokenEstimationConfig {
    /// Create a conservative config for mixed-language content
    pub fn conservative() -> Self {
        Self {
            cjk_chars_per_token: 1.5,
            non_cjk_chars_per_token: 3.5,
            output_reserve_ratio: 0.5,
            // System prompt + user prompt overhead (increased due to separate system message)
            system_prompt_tokens: 180,
        }
    }

    /// Estimate tokens for given text
    pub fn estimate_tokens(&self, text: &str) -> usize {
        let cjk_count = text.chars().filter(|c| is_cjk(*c)).count();
        let total_chars = text.chars().count();
        let non_cjk_count = total_chars - cjk_count;

        let cjk_tokens = cjk_count as f64 / self.cjk_chars_per_token;
        let non_cjk_tokens = non_cjk_count as f64 / self.non_cjk_chars_per_token;

        (cjk_tokens + non_cjk_tokens).ceil() as usize + self.system_prompt_tokens
    }

    /// Calculate max input characters for a given token limit
    pub fn calculate_max_chars(&self, max_tokens: usize) -> usize {
        // Available tokens after reserving for output and system prompt
        let available_tokens = (max_tokens as f64 * (1.0 - self.output_reserve_ratio)) as usize;
        let input_tokens = available_tokens.saturating_sub(self.system_prompt_tokens);

        // Use conservative estimate (CJK ratio) for safety
        // This ensures we don't exceed limit even with Chinese text
        (input_tokens as f64 * self.cjk_chars_per_token) as usize
    }
}

/// Check if a character is CJK (Chinese, Japanese, Korean)
fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
        '\u{3040}'..='\u{309F}' | // Hiragana
        '\u{30A0}'..='\u{30FF}' | // Katakana
        '\u{AC00}'..='\u{D7AF}' | // Korean Hangul Syllables
        '\u{FF00}'..='\u{FFEF}'   // Full-width characters
    )
}

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
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    extra_params: Option<serde_json::Map<String, serde_json::Value>>,
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

/// Health check configuration
#[derive(Debug, Clone, Copy)]
pub struct HealthConfig {
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes required to recover
    pub recovery_threshold: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            recovery_threshold: 2,
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
    extra_headers: Option<Vec<(String, String)>>,
    extra_params: Option<serde_json::Map<String, serde_json::Value>>,
    custom_system_prompt: Option<String>,
    custom_user_prompt: Option<String>,

    // Capacity
    max_input_chars: usize,
    token_config: TokenEstimationConfig,

    // State
    health: Arc<RwLock<ProviderHealth>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    stats: Arc<RwLock<ProviderStats>>,
    rate_limit: u32,
    current_key_index: Arc<AtomicU32>,
    health_config: HealthConfig,

    // Reporter for statistics tracking
    reporter: Option<Arc<dyn Reporter>>,
}

impl std::fmt::Debug for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMProvider")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("max_input_chars", &self.max_input_chars)
            .field("rate_limit", &self.rate_limit)
            .field("health_config", &self.health_config)
            .field("token_config", &self.token_config)
            .finish_non_exhaustive()
    }
}

impl LLMProvider {
    /// Create a new LLM provider from configuration
    pub fn new(config: &LLMProviderConfig) -> Result<Self> {
        Self::new_with_health_config(config, HealthConfig::default())
    }

    /// Create a new LLM provider with custom health configuration
    pub fn new_with_health_config(
        config: &LLMProviderConfig,
        health_config: HealthConfig,
    ) -> Result<Self> {
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

        // Use conservative token estimation for translation tasks
        // This ensures we don't exceed token limits even with CJK text
        let token_config = TokenEstimationConfig::conservative();
        let max_input_chars = token_config.calculate_max_chars(config.max_tokens as usize);

        // Convert extra_headers and extra_params
        let extra_headers = if config.extra_headers.is_empty() {
            None
        } else {
            Some(
                config
                    .extra_headers
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        };

        let extra_params = if config.extra_params.is_empty() {
            None
        } else {
            Some(config.extra_params.clone().into_iter().collect())
        };

        info!(
            provider_id = %config.id,
            base_url = %config.base_url,
            model = %model,
            max_tokens = config.max_tokens,
            max_input_chars = max_input_chars,
            failure_threshold = health_config.failure_threshold,
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
            extra_headers,
            extra_params,
            max_input_chars,
            token_config,
            health: Arc::new(RwLock::new(ProviderHealth::Healthy)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            stats: Arc::new(RwLock::new(ProviderStats::default())),
            rate_limit: config.rate_limit,
            current_key_index: Arc::new(AtomicU32::new(0)),
            health_config,
            custom_system_prompt: config.custom_system_prompt.clone(),
            custom_user_prompt: config.custom_user_prompt.clone(),
            reporter: None,
        })
    }

    /// Set reporter for statistics tracking
    pub fn set_reporter(&mut self, reporter: Arc<dyn Reporter>) {
        self.reporter = Some(reporter);
    }

    /// Get reporter if set
    pub fn reporter(&self) -> Option<Arc<dyn Reporter>> {
        self.reporter.clone()
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

    /// Get provider rate limit (used for routing weight)
    pub fn rate_limit(&self) -> u32 {
        self.rate_limit
    }

    /// Get maximum input characters
    pub fn max_input_chars(&self) -> usize {
        self.max_input_chars
    }

    /// Check if provider can handle text of given length
    pub fn can_handle(&self, text_len: usize) -> bool {
        self.max_input_chars == 0 || text_len <= self.max_input_chars
    }

    /// Check if provider can handle specific text with accurate token estimation
    pub fn can_handle_text(&self, text: &str) -> bool {
        if self.max_input_chars == 0 {
            return true;
        }

        // Use precise token estimation
        let estimated_tokens = self.token_config.estimate_tokens(text);
        let max_tokens = self.max_tokens as usize;

        // Must leave room for output (which may be longer than input)
        let available_for_input =
            (max_tokens as f64 * (1.0 - self.token_config.output_reserve_ratio)) as usize;

        estimated_tokens <= available_for_input
    }

    /// Estimate tokens for given text
    pub fn estimate_tokens(&self, text: &str) -> usize {
        self.token_config.estimate_tokens(text)
    }

    /// Get token estimation configuration
    pub fn token_config(&self) -> &TokenEstimationConfig {
        &self.token_config
    }

    // -------------------------------------------------------------------------
    // Health Management (with threshold)
    // -------------------------------------------------------------------------

    /// Check if provider is healthy
    pub async fn is_healthy(&self) -> bool {
        *self.health.read().await == ProviderHealth::Healthy
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let new_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

        // Check if we should recover
        if new_count >= self.health_config.recovery_threshold {
            let mut health = self.health.write().await;
            if *health == ProviderHealth::Unhealthy {
                *health = ProviderHealth::Healthy;
                self.failure_count.store(0, Ordering::Relaxed);
                info!(
                    "Provider {} recovered after {} consecutive successes",
                    self.id, new_count
                );
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) -> bool {
        let new_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        // Check if we should mark as unhealthy
        if new_count >= self.health_config.failure_threshold {
            let mut health = self.health.write().await;
            if *health == ProviderHealth::Healthy {
                *health = ProviderHealth::Unhealthy;
                self.success_count.store(0, Ordering::Relaxed);
                warn!(
                    "Provider {} marked as unhealthy after {} consecutive failures",
                    self.id, new_count
                );
                return true; // Provider became unhealthy
            }
        }
        false
    }

    /// Mark provider as healthy (manual override)
    pub async fn mark_healthy(&self) {
        *self.health.write().await = ProviderHealth::Healthy;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        debug!("Provider {} manually marked as healthy", self.id);
    }

    /// Mark provider as unhealthy (manual override)
    pub async fn mark_unhealthy(&self) {
        *self.health.write().await = ProviderHealth::Unhealthy;
        warn!("Provider {} manually marked as unhealthy", self.id);
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get success count (since last failure)
    pub fn success_count(&self) -> u32 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// Get provider statistics
    pub async fn stats(&self) -> ProviderStats {
        self.stats.read().await.clone()
    }

    /// Get health configuration
    pub fn health_config(&self) -> &HealthConfig {
        &self.health_config
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

    /// Build system prompt for translation
    fn build_system_prompt(&self) -> String {
        if let Some(ref custom_prompt) = self.custom_system_prompt {
            return custom_prompt.clone();
        }

        r#"You are a professional code comment translator. Translate natural language content to the target language.

Rules:
- Return ONLY the translated text
- Preserve code syntax, URLs, and special characters exactly
- Keep existing formatting in the original text
- Do not add explanations or markdown wrappers
- CRITICAL: Keep ALL variable placeholders completely unchanged:
  - Template literals: ${variable}, ${value}, ${name}, etc.
  - Format strings: {variable}, {name}, {0}, {1}, etc.
  - Any text starting with $ followed by { and ending with }
  - Any text starting with { and ending with }
  - These placeholders must remain EXACTLY as they appear
- Do NOT translate, modify, or add the word "placeholder"
- Do NOT translate variable names or code elements

Examples:
Original: 错误：${error}，代码：${code}
Translation: Error: ${error}, code: ${code}

Original: 你好 {name}，欢迎来到 {place}
Translation: Hello {name}, welcome to {place}

Original: 这个函数计算总和
Translation: This function calculates the sum"#.to_string()
    }

    /// Build user prompt for translation
    fn build_user_prompt(&self, text: &str, source_lang: &str, target_lang: &str) -> String {
        if let Some(ref template) = self.custom_user_prompt {
            return template
                .replace("{source_lang}", source_lang)
                .replace("{target_lang}", target_lang)
                .replace("{text}", text);
        }

        let source_instruction = if source_lang == "AUTO" {
            "Auto-detect the source language and translate".to_string()
        } else {
            format!("Translate from {} to {}", source_lang, target_lang)
        };

        format!("{}:\n\n{}", source_instruction, text)
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

        // Redact common API key patterns using pre-compiled regex
        result = API_KEY_PATTERN
            .replace_all(&result, "***REDACTED***")
            .to_string();
        result = BEARER_PATTERN
            .replace_all(&result, "***REDACTED***")
            .to_string();

        result
    }

    /// Clean outer markdown wrapper that LLM may have added
    /// Only removes wrapper if the ENTIRE response is wrapped in markdown
    fn clean_markdown_wrapper(text: &str) -> String {
        let result = text.trim();

        if let Some(captures) = MARKDOWN_CODE_BLOCK_WRAPPER.captures(result) {
            if let Some(inner) = captures.get(1) {
                return inner.as_str().trim().to_string();
            }
        }

        if let Some(captures) = MARKDOWN_INLINE_WRAPPER.captures(result) {
            if let Some(inner) = captures.get(1) {
                return inner.as_str().to_string();
            }
        }

        result.to_string()
    }

    /// Validate translation result quality
    /// Returns Ok(()) if valid, Err if the result appears to be hallucination or low quality
    fn validate_translation(original: &str, translated: &str, provider_id: &str) -> Result<()> {
        if translated.is_empty() {
            warn!(
                provider_id = %provider_id,
                "Translation result is empty"
            );
            return Err(TranslateError::Translation(
                "Translation result is empty".to_string(),
            ));
        }

        let original_len = original.chars().count();
        let translated_len = translated.chars().count();

        let length_ratio = if original_len > 0 {
            translated_len as f64 / original_len as f64
        } else {
            1.0
        };

        if length_ratio > 5.0 {
            warn!(
                provider_id = %provider_id,
                original_len = original_len,
                translated_len = translated_len,
                ratio = length_ratio,
                "Translation result is suspiciously long (possible hallucination)"
            );
            return Err(TranslateError::Translation(format!(
                "Translation result is {}x longer than original (possible hallucination)",
                length_ratio
            )));
        }

        let hallucination_patterns = [
            "translate",
            "translation",
            "here is the translation",
            "the translation is",
            "I will translate",
            "as a translator",
            "in the target language",
        ];

        let translated_lower = translated.to_lowercase();
        for pattern in &hallucination_patterns {
            if translated_lower.contains(pattern) && !original.to_lowercase().contains(pattern) {
                warn!(
                    provider_id = %provider_id,
                    pattern = %pattern,
                    "Translation result contains hallucination pattern"
                );
                return Err(TranslateError::Translation(format!(
                    "Translation result contains hallucination pattern: '{}'",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Perform translation via HTTP API
    async fn translate_via_api(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt(text, source_lang, target_lang);
        let api_key = self.current_api_key();

        let req_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            max_tokens: Some(self.max_tokens.max(1)),
            temperature: Some(self.temperature.clamp(0.0, 2.0)),
            top_p: None,
            stream: Some(false),
            extra_params: self.extra_params.clone(),
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

        let cleaned_text = Self::clean_markdown_wrapper(&translated_text);

        if cleaned_text != translated_text {
            debug!(
                provider_id = %self.id,
                original = %translated_text,
                cleaned = %cleaned_text,
                "Cleaned markdown wrapper from LLM response"
            );
        }

        Self::validate_translation(text, &cleaned_text, &self.id)?;

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text: cleaned_text,
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            ..Default::default()
        })
    }

    /// Translate text with statistics tracking and health management
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = Instant::now();
        let chars = text.len();

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
        let result = self.translate_via_api(text, source_lang, target_lang).await;

        let latency = start_time.elapsed();

        // Update statistics and health status
        self.update_stats_and_health(&result, latency, chars).await;

        result
    }

    /// Update statistics and health based on result
    async fn update_stats_and_health(
        &self,
        result: &Result<TranslateResponse>,
        latency: Duration,
        chars: usize,
    ) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.last_request_time = Some(Instant::now());

        let success = result.is_ok();

        match result {
            Ok(_) => {
                stats.successful_requests += 1;
                // Update rolling average latency
                let prev_avg = stats.average_latency_ms;
                let n = stats.successful_requests as f64;
                stats.average_latency_ms = prev_avg + (latency.as_millis() as f64 - prev_avg) / n;

                // Record success for health tracking
                drop(stats); // Release lock before async call
                self.record_success().await;
            }
            Err(_) => {
                stats.failed_requests += 1;
                drop(stats); // Release lock before async call
                self.record_failure().await;
            }
        }

        // Report to external reporter if available
        if let Some(ref reporter) = self.reporter {
            reporter.report_llm_provider_call(
                &self.id,
                &self.name,
                &self.model,
                latency.as_millis() as u64,
                success,
                chars,
            );
        }
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
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let resp = self.translate(text, source_lang, target_lang).await?;
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

    fn set_reporter(&mut self, reporter: Arc<dyn Reporter>) {
        self.reporter = Some(reporter);
    }

    fn reporter(&self) -> Option<Arc<dyn Reporter>> {
        self.reporter.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.recovery_threshold, 2);
    }

    #[test]
    fn test_sanitize_error_static_regex() {
        // This test verifies that static regex patterns are compiled correctly
        let message = "Error with sk-abc12345678901234567890 and Bearer token123";
        let sanitized = API_KEY_PATTERN.replace_all(message, "***REDACTED***");
        assert!(sanitized.contains("***REDACTED***"));
    }

    #[test]
    fn test_token_estimation_config_default() {
        let config = TokenEstimationConfig::default();
        assert_eq!(config.cjk_chars_per_token, 1.5);
        assert_eq!(config.non_cjk_chars_per_token, 4.0);
        assert_eq!(config.output_reserve_ratio, 0.4);
        assert_eq!(config.system_prompt_tokens, 150);
    }

    #[test]
    fn test_token_estimation_config_conservative() {
        let config = TokenEstimationConfig::conservative();
        assert_eq!(config.cjk_chars_per_token, 1.5);
        assert_eq!(config.non_cjk_chars_per_token, 3.5);
        assert_eq!(config.output_reserve_ratio, 0.5);
        assert_eq!(config.system_prompt_tokens, 180);
    }

    #[test]
    fn test_estimate_tokens_cjk() {
        let config = TokenEstimationConfig::default();
        // Chinese text: "你好世界" (4 CJK chars)
        let chinese = "你好世界";
        let cjk_count = chinese.chars().filter(|c| is_cjk(*c)).count();
        let total_chars = chinese.chars().count();
        let non_cjk_count = total_chars - cjk_count;

        // All 4 chars are CJK
        assert_eq!(cjk_count, 4);
        assert_eq!(non_cjk_count, 0);

        let tokens = config.estimate_tokens(chinese);
        // CJK tokens: 4 / 1.5 = 2.67 -> 3 + 150 system = 153
        assert!(
            (152..=154).contains(&tokens),
            "Expected ~153 tokens, got {}",
            tokens
        );
    }

    #[test]
    fn test_estimate_tokens_english() {
        let config = TokenEstimationConfig::default();
        // English text: "Hello world" (11 chars, non-CJK)
        let english = "Hello world";
        let tokens = config.estimate_tokens(english);
        // 11 chars / 4 chars/token = 2.75 -> 3 + 150 system = 153
        assert!((150..=155).contains(&tokens));
    }

    #[test]
    fn test_estimate_tokens_mixed() {
        let config = TokenEstimationConfig::default();
        // Mixed text: "Hello 世界" (5 non-CJK letters + 1 space + 2 CJK = 8 chars)
        let mixed = "Hello 世界";
        let tokens = config.estimate_tokens(mixed);
        // 2 CJK / 1.5 = 1.33, 6 non-CJK / 4 = 1.5, total ~2.83 -> 3 + 150 = 153
        assert!(
            (152..=154).contains(&tokens),
            "Expected ~153 tokens, got {}",
            tokens
        );
    }

    #[test]
    fn test_calculate_max_chars() {
        let config = TokenEstimationConfig::conservative();
        // With 1000 tokens, 50% reserved for output, 180 for system
        // Available: 1000 * (1.0 - 0.5) - 180 = 320 tokens for input
        // At 1.5 chars/token: 320 * 1.5 = 480 chars
        let max_chars = config.calculate_max_chars(1000);
        assert_eq!(max_chars, 480);
    }

    #[test]
    fn test_is_cjk() {
        assert!(is_cjk('中'));
        assert!(is_cjk('あ')); // Hiragana
        assert!(is_cjk('ア')); // Katakana
        assert!(is_cjk('한')); // Korean
        assert!(!is_cjk('A'));
        assert!(!is_cjk('1'));
        assert!(!is_cjk(' '));
    }

    #[test]
    fn test_build_system_prompt_default() {
        let config = LLMProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_keys: vec!["key".to_string()],
            model: "gpt-4".to_string(),
            custom_system_prompt: None,
            custom_user_prompt: None,
            ..Default::default()
        };

        let provider = LLMProvider::new(&config).expect("Failed to create provider");
        let prompt = provider.build_system_prompt();

        assert!(prompt.contains("translator"));
        assert!(prompt.contains("ONLY"));
    }

    #[test]
    fn test_build_system_prompt_custom() {
        let config = LLMProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_keys: vec!["key".to_string()],
            model: "gpt-4".to_string(),
            custom_system_prompt: Some("Custom system prompt".to_string()),
            custom_user_prompt: None,
            ..Default::default()
        };

        let provider = LLMProvider::new(&config).expect("Failed to create provider");
        let prompt = provider.build_system_prompt();

        assert_eq!(prompt, "Custom system prompt");
    }

    #[test]
    fn test_build_user_prompt_default() {
        let config = LLMProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_keys: vec!["key".to_string()],
            model: "gpt-4".to_string(),
            custom_system_prompt: None,
            custom_user_prompt: None,
            ..Default::default()
        };

        let provider = LLMProvider::new(&config).expect("Failed to create provider");
        let prompt = provider.build_user_prompt("Hello world", "AUTO", "zh");

        assert!(prompt.contains("Auto-detect"));
        assert!(prompt.contains("Hello world"));
    }

    #[test]
    fn test_build_user_prompt_custom() {
        let config = LLMProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_keys: vec!["key".to_string()],
            model: "gpt-4".to_string(),
            custom_system_prompt: None,
            custom_user_prompt: Some(
                "Translate from {source_lang} to {target_lang}: {text}".to_string(),
            ),
            ..Default::default()
        };

        let provider = LLMProvider::new(&config).expect("Failed to create provider");
        let prompt = provider.build_user_prompt("Hello", "en", "zh");

        assert_eq!(prompt, "Translate from en to zh: Hello");
    }

    #[test]
    fn test_build_user_prompt_with_auto() {
        let config = LLMProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://api.example.com".to_string(),
            api_keys: vec!["key".to_string()],
            model: "gpt-4".to_string(),
            custom_system_prompt: None,
            custom_user_prompt: Some(
                "Translate from {source_lang} to {target_lang}: {text}".to_string(),
            ),
            ..Default::default()
        };

        let provider = LLMProvider::new(&config).expect("Failed to create provider");
        let prompt = provider.build_user_prompt("Hello", "AUTO", "zh");

        assert_eq!(prompt, "Translate from AUTO to zh: Hello");
    }
}
