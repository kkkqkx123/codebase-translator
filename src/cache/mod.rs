//! Caching system

pub mod binary;
pub mod hierarchical;
pub mod util;

pub use crate::core::models::{CacheConfig, CacheEntry, CacheEntryInfo, CacheMode, CacheStats};
pub use hierarchical::DirectoryCache;

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

    /// Create directory cache instance
    pub fn create_directory(
        cache_config: &CacheConfig,
        project_path: &str,
    ) -> Result<DirectoryCache, crate::core::error::TranslateError> {
        info!(
            cache_type = %cache_config.mode,
            cache_dir = %project_path,
            "Creating directory cache instance"
        );
        let cache = DirectoryCache::new(cache_config.clone(), std::path::Path::new(project_path))?;
        tracing::debug!("Directory cache instance created successfully");
        Ok(cache)
    }
}
