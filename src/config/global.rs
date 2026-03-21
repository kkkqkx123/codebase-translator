use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::translator::ProviderType;

use super::env::{expand_env_vars, replace_env_vars_in_map, replace_env_vars_in_nested_map};
use tracing::debug;

/// Global configuration (user-level settings)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default translation provider
    #[serde(default)]
    pub provider: ProviderType,
    /// Enabled providers list
    #[serde(default)]
    pub enabled_providers: Vec<String>,
    /// DeepLX configuration
    #[serde(default)]
    pub deeplx: DeepLXConfig,
    /// LLM configuration
    #[serde(default)]
    pub llm: LLMGlobalConfig,
    /// Tencent Cloud configuration
    #[serde(default)]
    pub tencent: TencentConfig,
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    /// Global limits configuration
    #[serde(default)]
    pub limits: LimitConfig,
}

impl GlobalConfig {
    /// Validate the global configuration
    pub fn validate(&mut self) -> Result<(), String> {
        debug!(
            provider = %self.provider,
            enabled_providers = ?self.enabled_providers,
            "Validating global configuration"
        );

        let valid_providers = ["deeplx", "llm", "tencent"];

        let providers = self.get_enabled_providers();
        if providers.is_empty() {
            return Err("At least one provider must be enabled".to_string());
        }

        for provider in &providers {
            if !valid_providers.contains(&provider.as_str()) {
                return Err(format!("Invalid provider: {}", provider));
            }
        }

        let default_provider = self.provider.to_string();
        if !valid_providers.contains(&default_provider.as_str()) {
            return Err(format!("Invalid default provider: {}", default_provider));
        }

        for provider in &providers {
            match provider.as_str() {
                "llm" => {
                    self.filter_invalid_llm_providers();

                    if self.llm.providers.is_empty() {
                        return Err("LLM providers configuration is required when provider 'llm' is enabled (all providers were filtered out due to invalid configuration)".to_string());
                    }

                    for p in &self.llm.providers {
                        if p.base_url.is_empty() {
                            return Err(format!("LLM provider {}: base_url is required", p.id));
                        }
                        if p.api_keys.is_empty() {
                            return Err(format!("LLM provider {}: api_keys is required", p.id));
                        }
                        if p.model.is_empty() {
                            return Err(format!("LLM provider {}: model is required", p.id));
                        }
                        if p.max_tokens == 0 {
                            return Err(format!(
                                "LLM provider {}: max_tokens must be positive",
                                p.id
                            ));
                        }
                        if p.rate_limit == 0 {
                            return Err(format!(
                                "LLM provider {}: rate_limit must be positive",
                                p.id
                            ));
                        }
                    }
                }
                "deeplx" => {
                    if self.deeplx.rate_limit == 0 {
                        return Err("deeplx: rate_limit must be positive".to_string());
                    }
                }
                "tencent" => {
                    if self.tencent.secret_id.is_none()
                        || self
                            .tencent
                            .secret_id
                            .as_ref()
                            .map_or(true, |s| {
                                s.is_empty() || s.starts_with("${")
                            })
                    {
                        return Err("tencent: secret_id is required".to_string());
                    }
                    if self.tencent.secret_key.is_none()
                        || self
                            .tencent
                            .secret_key
                            .as_ref()
                            .map_or(true, |s| {
                                s.is_empty() || s.starts_with("${")
                            })
                    {
                        return Err("tencent: secret_key is required".to_string());
                    }
                    if self.tencent.rate_limit == 0 {
                        return Err("tencent: rate_limit must be positive".to_string());
                    }
                }
                _ => {}
            }
        }

        // Validate logging configuration
        crate::logger::validate_config(&self.logging)
            .map_err(|e| format!("Logging configuration error: {}", e))?;

        debug!("Global configuration validated successfully");
        Ok(())
    }

    /// Get enabled providers list
    pub fn get_enabled_providers(&self) -> Vec<String> {
        if !self.enabled_providers.is_empty() {
            self.enabled_providers.clone()
        } else {
            vec![self.provider.to_string()]
        }
    }

    /// Filter invalid LLM providers
    ///
    /// This method modifies `self.llm.providers` in place, removing invalid providers.
    /// A provider is considered invalid if:
    /// - It has no valid API keys
    /// - Model name is empty or invalid
    pub fn filter_invalid_llm_providers(&mut self) {
        debug!("Filtering invalid LLM providers");
        let mut valid_providers = Vec::new();

        for mut provider in self.llm.providers.drain(..) {
            // If model is empty but model_list has values, use the first model
            if provider.model.is_empty() && !provider.model_list.is_empty() {
                provider.model = provider.model_list[0].clone();
            }

            let mut valid_api_keys = Vec::new();
            for key in &provider.api_keys {
                if Self::is_valid_api_key(key) {
                    valid_api_keys.push(key.clone());
                }
            }

            if valid_api_keys.is_empty() {
                continue;
            }

            // Check if model name is valid
            if provider.model.is_empty() || provider.model.starts_with("${") {
                continue;
            }

            provider.api_keys = valid_api_keys;
            valid_providers.push(provider);
        }

        debug!(
            total_providers = self.llm.providers.len(),
            valid_providers = valid_providers.len(),
            "LLM providers filtered"
        );
        self.llm.providers = valid_providers;
    }

    /// Check if API key is valid
    fn is_valid_api_key(key: &str) -> bool {
        if key.is_empty() {
            return false;
        }

        if key.starts_with("${") && key.ends_with('}') {
            return false;
        }

        let invalid_patterns = [
            "xxx",
            "your-api-key",
            "your_api_key",
            "api-key-here",
            "placeholder",
            "test",
            "example",
            "null",
            "undefined",
            "none",
            "empty",
            "default",
        ];

        let lower_key = key.to_lowercase();
        !invalid_patterns.contains(&lower_key.as_str())
    }

    /// Apply environment variables to configuration
    pub fn apply_env_vars(&mut self) {
        debug!("Applying environment variables to configuration");
        if let Ok(provider) = std::env::var("TRANSLATOR_PROVIDER") {
            if let Ok(provider_type) = provider.parse::<ProviderType>() {
                debug!(
                    env_var = "TRANSLATOR_PROVIDER",
                    value = %provider,
                    "Setting provider from environment variable"
                );
                self.provider = provider_type;
            }
        }

        if let Ok(api_url) = std::env::var("DEEPLX_API_URL") {
            debug!(
                env_var = "DEEPLX_API_URL",
                "Setting DeepLX API URL from environment variable"
            );
            self.deeplx.api_url = api_url;
        }
        if let Ok(api_key) = std::env::var("DEEPLX_API_KEY") {
            debug!(
                env_var = "DEEPLX_API_KEY",
                "Setting DeepLX API key from environment variable"
            );
            self.deeplx.api_key = Some(api_key);
        }

        if let Ok(secret_id) = std::env::var("TENCENT_SECRET_ID") {
            debug!(
                env_var = "TENCENT_SECRET_ID",
                "Setting Tencent secret ID from environment variable"
            );
            self.tencent.secret_id = Some(secret_id);
        }
        if let Ok(secret_key) = std::env::var("TENCENT_SECRET_KEY") {
            debug!(
                env_var = "TENCENT_SECRET_KEY",
                "Setting Tencent secret key from environment variable"
            );
            self.tencent.secret_key = Some(secret_key);
        }

        // Logging configuration environment variables
        if let Ok(log_level) = std::env::var("TRANSLATOR_LOG_LEVEL") {
            debug!(
                env_var = "TRANSLATOR_LOG_LEVEL",
                value = %log_level,
                "Setting log level from environment variable"
            );
            self.logging.level = log_level;
        }
        if let Ok(log_output) = std::env::var("TRANSLATOR_LOG_OUTPUT") {
            debug!(
                env_var = "TRANSLATOR_LOG_OUTPUT",
                value = %log_output,
                "Setting log output from environment variable"
            );
            self.logging.output = log_output;
        }
        if let Ok(log_format) = std::env::var("TRANSLATOR_LOG_FORMAT") {
            debug!(
                env_var = "TRANSLATOR_LOG_FORMAT",
                value = %log_format,
                "Setting log format from environment variable"
            );
            self.logging.format = log_format;
        }
        if let Ok(log_file) = std::env::var("TRANSLATOR_LOG_FILE") {
            debug!(
                env_var = "TRANSLATOR_LOG_FILE",
                "Setting log file from environment variable"
            );
            self.logging.file = Some(log_file);
        }

        // Apply LLM provider environment variables
        // Format: TRANSLATOR_LLM_<PROVIDER_ID>_<KEY>
        // Example: TRANSLATOR_LLM_SILICON_API_KEY
        for provider in &mut self.llm.providers {
            let provider_id_upper = provider.id.to_uppercase();
            
            // API keys: TRANSLATOR_LLM_<PROVIDER_ID>_API_KEY
            let api_key_env = format!("TRANSLATOR_LLM_{}_API_KEY", provider_id_upper);
            if let Ok(api_key) = std::env::var(&api_key_env) {
                debug!(
                    env_var = %api_key_env,
                    provider_id = %provider.id,
                    "Setting LLM provider API key from environment variable"
                );
                provider.api_keys = vec![api_key];
            }
            
            // Base URL: TRANSLATOR_LLM_<PROVIDER_ID>_BASE_URL
            let base_url_env = format!("TRANSLATOR_LLM_{}_BASE_URL", provider_id_upper);
            if let Ok(base_url) = std::env::var(&base_url_env) {
                debug!(
                    env_var = %base_url_env,
                    provider_id = %provider.id,
                    "Setting LLM provider base URL from environment variable"
                );
                provider.base_url = base_url;
            }
            
            // Model: TRANSLATOR_LLM_<PROVIDER_ID>_MODEL
            let model_env = format!("TRANSLATOR_LLM_{}_MODEL", provider_id_upper);
            if let Ok(model) = std::env::var(&model_env) {
                debug!(
                    env_var = %model_env,
                    provider_id = %provider.id,
                    "Setting LLM provider model from environment variable"
                );
                provider.model = model;
            }
        }
    }

    /// Expand environment variables in configuration
    pub fn expand_env_vars(&mut self) {
        debug!("Expanding environment variables in configuration");
        self.deeplx.api_url = expand_env_vars(&self.deeplx.api_url);
        if let Some(ref mut api_key) = self.deeplx.api_key {
            *api_key = expand_env_vars(api_key);
        }
        if let Some(ref mut proxy_url) = self.deeplx.proxy_url {
            *proxy_url = expand_env_vars(proxy_url);
        }

        if let Some(ref mut secret_id) = self.tencent.secret_id {
            *secret_id = expand_env_vars(secret_id);
        }
        if let Some(ref mut secret_key) = self.tencent.secret_key {
            *secret_key = expand_env_vars(secret_key);
        }
        self.tencent.region = expand_env_vars(&self.tencent.region);
        if let Some(ref mut proxy_url) = self.tencent.proxy_url {
            *proxy_url = expand_env_vars(proxy_url);
        }

        for provider in &mut self.llm.providers {
            provider.base_url = expand_env_vars(&provider.base_url);
            provider.model = expand_env_vars(&provider.model);
            // If model is empty but model_list has values, use the first model
            if provider.model.is_empty() && !provider.model_list.is_empty() {
                provider.model = provider.model_list[0].clone();
            }
            // Expand env vars in model_list
            for model in &mut provider.model_list {
                *model = expand_env_vars(model);
            }
            for key in &mut provider.api_keys {
                *key = expand_env_vars(key);
            }
            if let Some(ref mut proxy_url) = provider.proxy_url {
                *proxy_url = expand_env_vars(proxy_url);
            }
            replace_env_vars_in_map(&mut provider.extra_headers);
            replace_env_vars_in_nested_map(&mut provider.extra_params);
        }

        debug!("Environment variables expanded successfully");
    }

    /// Merge another configuration into this one
    pub fn merge(&mut self, other: GlobalConfig) {
        debug!("Merging global configuration");
        if other.provider != ProviderType::default() {
            self.provider = other.provider;
        }
        if !other.enabled_providers.is_empty() {
            self.enabled_providers = other.enabled_providers;
        }
        if !other.deeplx.api_url.is_empty() {
            self.deeplx.api_url = other.deeplx.api_url;
        }
        if other.deeplx.api_key.is_some() {
            self.deeplx.api_key = other.deeplx.api_key;
        }
        if other.deeplx.proxy_url.is_some() {
            self.deeplx.proxy_url = other.deeplx.proxy_url;
        }
        if other.deeplx.rate_limit > 0 {
            self.deeplx.rate_limit = other.deeplx.rate_limit;
        }
        if other.deeplx.max_retries > 0 {
            self.deeplx.max_retries = other.deeplx.max_retries;
        }

        if !other.llm.providers.is_empty() {
            self.llm.providers = other.llm.providers;
        }

        if other.tencent.secret_id.is_some() {
            self.tencent.secret_id = other.tencent.secret_id;
        }
        if other.tencent.secret_key.is_some() {
            self.tencent.secret_key = other.tencent.secret_key;
        }
        if !other.tencent.region.is_empty() {
            self.tencent.region = other.tencent.region;
        }
        if other.tencent.proxy_url.is_some() {
            self.tencent.proxy_url = other.tencent.proxy_url;
        }
        if other.tencent.timeout > 0 {
            self.tencent.timeout = other.tencent.timeout;
        }
        if other.tencent.rate_limit > 0 {
            self.tencent.rate_limit = other.tencent.rate_limit;
        }
        if other.tencent.max_retries > 0 {
            self.tencent.max_retries = other.tencent.max_retries;
        }

        if other.limits.rate_limit > 0 {
            self.limits.rate_limit = other.limits.rate_limit;
        }
        if other.limits.max_char_count > 0 {
            self.limits.max_char_count = other.limits.max_char_count;
        }
        if other.limits.split_max_chars > 0 {
            self.limits.split_max_chars = other.limits.split_max_chars;
        }

        if !other.logging.level.is_empty() {
            self.logging.level = other.logging.level;
        }
        if !other.logging.format.is_empty() {
            self.logging.format = other.logging.format;
        }
        if !other.logging.output.is_empty() {
            self.logging.output = other.logging.output;
        }
        if other.logging.file.is_some() {
            self.logging.file = other.logging.file;
        }

        debug!("Global configuration merged successfully");
    }
}

/// DeepLX configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLXConfig {
    /// DeepLX API URL
    #[serde(default = "default_deeplx_url")]
    pub api_url: String,
    /// API key (optional for some instances)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Proxy URL
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Rate limit (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    /// Max retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

impl Default for DeepLXConfig {
    fn default() -> Self {
        Self {
            api_url: default_deeplx_url(),
            api_key: None,
            proxy_url: None,
            rate_limit: default_rate_limit(),
            max_retries: default_max_retries(),
        }
    }
}

fn default_deeplx_url() -> String {
    "http://localhost:1188".to_string()
}

/// LLM global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMGlobalConfig {
    /// Health check configuration
    #[serde(default)]
    pub health_check: HealthCheckConfig,
    /// LLM providers
    #[serde(default)]
    pub providers: Vec<LLMProviderConfig>,
}

impl Default for LLMGlobalConfig {
    fn default() -> Self {
        Self {
            health_check: HealthCheckConfig::default(),
            providers: Vec::new(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health check
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub interval: u64,
    /// Check timeout in seconds
    #[serde(default = "default_health_check_timeout")]
    pub timeout: u64,
    /// Failure threshold
    #[serde(default = "default_health_check_failure_threshold")]
    pub failure_threshold: u32,
    /// Recovery interval in seconds
    #[serde(default = "default_health_check_recovery_interval")]
    pub recovery_interval: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: default_health_check_interval(),
            timeout: default_health_check_timeout(),
            failure_threshold: default_health_check_failure_threshold(),
            recovery_interval: default_health_check_recovery_interval(),
        }
    }
}

fn default_health_check_interval() -> u64 {
    30
}

fn default_health_check_timeout() -> u64 {
    5
}

fn default_health_check_failure_threshold() -> u32 {
    3
}

fn default_health_check_recovery_interval() -> u64 {
    60
}

/// LLM provider configuration
///
/// Each provider represents a single model/endpoint combination.
/// For models with different context lengths, configure them as separate providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderConfig {
    /// Provider ID (unique identifier)
    pub id: String,
    /// Provider name (human readable)
    pub name: String,
    /// Weight for capacity-based routing (higher = preferred for larger texts)
    #[serde(default)]
    pub weight: u32,
    /// Base URL for API
    pub base_url: String,
    /// API keys (for rotation)
    pub api_keys: Vec<String>,
    /// Model name (single model, preferred)
    #[serde(default)]
    pub model: String,
    /// Model list (for multi-model rotation, uses first model if model is empty)
    #[serde(default, alias = "models")]
    pub model_list: Vec<String>,
    /// Max tokens per request (determines capacity)
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Temperature (0.0 - 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Proxy URL
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Rate limit (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    /// Extra headers
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
    /// Extra parameters
    #[serde(default)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.3
}

/// Tencent Cloud configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TencentConfig {
    /// Secret ID
    pub secret_id: Option<String>,
    /// Secret key
    pub secret_key: Option<String>,
    /// Region
    #[serde(default = "default_tencent_region")]
    pub region: String,
    /// Project ID
    #[serde(default)]
    pub project_id: u32,
    /// Endpoint
    #[serde(default = "default_tencent_endpoint")]
    pub endpoint: String,
    /// Proxy URL
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Rate limit
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    /// Max retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Untranslated text patterns
    #[serde(default)]
    pub untranslated_text: Vec<String>,
    /// Term repository ID list
    #[serde(default)]
    pub term_repo_id_list: Vec<String>,
    /// Sentence repository ID list
    #[serde(default)]
    pub sent_repo_id_list: Vec<String>,
}

impl Default for TencentConfig {
    fn default() -> Self {
        Self {
            secret_id: None,
            secret_key: None,
            region: default_tencent_region(),
            project_id: 0,
            endpoint: default_tencent_endpoint(),
            proxy_url: None,
            timeout: default_timeout(),
            rate_limit: 5,
            max_retries: default_max_retries(),
            untranslated_text: Vec::new(),
            term_repo_id_list: Vec::new(),
            sent_repo_id_list: Vec::new(),
        }
    }
}

fn default_tencent_region() -> String {
    "ap-guangzhou".to_string()
}

fn default_tencent_endpoint() -> String {
    "tmt.tencentcloudapi.com".to_string()
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Output: stdout, stderr, file
    #[serde(default = "default_log_output")]
    pub output: String,
    /// Log file path (when output is "file")
    #[serde(default)]
    pub file: Option<String>,
    /// Log format: pretty, json, compact
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            output: default_log_output(),
            file: None,
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_output() -> String {
    "stdout".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_rate_limit() -> u32 {
    10
}

fn default_max_retries() -> u32 {
    3
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

/// Global limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitConfig {
    /// Rate limit (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    /// Maximum character count per request
    #[serde(default)]
    pub max_char_count: u32,
    /// Maximum characters per split chunk
    #[serde(default = "default_split_max_chars")]
    pub split_max_chars: u32,
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self {
            rate_limit: default_rate_limit(),
            max_char_count: 0,
            split_max_chars: default_split_max_chars(),
        }
    }
}

fn default_split_max_chars() -> u32 {
    1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_global_config() {
        let config = GlobalConfig::default();
        assert_eq!(config.provider.to_string(), "deeplx");
        assert_eq!(config.deeplx.api_url, "http://localhost:1188");
        assert_eq!(config.deeplx.rate_limit, 10);
        assert_eq!(config.tencent.region, "ap-guangzhou");
        assert_eq!(config.tencent.rate_limit, 5);
    }

    #[test]
    fn test_validate_empty_providers() {
        let mut config = GlobalConfig {
            enabled_providers: Vec::new(),
            provider: ProviderType::LLM,
            ..Default::default()
        };
        config.llm.providers = Vec::new();

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("LLM providers configuration is required"));
    }

    #[test]
    fn test_validate_invalid_provider() {
        let mut config = GlobalConfig {
            enabled_providers: vec!["invalid".to_string()],
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid provider"));
    }

    #[test]
    fn test_filter_invalid_llm_providers() {
        let mut config = GlobalConfig::default();
        config.llm.providers = vec![
            LLMProviderConfig {
                id: "provider1".to_string(),
                name: "Provider 1".to_string(),
                weight: 1,
                base_url: "https://api.example.com".to_string(),
                api_keys: vec!["valid-key".to_string(), "xxx".to_string()],
                model: "model1".to_string(),
                model_list: vec![],
                max_tokens: 4096,
                temperature: 0.7,
                proxy_url: None,
                timeout: 30,
                rate_limit: 10,
                extra_headers: std::collections::HashMap::new(),
                extra_params: std::collections::HashMap::new(),
            },
            LLMProviderConfig {
                id: "provider2".to_string(),
                name: "Provider 2".to_string(),
                weight: 1,
                base_url: "https://api.example2.com".to_string(),
                api_keys: vec!["xxx".to_string()],
                model: "".to_string(), // Invalid: empty model name
                model_list: vec![],
                max_tokens: 4096,
                temperature: 0.7,
                proxy_url: None,
                timeout: 30,
                rate_limit: 10,
                extra_headers: std::collections::HashMap::new(),
                extra_params: std::collections::HashMap::new(),
            },
        ];

        config.filter_invalid_llm_providers();
        assert_eq!(config.llm.providers.len(), 1);
        assert_eq!(config.llm.providers[0].id, "provider1");
        assert_eq!(config.llm.providers[0].api_keys.len(), 1);
        assert_eq!(config.llm.providers[0].api_keys[0], "valid-key");
    }

    #[test]
    fn test_expand_env_vars_in_config() {
        std::env::set_var("DEEPLX_URL", "https://deeplx.example.com");
        std::env::set_var("DEEPLX_KEY", "test-key");
        std::env::set_var("TENCENT_ID", "tencent-id");
        std::env::set_var("TENCENT_KEY", "tencent-key");

        let mut config = GlobalConfig::default();
        config.deeplx.api_url = "${DEEPLX_URL}".to_string();
        config.deeplx.api_key = Some("${DEEPLX_KEY}".to_string());
        config.tencent.secret_id = Some("${TENCENT_ID}".to_string());
        config.tencent.secret_key = Some("${TENCENT_KEY}".to_string());

        config.expand_env_vars();

        assert_eq!(config.deeplx.api_url, "https://deeplx.example.com");
        assert_eq!(config.deeplx.api_key.unwrap(), "test-key");
        assert_eq!(config.tencent.secret_id.unwrap(), "tencent-id");
        assert_eq!(config.tencent.secret_key.unwrap(), "tencent-key");

        std::env::remove_var("DEEPLX_URL");
        std::env::remove_var("DEEPLX_KEY");
        std::env::remove_var("TENCENT_ID");
        std::env::remove_var("TENCENT_KEY");
    }

    #[test]
    fn test_apply_env_vars() {
        std::env::set_var("TRANSLATOR_PROVIDER", "llm");
        std::env::set_var("DEEPLX_API_URL", "https://deeplx.example.com");
        std::env::set_var("DEEPLX_API_KEY", "test-key");
        std::env::set_var("TENCENT_SECRET_ID", "tencent-id");
        std::env::set_var("TENCENT_SECRET_KEY", "tencent-key");

        let mut config = GlobalConfig::default();
        config.apply_env_vars();

        assert_eq!(config.provider.to_string(), "llm");
        assert_eq!(config.deeplx.api_url, "https://deeplx.example.com");
        assert_eq!(config.deeplx.api_key.unwrap(), "test-key");
        assert_eq!(config.tencent.secret_id.unwrap(), "tencent-id");
        assert_eq!(config.tencent.secret_key.unwrap(), "tencent-key");

        std::env::remove_var("TRANSLATOR_PROVIDER");
        std::env::remove_var("DEEPLX_API_URL");
        std::env::remove_var("DEEPLX_API_KEY");
        std::env::remove_var("TENCENT_SECRET_ID");
        std::env::remove_var("TENCENT_SECRET_KEY");
    }

    #[test]
    fn test_merge_config() {
        let mut base = GlobalConfig::default();
        base.deeplx.api_url = "http://localhost:1188".to_string();
        base.deeplx.rate_limit = 10;

        let other = GlobalConfig {
            provider: ProviderType::LLM,
            enabled_providers: vec!["llm".to_string()],
            deeplx: DeepLXConfig {
                api_url: "https://new-url.com".to_string(),
                api_key: Some("new-key".to_string()),
                proxy_url: None,
                rate_limit: 20,
                max_retries: 5,
            },
            llm: LLMGlobalConfig::default(),
            tencent: TencentConfig::default(),
            logging: LoggingConfig::default(),
            limits: LimitConfig::default(),
        };

        base.merge(other);

        assert_eq!(base.provider.to_string(), "llm");
        assert_eq!(base.deeplx.api_url, "https://new-url.com");
        assert_eq!(base.deeplx.api_key.unwrap(), "new-key");
        assert_eq!(base.deeplx.rate_limit, 20);
        assert_eq!(base.deeplx.max_retries, 5);
    }

    #[test]
    fn test_get_enabled_providers() {
        let config = GlobalConfig {
            enabled_providers: vec!["deeplx".to_string(), "llm".to_string()],
            ..Default::default()
        };
        let providers = config.get_enabled_providers();
        assert_eq!(providers, vec!["deeplx", "llm"]);

        let config = GlobalConfig {
            enabled_providers: Vec::new(),
            ..Default::default()
        };
        let providers = config.get_enabled_providers();
        assert_eq!(providers, vec!["deeplx"]);
    }

    #[test]
    fn test_llm_provider_with_multiple_models() {
        let mut config = GlobalConfig {
            enabled_providers: vec!["llm".to_string()],
            provider: ProviderType::LLM,
            ..Default::default()
        };

        config.llm.providers = vec![LLMProviderConfig {
            id: "silicon".to_string(),
            name: "Siliconflow".to_string(),
            weight: 50,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_keys: vec!["test-api-key".to_string()],
            model: "".to_string(), // Empty model, should use first from model_list
            model_list: vec![
                "tencent/Hunyuan-MT-7B".to_string(),
                "THUDM/GLM-4-9B-0414".to_string(),
                "Qwen/Qwen2.5-7B-Instruct".to_string(),
            ],
            max_tokens: 4096,
            temperature: 0.3,
            proxy_url: None,
            timeout: 20,
            rate_limit: 40,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        }];

        // Validate should filter and set model from model_list
        let result = config.validate();
        assert!(result.is_ok(), "Validation failed: {:?}", result.err());
        assert_eq!(config.llm.providers.len(), 1);
        assert_eq!(config.llm.providers[0].model, "tencent/Hunyuan-MT-7B");
        assert_eq!(config.llm.providers[0].model_list.len(), 3);
    }

    #[test]
    fn test_filter_llm_providers_with_empty_api_key() {
        let mut config = GlobalConfig {
            enabled_providers: vec!["llm".to_string()],
            provider: ProviderType::LLM,
            ..Default::default()
        };

        config.llm.providers = vec![LLMProviderConfig {
            id: "silicon".to_string(),
            name: "Siliconflow".to_string(),
            weight: 50,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_keys: vec!["${SILICON_API_KEY}".to_string()], // Unresolved placeholder
            model: "tencent/Hunyuan-MT-7B".to_string(),
            model_list: vec![],
            max_tokens: 4096,
            temperature: 0.3,
            proxy_url: None,
            timeout: 20,
            rate_limit: 40,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        }];

        // Validate should fail because provider has no valid API keys
        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("LLM providers configuration is required"));
    }

    #[test]
    fn test_apply_env_vars_to_llm_provider() {
        std::env::set_var("TRANSLATOR_LLM_SILICON_API_KEY", "env-api-key-67890");
        std::env::set_var("TRANSLATOR_LLM_SILICON_BASE_URL", "https://env.example.com");
        std::env::set_var("TRANSLATOR_LLM_SILICON_MODEL", "env-model");

        let mut config = GlobalConfig {
            enabled_providers: vec!["llm".to_string()],
            provider: ProviderType::LLM,
            ..Default::default()
        };

        config.llm.providers = vec![LLMProviderConfig {
            id: "silicon".to_string(),
            name: "Siliconflow".to_string(),
            weight: 50,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            api_keys: vec!["original-key".to_string()],
            model: "original-model".to_string(),
            model_list: vec![],
            max_tokens: 4096,
            temperature: 0.3,
            proxy_url: None,
            timeout: 20,
            rate_limit: 40,
            extra_headers: std::collections::HashMap::new(),
            extra_params: std::collections::HashMap::new(),
        }];

        config.apply_env_vars();

        assert_eq!(config.llm.providers[0].api_keys, vec!["env-api-key-67890"]);
        assert_eq!(config.llm.providers[0].base_url, "https://env.example.com");
        assert_eq!(config.llm.providers[0].model, "env-model");

        std::env::remove_var("TRANSLATOR_LLM_SILICON_API_KEY");
        std::env::remove_var("TRANSLATOR_LLM_SILICON_BASE_URL");
        std::env::remove_var("TRANSLATOR_LLM_SILICON_MODEL");
    }

    #[test]
    fn test_validate_tencent_with_empty_credentials() {
        let mut config = GlobalConfig {
            enabled_providers: vec!["tencent".to_string()],
            provider: ProviderType::Tencent,
            ..Default::default()
        };

        // Test with empty secret_id
        config.tencent.secret_id = Some("".to_string());
        config.tencent.secret_key = Some("valid-key".to_string());

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("tencent: secret_id is required"));

        // Test with empty secret_key
        config.tencent.secret_id = Some("valid-id".to_string());
        config.tencent.secret_key = Some("".to_string());

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("tencent: secret_key is required"));

        // Test with unresolved placeholder
        config.tencent.secret_id = Some("${TENCENT_SECRET_ID}".to_string());
        config.tencent.secret_key = Some("${TENCENT_SECRET_KEY}".to_string());

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("tencent: secret_id is required"));
    }
}
