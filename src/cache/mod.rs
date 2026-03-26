//! Caching system

pub mod binary;
pub mod hierarchical;
pub mod util;

pub use crate::core::models::{CacheConfig, CacheEntry, CacheEntryInfo, CacheMode, CacheStats};
pub use hierarchical::HierarchicalCache;

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

    /// Create hierarchical cache instance
    pub fn create_hierarchical(
        cache_config: &CacheConfig,
        project_path: &str,
    ) -> Result<HierarchicalCache, crate::core::error::TranslateError> {
        info!(
            cache_type = %cache_config.mode,
            cache_dir = %project_path,
            "Creating hierarchical cache instance"
        );
        let cache =
            HierarchicalCache::new(cache_config.clone(), std::path::Path::new(project_path))?;
        tracing::debug!("Hierarchical cache instance created successfully");
        Ok(cache)
    }
}
