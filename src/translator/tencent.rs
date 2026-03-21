//! Tencent Cloud translation service

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

use crate::core::error::{Result, TranslateError};
use crate::translator::common::{TencentConfig, TranslateResponse};
use crate::translator::Translator;

const API_HOST: &str = "tmt.tencentcloudapi.com";
const API_URL: &str = "https://tmt.tencentcloudapi.com";
const VERSION: &str = "2018-03-21";
const ACTION: &str = "TextTranslate";
const SERVICE: &str = "tmt";
const ALGORITHM: &str = "TC3-HMAC-SHA256";

/// Tencent translate request
#[derive(Debug, Serialize)]
struct TencentTranslateRequest {
    #[serde(rename = "SourceText")]
    source_text: String,
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Target")]
    target: String,
    #[serde(rename = "ProjectId")]
    project_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "UntranslatedText")]
    untranslated_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "TermRepoIDList")]
    term_repo_id_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "SentRepoIDList")]
    sent_repo_id_list: Option<Vec<String>>,
}

/// Tencent translate response
#[derive(Debug, Deserialize)]
struct TencentTranslateResponse {
    #[serde(rename = "Response")]
    response: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    #[serde(rename = "TargetText")]
    target_text: String,
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Target")]
    target: String,
    #[serde(default)]
    #[serde(rename = "Error")]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: String,
}

/// Tencent translator
pub struct TencentTranslator {
    client: Client,
    config: TencentConfig,
}

impl std::fmt::Debug for TencentTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TencentTranslator")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl TencentTranslator {
    /// Create a new Tencent translator
    pub fn new(config: TencentConfig) -> Result<Self> {
        if config.secret_id.is_empty() {
            return Err(TranslateError::Config(
                "secret_id is required for Tencent translator".to_string(),
            ));
        }
        if config.secret_key.is_empty() {
            return Err(TranslateError::Config(
                "secret_key is required for Tencent translator".to_string(),
            ));
        }

        let timeout = Duration::from_secs(config.timeout.clamp(1, 300));

        let client_builder = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10);

        // Set proxy if provided
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

        info!("Tencent translator created with region: {}", config.region);

        Ok(Self { client, config })
    }

    /// Generate signature for Tencent Cloud API
    fn sign(&self, timestamp: i64, payload: &str) -> Result<(String, String)> {
        let date = Self::format_date(timestamp);
        let credential_scope = format!("{}/{}/tc3_request", date, SERVICE);

        // Build canonical request
        let canonical_request = format!(
            "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\n\ncontent-type;host;x-tc-action\n{}",
            API_HOST,
            ACTION.to_lowercase(),
            Self::sha256_hex(payload)
        );

        // Build string to sign
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM,
            timestamp,
            credential_scope,
            Self::sha256_hex(&canonical_request)
        );

        // Calculate signature
        let signature = self.calculate_signature(&string_to_sign, &date)?;

        // Build authorization
        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders=content-type;host;x-tc-action, Signature={}",
            ALGORITHM, self.config.secret_id, credential_scope, signature
        );

        Ok((authorization, date))
    }

    /// Calculate HMAC-SHA256 signature
    fn calculate_signature(&self, string_to_sign: &str, date: &str) -> Result<String> {
        let secret_date =
            Self::hmac_sha256(date, format!("TC3{}", self.config.secret_key).as_bytes());
        let secret_service = Self::hmac_sha256(SERVICE, &secret_date);
        let secret_signing = Self::hmac_sha256("tc3_request", &secret_service);
        let signature = Self::hmac_sha256(string_to_sign, &secret_signing);
        Ok(hex::encode(signature))
    }

    /// HMAC-SHA256
    fn hmac_sha256(data: &str, key: &[u8]) -> Vec<u8> {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    /// SHA256 hex
    fn sha256_hex(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Format date from timestamp
    fn format_date(timestamp: i64) -> String {
        let secs = timestamp as u64;
        let duration = Duration::from_secs(secs);
        let datetime = UNIX_EPOCH + duration;
        let datetime: chrono::DateTime<chrono::Utc> = datetime.into();
        datetime.format("%Y-%m-%d").to_string()
    }

    /// Translate a single text
    async fn translate_single_internal(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslateResponse> {
        if text.is_empty() {
            return Ok(TranslateResponse {
                original_text: text.to_string(),
                translated_text: text.to_string(),
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                ..Default::default()
            });
        }

        if text.len() >= 6000 {
            return Err(TranslateError::Translation(
                "Text length exceeds 6000 characters limit".to_string(),
            ));
        }

        let req_body = TencentTranslateRequest {
            source_text: text.to_string(),
            source: source_lang.to_string(),
            target: target_lang.to_string(),
            project_id: self.config.project_id,
            untranslated_text: if self.config.untranslated_text.is_empty() {
                None
            } else {
                Some(self.config.untranslated_text.join(","))
            },
            term_repo_id_list: if self.config.term_repo_id_list.is_empty() {
                None
            } else {
                Some(self.config.term_repo_id_list.clone())
            },
            sent_repo_id_list: if self.config.sent_repo_id_list.is_empty() {
                None
            } else {
                Some(self.config.sent_repo_id_list.clone())
            },
        };

        let payload = serde_json::to_string(&req_body)
            .map_err(|e| TranslateError::Parse(format!("Failed to serialize request: {}", e)))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let (authorization, date) = self.sign(timestamp, &payload)?;

        debug!(
            "Tencent request: timestamp={}, action={}",
            timestamp, ACTION
        );

        let response = self
            .client
            .post(API_URL)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", API_HOST)
            .header("X-TC-Action", ACTION)
            .header("X-TC-Version", VERSION)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Region", &self.config.region)
            .header("Authorization", authorization)
            .header("X-TC-Date", date)
            .body(payload)
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
                "Tencent API error: status={}, body={}",
                status, response_text
            );
            return Err(TranslateError::Translation(format!(
                "Tencent API error: {} - {}",
                status, response_text
            )));
        }

        let tencent_resp: TencentTranslateResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                TranslateError::Parse(format!(
                    "Failed to parse Tencent response: {} - {}",
                    e, response_text
                ))
            })?;

        // Check for API error in response body
        if let Some(ref api_err) = tencent_resp.response.error {
            error!(
                "Tencent API error: code={}, message={}",
                api_err.code, api_err.message
            );
            return Err(TranslateError::Translation(format!(
                "Tencent API error: {} - {}",
                api_err.code, api_err.message
            )));
        }

        Ok(TranslateResponse {
            original_text: text.to_string(),
            translated_text: tencent_resp.response.target_text,
            source_lang: tencent_resp.response.source,
            target_lang: tencent_resp.response.target,
            ..Default::default()
        })
    }
}

#[async_trait]
impl Translator for TencentTranslator {
    async fn translate(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let translated = self.translate_single(text, "auto", target_lang).await?;
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
        "tencent"
    }

    async fn is_available(&self) -> bool {
        match self.translate_single_internal("hello", "auto", "zh").await {
            Ok(_) => true,
            Err(e) => {
                warn!("Tencent availability check failed: {}", e);
                false
            }
        }
    }

    fn supported_source_langs(&self) -> Vec<&str> {
        vec![
            "auto", "zh", "en", "ja", "ko", "de", "fr", "es", "it", "pt", "ru",
        ]
    }

    fn supported_target_langs(&self) -> Vec<&str> {
        vec!["zh", "en", "ja", "ko", "de", "fr", "es", "it", "pt", "ru"]
    }

    fn max_input_chars(&self) -> usize {
        6000
    }
}

/// Get default limit policy for Tencent
pub fn default_limit_policy() -> crate::translator::common::LimitPolicy {
    crate::translator::common::LimitPolicy {
        rate_limit: 20,
        max_char_count: 6000,
        split_max_chars: 5000,
    }
}
