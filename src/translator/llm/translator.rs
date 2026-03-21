use async_trait::async_trait;
use regex::Regex;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::translator::common::{LLMConfig, TranslateResponse};
use crate::translator::Translator;

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

/// LLM translator
pub struct LLMTranslator {
    client: Client,
    config: LLMConfig,
    max_input_chars: usize,
}

impl std::fmt::Debug for LLMTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMTranslator")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl LLMTranslator {
    /// Create a new LLM translator
    pub fn new(config: LLMConfig) -> Result<Self> {
        if config.base_url.is_empty() {
            return Err(TranslateError::Config(
                "base_url is required for LLM translator".to_string(),
            ));
        }
        if config.model.is_empty() {
            return Err(TranslateError::Config(
                "model is required for LLM translator".to_string(),
            ));
        }

        let timeout = Duration::from_secs(config.timeout.clamp(1, 600));

        let client_builder = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10);

        let client_builder = if let Some(ref proxy_url) = config.proxy_url {
            if !proxy_url.is_empty() {
                match Proxy::all(proxy_url) {
                    Ok(proxy) => client_builder.proxy(proxy),
                    Err(e) => {
                        warn!("Failed to set proxy: {}", e);
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

        let max_input_chars = config.max_input_chars();

        info!(
            "LLM translator created with base_url: {}, model: {}, max_input_chars: {}",
            config.base_url, config.model, max_input_chars
        );

        Ok(Self {
            client,
            config,
            max_input_chars,
        })
    }

    /// Get maximum input characters allowed for this translator
    pub fn max_input_chars(&self) -> usize {
        self.max_input_chars
    }

    /// Check if text exceeds the maximum input length
    pub fn is_text_too_long(&self, text: &str) -> bool {
        text.len() > self.max_input_chars
    }

    /// Sanitize error message to remove sensitive info
    fn sanitize_error(&self, message: &str) -> String {
        let mut result = message.to_string();

        if !self.config.api_key.is_empty() {
            result = result.replace(&self.config.api_key, "***REDACTED***");
        }

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

    /// Translate a single text
    async fn translate_single_internal(
        &self,
        text: &str,
        _source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        if text.is_empty() {
            return Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text: text.to_string(),
                source_lang: _source_lang.to_string(),
                target_lang: target_lang.to_string(),
                ..Default::default()
            });
        }

        // Check if text exceeds maximum input length
        if text.len() > self.max_input_chars {
            return Err(TranslateError::Translation(format!(
                "Text length {} exceeds maximum allowed {} characters for LLM translator",
                text.len(),
                self.max_input_chars
            )));
        }

        let prompt = self.build_prompt(text, target_lang);

        let req_body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: Some(self.config.max_tokens.max(1)),
            temperature: Some(self.config.temperature.clamp(0.0, 2.0)),
            top_p: self.config.top_p,
            stream: Some(false),
        };

        // Construct URL, handling base_url that may or may not end with /v1
        let base = self.config.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        };

        debug!("LLM request: url={}, model={}", url, self.config.model);

        let mut request_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key));

        if let Some(ref headers) = self.config.extra_headers {
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
            error!("LLM API error: status={}, body={}", status, sanitized);
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
            error!("LLM API error: {}", sanitized);
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
            source_lang: _source_lang.to_string(),
            target_lang: target_lang.to_string(),
            ..Default::default()
        })
    }
}

#[async_trait]
impl Translator for LLMTranslator {
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let resp = self
                .translate_single_internal(text, "", target_lang)
                .await?;
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
        let resp = self
            .translate_single_internal(text, source_lang, target_lang)
            .await?;
        Ok(resp.translated_text)
    }

    fn name(&self) -> &str {
        "llm"
    }

    async fn is_available(&self) -> bool {
        match self.translate_single_internal("hello", "", "Chinese").await {
            Ok(_) => true,
            Err(e) => {
                warn!("LLM availability check failed: {}", e);
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
