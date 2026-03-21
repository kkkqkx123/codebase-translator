use std::fs;
use tempfile::TempDir;

use codebase_translate::config::loader::ConfigLoader;

#[test]
fn test_tencent_empty_credentials_behavior() {
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

    // Load config without environment variables
    let loader = ConfigLoader::new().with_global_config(&config_path);
    let result = loader.load_global();

    // Check of result
    match result {
        Ok(config) => {
            let secret_id_is_none = config.tencent.secret_id.is_none();
            let secret_id_is_empty = config.tencent.secret_id.as_ref().map_or(true, |s| s.is_empty());
            let secret_key_is_none = config.tencent.secret_key.is_none();
            let secret_key_is_empty = config.tencent.secret_key.as_ref().map_or(true, |s| s.is_empty());
            
            panic!(
                "Config loaded successfully\n\
                 secret_id: {:?}\n\
                 secret_key: {:?}\n\
                 secret_id is None: {}\n\
                 secret_id is empty: {}\n\
                 secret_key is None: {}\n\
                 secret_key is empty: {}",
                config.tencent.secret_id,
                config.tencent.secret_key,
                secret_id_is_none,
                secret_id_is_empty,
                secret_key_is_none,
                secret_key_is_empty
            );
        }
        Err(e) => {
            panic!("Config loading failed: {:?}", e);
        }
    }
}
