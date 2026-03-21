use std::fs;
use tempfile::TempDir;

use codebase_translate::config::loader::ConfigLoader;

#[test]
fn test_all_providers_enabled_with_env_vars() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["deeplx", "llm", "tencent"]

[deeplx]
api_url = "https://api.deeplx.org"
api_key = "${DEEPLX_API_KEY}"
proxy_url = ""
rate_limit = 5
max_retries = 3

[llm]
[[llm.providers]]
id = "silicon"
name = "Siliconflow"
models = ["tencent/Hunyuan-MT-7B"]
max_tokens = 4096
temperature = 0.3
weight = 50
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["${SILICON_API_KEY}"]
proxy_url = ""
timeout = 20
rate_limit = 40

[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
region = "ap-guangzhou"
project_id = 0
proxy_url = ""
timeout = 30
rate_limit = 5
max_retries = 3

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Set environment variables
    std::env::set_var("DEEPLX_API_KEY", "test-deeplx-key");
    std::env::set_var("SILICON_API_KEY", "test-silicon-key");
    std::env::set_var("TENCENT_SECRET_ID", "test-secret-id");
    std::env::set_var("TENCENT_SECRET_KEY", "test-secret-key");

    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Clean up environment variables
    std::env::remove_var("DEEPLX_API_KEY");
    std::env::remove_var("SILICON_API_KEY");
    std::env::remove_var("TENCENT_SECRET_ID");
    std::env::remove_var("TENCENT_SECRET_KEY");

    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Verify all providers are enabled
    assert_eq!(config.enabled_providers, vec!["deeplx", "llm", "tencent"]);

    // Verify DeepLX configuration
    assert_eq!(config.deeplx.api_url, "https://api.deeplx.org");
    assert_eq!(config.deeplx.api_key, Some("test-deeplx-key".to_string()));

    // Verify LLM configuration
    assert_eq!(config.llm.providers.len(), 1);
    assert_eq!(config.llm.providers[0].id, "silicon");
    assert_eq!(config.llm.providers[0].api_keys, vec!["test-silicon-key"]);
    assert_eq!(config.llm.providers[0].model, "tencent/Hunyuan-MT-7B");

    // Verify Tencent configuration
    assert_eq!(config.tencent.secret_id, Some("test-secret-id".to_string()));
    assert_eq!(config.tencent.secret_key, Some("test-secret-key".to_string()));
    assert_eq!(config.tencent.region, "ap-guangzhou");
}

#[test]
fn test_llm_provider_with_multiple_models() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["llm"]

[llm]
[[llm.providers]]
id = "silicon"
name = "Siliconflow"
models = [
    "tencent/Hunyuan-MT-7B",
    "THUDM/GLM-4-9B-0414",
    "Qwen/Qwen2.5-7B-Instruct"
]
max_tokens = 4096
temperature = 0.3
weight = 50
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["${SILICON_API_KEY}"]
proxy_url = ""
timeout = 20
rate_limit = 40

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Set environment variable
    std::env::set_var("SILICON_API_KEY", "test-silicon-key");

    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Clean up environment variables
    std::env::remove_var("SILICON_API_KEY");

    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Verify LLM provider has multiple models
    assert_eq!(config.llm.providers.len(), 1);
    assert_eq!(config.llm.providers[0].id, "silicon");
    assert_eq!(config.llm.providers[0].model_list.len(), 3);
    assert_eq!(config.llm.providers[0].model_list[0], "tencent/Hunyuan-MT-7B");
    assert_eq!(config.llm.providers[0].model_list[1], "THUDM/GLM-4-9B-0414");
    assert_eq!(config.llm.providers[0].model_list[2], "Qwen/Qwen2.5-7B-Instruct");
    // Model should be set to the first model in the list
    assert_eq!(config.llm.providers[0].model, "tencent/Hunyuan-MT-7B");
}

#[test]
fn test_tencent_provider_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["tencent"]

[tencent]
secret_id = "${TENCENT_SECRET_ID}"
secret_key = "${TENCENT_SECRET_KEY}"
region = "ap-guangzhou"
project_id = 0
proxy_url = ""
timeout = 30
rate_limit = 5
max_retries = 3

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Set environment variables
    std::env::set_var("TENCENT_SECRET_ID", "test-secret-id");
    std::env::set_var("TENCENT_SECRET_KEY", "test-secret-key");

    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Clean up environment variables
    std::env::remove_var("TENCENT_SECRET_ID");
    std::env::remove_var("TENCENT_SECRET_KEY");

    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Verify Tencent configuration
    assert_eq!(config.tencent.secret_id, Some("test-secret-id".to_string()));
    assert_eq!(config.tencent.secret_key, Some("test-secret-key".to_string()));
    assert_eq!(config.tencent.region, "ap-guangzhou");
    assert_eq!(config.tencent.project_id, 0);
    assert_eq!(config.tencent.timeout, 30);
    assert_eq!(config.tencent.rate_limit, 5);
    assert_eq!(config.tencent.max_retries, 3);
}

#[test]
fn test_llm_provider_with_empty_api_key_filtered() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["llm"]

[llm]
[[llm.providers]]
id = "silicon"
name = "Siliconflow"
models = ["tencent/Hunyuan-MT-7B"]
max_tokens = 4096
temperature = 0.3
weight = 50
base_url = "https://api.siliconflow.cn/v1"
api_keys = ["${SILICON_API_KEY}"]
proxy_url = ""
timeout = 20
rate_limit = 40

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Do NOT set environment variable - API key should be empty and provider should be filtered out
    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // This should fail because LLM provider is enabled but no valid providers are configured
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("LLM providers configuration is required"));
}

#[test]
fn test_tencent_provider_without_credentials_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["tencent"]

[tencent]
secret_id = ""
secret_key = ""
region = "ap-guangzhou"
project_id = 0
proxy_url = ""
timeout = 30
rate_limit = 5
max_retries = 3

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Do NOT set environment variables - credentials should be empty
    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // This should fail because Tencent provider requires credentials
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("tencent: secret_id is required"));
}
