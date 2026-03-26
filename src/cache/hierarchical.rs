//! Hierarchical cache implementation
//!
//! This module provides a hierarchical cache system that searches for cache files
//! in parent directories, allowing subdirectories to have their own caches that
//! can be reused when translating from higher-level directories.

use crate::cache::binary::BinaryCache;
use crate::core::error::Result;
use crate::core::models::{CacheConfig, CacheEntry};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Hierarchical cache that searches for cache files in parent directories
pub struct HierarchicalCache {
    root_cache: BinaryCache,
    cache_map: HashMap<String, BinaryCache>,
}

impl HierarchicalCache {
    /// Create a new hierarchical cache with the root cache
    pub fn new(config: CacheConfig, project_dir: &Path) -> Result<Self> {
        let root_cache = BinaryCache::new(config, project_dir)?;
        Ok(Self {
            root_cache,
            cache_map: HashMap::new(),
        })
    }

    /// Get cache entry for a file, searching from file's directory up to root
    pub fn get(
        &self,
        file_path: &Path,
        file_hash: &str,
        config_hash: &str,
    ) -> Result<Option<CacheEntry>> {
        // First try root cache
        if let Some(entry) = self.root_cache.get(file_hash, config_hash)? {
            return Ok(Some(entry));
        }

        // Try caches from parent directories
        // For subdirectory caches, we need to bypass fingerprint check
        // since each cache has its own fingerprint
        let file_dir = file_path.parent().unwrap_or(file_path);
        for ancestor in file_dir.ancestors() {
            let cache_key = ancestor.to_string_lossy().to_string();
            if let Some(cache) = self.cache_map.get(&cache_key) {
                // Get raw entry without fingerprint validation
                if let Some(entry) = cache.get_raw(file_hash)? {
                    // Check config hash separately
                    if entry.is_config_valid(config_hash) {
                        info!(
                            file = %file_path.display(),
                            cache_dir = %ancestor.display(),
                            "Cache hit from hierarchical cache"
                        );
                        return Ok(Some(entry));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Load cache from a specific directory
    pub fn load_cache_from_dir(&mut self, cache_dir: &Path) -> Result<()> {
        if let Some(cache) = BinaryCache::try_load_from_dir(cache_dir)? {
            // Use the parent directory of cache_dir as the key
            // cache_dir is like /path/to/.translator, we want /path/to
            let cache_key = cache_dir
                .parent()
                .unwrap_or(cache_dir)
                .to_string_lossy()
                .to_string();
            self.cache_map.insert(cache_key, cache);
            info!(cache_dir = %cache_dir.display(), "Loaded hierarchical cache");
        }
        Ok(())
    }

    /// Set cache entry (always writes to root cache)
    pub fn set(&self, entry: &CacheEntry) -> Result<()> {
        self.root_cache.set(entry)
    }

    /// Get the project fingerprint from root cache
    pub fn project_fingerprint(&self) -> &str {
        self.root_cache.project_fingerprint()
    }

    /// Get cache statistics from root cache
    pub fn stats(&self) -> Result<crate::core::models::CacheStats> {
        self.root_cache.stats()
    }

    /// Clear all cache entries from root cache
    pub fn clear(&self) -> Result<()> {
        self.root_cache.clear()
    }

    /// Close the root cache
    pub fn close(&self) -> Result<()> {
        self.root_cache.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::CacheMode;
    use tempfile::tempdir;

    #[test]
    fn test_hierarchical_cache_creation() {
        let temp_dir = tempdir().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: CacheMode::Local,
            directory: ".translator".to_string(),
            format: "binary".to_string(),
        };

        let cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
        assert!(!cache.project_fingerprint().is_empty());
    }

    #[test]
    fn test_hierarchical_cache_load_nonexistent_dir() {
        let temp_dir = tempdir().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: CacheMode::Local,
            directory: ".translator".to_string(),
            format: "binary".to_string(),
        };

        let mut cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        // Should not fail, just not load anything
        cache.load_cache_from_dir(&nonexistent_dir).unwrap();
    }
}
