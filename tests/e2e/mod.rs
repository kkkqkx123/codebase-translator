//! E2E Integration Tests
//!
//! These tests verify the integration between components and
//! test against real translation services (when configured).

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
pub fn init_test_config() -> &'static GlobalConfig {
    unsafe {
        INIT.call_once(|| {
            let project_root = get_project_root();
            std::env::set_current_dir(&project_root).ok();

            let loader = ConfigLoader::new();
            let config = loader.load_global().expect("Failed to load global config");
            GLOBAL_CONFIG = Some(config);
        });
        GLOBAL_CONFIG
            .as_ref()
            .expect("Global config should be initialized")
    }
}

/// Check if running in CI environment
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Skip test if no API credentials are configured
pub fn skip_if_no_credentials(configured: bool) {
    if !configured && is_ci() {
        panic!("Test requires API credentials but none configured in CI");
    }
}

pub mod deeplx_test;
pub mod llm_test;
pub mod tencent_test;
