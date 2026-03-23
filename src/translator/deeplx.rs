//! DeepLX translation service

use async_trait::async_trait;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::reporter::Reporter;
use crate::translator::common::{DeepLXConfig, TranslateResponse};
use crate::translator::Translator;

const DEFAULT_API_URL: &str = "https://api.deeplx.org";
const DEFAULT_TIMEOUT: u64 = 30;

/// DeepLX request
#[derive(Debug, Serialize)]
struct DeepLXRequest {
    text: String,
    #[serde(rename = "source_lang", skip_serializing_if = "String::is_empty")]
    source_lang: String,
    #[serde(rename = "target_lang")]
    target_lang: String,
}

/// DeepLX response
#[derive(Debug, Deserialize)]
struct DeepLXResponse {
    data: String,
}

/// DeepLX translator
pub struct DeepLXTranslator {
    client: Client,
    config: DeepLXConfig,
    api_url: String,
    reporter: Option<Arc<dyn Reporter>>,
}

impl std::fmt::Debug for DeepLXTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeepLXTranslator")
            .field("config", &self.config)
            .field("api_url", &self.api_url)
            .finish_non_exhaustive()
    }
}

impl DeepLXTranslator {
    /// Create a new DeepLX translator
    pub fn new(config: DeepLXConfig) -> Result<Self> {
        let api_url = if config.api_url.is_empty() {
            DEFAULT_API_URL.to_string()
        } else {
            config.api_url.clone()
        };

        let timeout = Duration::from_secs(DEFAULT_TIMEOUT);

        let client_builder = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10);

        // Set proxy if provided
        let client_builder = if let Some(ref proxy_url) = config.proxy_url {
            if !proxy_url.is_empty() && !proxy_url.contains("${") {
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

        info!(
            api_url = %api_url,
            "DeepLX translator created"
        );

        Ok(Self {
            client,
            config,
            api_url,
            reporter: None,
        })
    }

    /// Translate a single text with statistics tracking
    async fn translate_single_internal(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        let start_time = Instant::now();
        let chars = text.len();

        if text.is_empty() {
            return Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text: text.to_string(),
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                ..Default::default()
            });
        }

        let req_body = DeepLXRequest {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        };

        let api_url = if let Some(ref api_key) = self.config.api_key {
            format!("{}/{}/translate", self.api_url, api_key)
        } else {
            format!("{}/translate", self.api_url)
        };

        debug!(
            url = %api_url,
            source_lang = source_lang,
            target_lang = target_lang,
            "Sending DeepLX translation request"
        );

        let result = async {
            let response = self
                .client
                .post(&api_url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
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
                error!(
                    status = %status,
                    response_body = %response_text,
                    "DeepLX API error"
                );
                return Err(TranslateError::Translation(format!(
                    "DeepLX API error: {} - {}",
                    status, response_text
                )));
            }

            let deeplx_resp: DeepLXResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    error!(
                        error = %e,
                        response_body = %response_text,
                        "Failed to parse DeepLX response"
                    );
                    TranslateError::Parse(format!(
                        "Failed to parse DeepLX response: {} - {}",
                        e, response_text
                    ))
                })?;

            Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text: deeplx_resp.data,
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                ..Default::default()
            })
        }
        .await;

        // Report statistics
        let latency_ms = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();
        if let Some(ref reporter) = self.reporter {
            reporter.report_translator_call("deeplx", latency_ms, success, chars);
        }

        result
    }
}

#[async_trait]
impl Translator for DeepLXTranslator {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let translated = self
                .translate_single(text, source_lang, target_lang)
                .await?;
            results.push(translated);
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
        "deeplx"
    }

    async fn is_available(&self) -> bool {
        match self.translate_single_internal("hello", "", "zh").await {
            Ok(_) => true,
            Err(e) => {
                warn!("DeepLX availability check failed: {}", e);
                false
            }
        }
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        vec![
            "AUTO", "EN", "ZH", "JA", "KO", "DE", "FR", "ES", "IT", "PT", "RU",
        ]
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        vec!["EN", "ZH", "JA", "KO", "DE", "FR", "ES", "IT", "PT", "RU"]
    }

    fn max_input_chars(&self) -> usize {
        5000
    }

    fn set_reporter(&mut self, reporter: Arc<dyn Reporter>) {
        self.reporter = Some(reporter);
    }

    fn reporter(&self) -> Option<Arc<dyn Reporter>> {
        self.reporter.clone()
    }
}

/// Get default limit policy for DeepLX
pub fn default_limit_policy() -> crate::translator::common::LimitPolicy {
    crate::translator::common::LimitPolicy {
        rate_limit: 10,
        max_char_count: 5000,
        split_max_chars: 4000,
    }
}
