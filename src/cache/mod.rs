//! Caching system

pub mod binary;
pub mod util;

pub use crate::core::models::{CacheConfig, CacheEntry, CacheEntryInfo, CacheMode, CacheStats};

use tracing::info;

/// Factory for creating cache instances
pub struct CacheFactory;

impl CacheFactory {
    /// Create cache instance
    pub fn create(
        cache_config: &CacheConfig,
        project_path: &str,
    ) -> Result<binary::BinaryCache, crate::core::error::TranslateError> {
        info!(
            cache_type = %cache_config.mode,
            cache_dir = %project_path,
            "Creating cache instance"
        );
        let cache = binary::BinaryCache::new(cache_config.clone(), project_path)?;
        tracing::debug!("Cache instance created successfully");
        Ok(cache)
    }
}
