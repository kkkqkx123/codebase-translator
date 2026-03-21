use std::fs;
use tempfile::TempDir;

use codebase_translate::config::loader::ConfigLoader;
use codebase_translate::config::global::GlobalConfig;

#[test]
fn test_llm_config_with_env_vars() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["deeplx", "llm"]

[deeplx]
api_url = "https://api.deeplx.org"
api_key = ""
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

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Set environment variable
    std::env::set_var("SILICON_API_KEY", "test-api-key-12345");
    std::env::set_var("TRANSLATOR_CONFIG_HOME", temp_dir.path());

    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Clean up environment variables
    std::env::remove_var("SILICON_API_KEY");
    std::env::remove_var("TRANSLATOR_CONFIG_HOME");

    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Verify that LLM provider has the API key from environment variable
    assert_eq!(config.llm.providers.len(), 1);
    assert_eq!(config.llm.providers[0].id, "silicon");
    assert_eq!(config.llm.providers[0].api_keys, vec!["test-api-key-12345"]);
}

#[test]
fn test_llm_config_with_env_vars_via_apply_env_vars() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("translator.toml");

    let config_content = r#"
enabled_providers = ["llm"]

[llm]
[[llm.providers]]
id = "test_provider"
name = "Test Provider"
models = ["test-model"]
max_tokens = 4096
temperature = 0.3
weight = 50
base_url = "https://api.test.com/v1"
api_keys = ["${TEST_API_KEY}"]
proxy_url = ""
timeout = 20
rate_limit = 40

[logging]
level = "info"
format = "pretty"
output = "stdout"
"#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    // Set environment variable for apply_env_vars
    std::env::set_var("TRANSLATOR_LLM_TEST_PROVIDER_API_KEY", "env-api-key-67890");
    std::env::set_var("TRANSLATOR_CONFIG_HOME", temp_dir.path());

    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Clean up environment variables
    std::env::remove_var("TRANSLATOR_LLM_TEST_PROVIDER_API_KEY");
    std::env::remove_var("TRANSLATOR_CONFIG_HOME");

    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Verify that LLM provider has the API key from environment variable via apply_env_vars
    assert_eq!(config.llm.providers.len(), 1);
    assert_eq!(config.llm.providers[0].id, "test_provider");
    assert_eq!(config.llm.providers[0].api_keys, vec!["env-api-key-67890"]);
}

#[test]
fn test_llm_config_validation_with_valid_env_vars() {
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

    // Set environment variable
    std::env::set_var("TRANSLATOR_LLM_SILICON_API_KEY", "valid-api-key");
    std::env::set_var("TRANSLATOR_CONFIG_HOME", temp_dir.path());

    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Clean up environment variables
    std::env::remove_var("TRANSLATOR_LLM_SILICON_API_KEY");
    std::env::remove_var("TRANSLATOR_CONFIG_HOME");

    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());
    let config = result.unwrap();

    // Verify that LLM provider is not filtered out
    assert_eq!(config.llm.providers.len(), 1);
    assert_eq!(config.llm.providers[0].id, "silicon");
    assert_eq!(config.llm.providers[0].api_keys, vec!["valid-api-key"]);
}
