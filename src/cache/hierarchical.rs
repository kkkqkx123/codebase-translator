//! Directory cache implementation
//!
//! This module provides a simple directory-based cache system that stores
//! cache entries only in the directory where translation is executed.
//! No hierarchical searching or cross-directory cache sharing.

use crate::cache::binary::BinaryCache;
use crate::core::error::Result;
use crate::core::models::{CacheConfig, CacheEntry};
use std::path::Path;

/// Directory cache that stores entries only in the execution directory
pub struct DirectoryCache {
    cache: BinaryCache,
}

impl DirectoryCache {
    /// Create a new directory cache
    pub fn new(config: CacheConfig, project_dir: &Path) -> Result<Self> {
        let cache = BinaryCache::new(config, project_dir)?;
        Ok(Self { cache })
    }

    /// Get cache entry for a file hash
    pub fn get(&self, file_hash: &str, config_hash: &str) -> Result<Option<CacheEntry>> {
        self.cache.get(file_hash, config_hash)
    }

    /// Set cache entry (writes to current directory cache)
    pub fn set(&self, entry: &CacheEntry) -> Result<()> {
        self.cache.set(entry)
    }

    /// Get the project fingerprint from cache
    pub fn project_fingerprint(&self) -> &str {
        self.cache.project_fingerprint()
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<crate::core::models::CacheStats> {
        self.cache.stats()
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<()> {
        self.cache.clear()
    }

    /// Close the cache
    pub fn close(&self) -> Result<()> {
        self.cache.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::CacheMode;
    use tempfile::tempdir;

    #[test]
    fn test_directory_cache_creation() {
        let temp_dir = tempdir().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: CacheMode::Local,
            directory: ".translator".to_string(),
            format: "binary".to_string(),
        };

        let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();
        assert!(!cache.project_fingerprint().is_empty());
    }

    #[test]
    fn test_directory_cache_set_and_get() {
        let temp_dir = tempdir().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: CacheMode::Local,
            directory: ".translator".to_string(),
            format: "binary".to_string(),
        };

        let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();
        let fingerprint = cache.project_fingerprint().to_string();

        let file_hash = "a".repeat(64);
        let config_hash = "test_config_hash";
        let mut entry = CacheEntry::new(
            &file_hash,
            "/path/to/test.txt",
            123456i64,
            "local",
            fingerprint,
            config_hash,
        );
        entry.mark_as_translated();

        cache.set(&entry).unwrap();

        let retrieved = cache.get(&file_hash, config_hash).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert!(retrieved.is_translated);
        assert_eq!(retrieved.file_hash, file_hash);
    }
}
