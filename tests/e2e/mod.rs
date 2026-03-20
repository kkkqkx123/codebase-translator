//! E2E Integration Tests
//!
//! These tests verify the integration between components and
//! test against real translation services (when configured).

#![allow(static_mut_refs)]

use std::path::PathBuf;
use std::sync::Once;

use codebase_translate::config::{ConfigLoader, GlobalConfig};

static INIT: Once = Once::new();
static mut GLOBAL_CONFIG: Option<GlobalConfig> = None;

/// Get the project root directory
fn get_project_root() -> PathBuf {
    PathBuf::from(file!())
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get tests directory")
        .parent()
        .expect("Failed to get project root")
        .to_path_buf()
}

/// Initialize test configuration
///
/// If no global config file exists, returns a default config.
/// Tests will skip actual API calls when credentials are not configured.
pub fn init_test_config() -> &'static GlobalConfig {
    unsafe {
        INIT.call_once(|| {
            let project_root = get_project_root();
            std::env::set_current_dir(&project_root).ok();

            let loader = ConfigLoader::new();
            let config = loader.load_global().unwrap_or_else(|_| {
                // No global config file found, use default config
                // Tests will skip when credentials are not configured
                GlobalConfig::default()
            });
            GLOBAL_CONFIG = Some(config);
        });
        GLOBAL_CONFIG
            .as_ref()
            .expect("Global config should be initialized")
    }
}

/// Check if a configuration value is actually configured (not empty and not a placeholder)
pub fn is_configured(value: &str) -> bool {
    !value.is_empty() && !value.starts_with("${")
}

pub mod deeplx_test;
pub mod llm_test;
pub mod tencent_test;
