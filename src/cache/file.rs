//! File-based cache implementation using JSON
//!
//! Multi-file JSON cache where each cache entry is stored as a separate JSON file.
//! Suitable for debugging and scenarios requiring high readability.

use crate::cache::r#trait::Cache;
use crate::cache::util;
use crate::core::error::{Result, TranslateError};
use crate::core::models::{CacheConfig, CacheEntry, CacheEntryInfo, CacheStats};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// File-based cache implementation
pub struct FileCache {
    config: CacheConfig,
    project_dir: PathBuf,
    project_fingerprint: String,
}

impl FileCache {
    /// Create a new file cache instance
    pub fn new(config: CacheConfig, project_dir: impl AsRef<Path>) -> Result<Self> {
        let project_dir = project_dir.as_ref().to_path_buf();

        // Generate project fingerprint
        let project_fingerprint = util::generate_project_fingerprint(&project_dir)?;

        Ok(Self {
            config,
            project_dir,
            project_fingerprint,
        })
    }

    /// Get the project fingerprint for this cache
    pub fn project_fingerprint(&self) -> &str {
        &self.project_fingerprint
    }

    /// Get current cache directory
    fn get_cache_dir(&self) -> PathBuf {
        util::resolve_cache_dir(&self.config.mode, &self.config.directory, &self.project_dir)
    }

    /// Get cache file path for a given hash
    fn get_cache_path(&self, file_hash: &str) -> PathBuf {
        self.get_cache_dir().join(format!("{}.json", file_hash))
    }
}

impl Cache for FileCache {
    fn get(&self, file_hash: &str) -> Result<Option<CacheEntry>> {
        if !self.config.enabled {
            return Ok(None);
        }

        let cache_path = self.get_cache_path(file_hash);

        let data = match std::fs::read_to_string(&cache_path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(TranslateError::Cache(format!(
                    "Failed to read cache file: {}",
                    e
                )))
            }
        };

        let entry: CacheEntry = serde_json::from_str(&data)
            .map_err(|e| TranslateError::Cache(format!("JSON parse error: {}", e)))?;

        if entry.project_fingerprint != self.project_fingerprint {
            return Ok(None);
        }

        Ok(Some(entry))
    }

    fn set(&self, entry: &CacheEntry) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_dir = self.get_cache_dir();
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| TranslateError::Cache(format!("Failed to create cache dir: {}", e)))?;

        let mut entry_to_save = entry.clone();
        entry_to_save.cache_mode = self.config.mode.to_string();
        entry_to_save.project_fingerprint = self.project_fingerprint.clone();

        let data = serde_json::to_string_pretty(&entry_to_save)
            .map_err(|e| TranslateError::Cache(format!("JSON serialize error: {}", e)))?;

        let cache_path = self.get_cache_path(&entry.file_hash);
        std::fs::write(&cache_path, data)
            .map_err(|e| TranslateError::Cache(format!("Failed to write cache file: {}", e)))?;

        Ok(())
    }

    fn invalidate(&self, file_hash: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_path = self.get_cache_path(file_hash);

        match std::fs::remove_file(&cache_path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(TranslateError::Cache(format!(
                "Failed to remove cache file: {}",
                e
            ))),
        }
    }

    fn clear(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_dir = self.get_cache_dir();

        match std::fs::remove_dir_all(&cache_dir) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(TranslateError::Cache(format!(
                "Failed to clear cache dir: {}",
                e
            ))),
        }
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }

    fn list_entries(&self) -> Result<Vec<CacheEntryInfo>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let cache_dir = self.get_cache_dir();

        let mut result = Vec::new();

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(e) => e,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(TranslateError::Cache(format!(
                    "Failed to read cache dir: {}",
                    e
                )))
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            let name = entry.file_name();

            if name == ".gitignore" {
                continue;
            }

            let name_str = name.to_string_lossy();
            if !name_str.ends_with(".json") {
                continue;
            }

            let data = match std::fs::read_to_string(&path) {
                Ok(d) => d,
                Err(_) => continue,
            };

            if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&data) {
                result.push(CacheEntryInfo {
                    file_hash: name_str.trim_end_matches(".json").to_string(),
                    file_path: cache_entry.file_path,
                });
            }
        }

        Ok(result)
    }

    fn cleanup_orphaned(&self, existing_hashes: HashMap<String, bool>) -> Result<usize> {
        if !self.config.enabled {
            return Ok(0);
        }

        let cache_dir = self.get_cache_dir();

        let mut cleaned_count = 0;

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(e) => e,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => {
                return Err(TranslateError::Cache(format!(
                    "Failed to read cache dir: {}",
                    e
                )))
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            let name = entry.file_name();

            if name == ".gitignore" {
                continue;
            }

            let name_str = name.to_string_lossy();
            if !name_str.ends_with(".json") {
                continue;
            }

            let file_hash = name_str.trim_end_matches(".json");

            if !existing_hashes.contains_key(file_hash) {
                if std::fs::remove_file(&path).is_ok() {
                    cleaned_count += 1;
                }
            }
        }

        Ok(cleaned_count)
    }

    fn stats(&self) -> Result<CacheStats> {
        if !self.config.enabled {
            return Ok(CacheStats::default());
        }

        let cache_dir = self.get_cache_dir();

        let mut stats = CacheStats::default();

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(e) => e,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(CacheStats::default())
            }
            Err(e) => {
                return Err(TranslateError::Cache(format!(
                    "Failed to read cache dir: {}",
                    e
                )))
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            stats.entry_count += 1;

            if let Ok(metadata) = std::fs::metadata(&path) {
                stats.total_size += metadata.len() as u64;
            }
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_cache_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: crate::core::models::CacheMode::Local,
            directory: ".cache".to_string(),
            format: "json".to_string(),
        };

        let cache = FileCache::new(config, temp_dir.path()).unwrap();

        // Test set and get
        let entry = CacheEntry::new(
            "test_hash",
            "/path/to/file.txt",
            123456,
            "local",
            "fingerprint123",
        );

        cache.set(&entry).unwrap();

        let retrieved = cache.get("test_hash").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.file_hash, "test_hash");
        assert_eq!(retrieved.file_path, "/path/to/file.txt");

        // Test stats
        let stats = cache.stats().unwrap();
        assert_eq!(stats.entry_count, 1);

        // Test invalidate
        cache.invalidate("test_hash").unwrap();

        let retrieved = cache.get("test_hash").unwrap();
        assert!(retrieved.is_none());

        // Test close
        cache.close().unwrap();
    }

    #[test]
    fn test_file_cache_list_entries() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: crate::core::models::CacheMode::Local,
            directory: ".cache".to_string(),
            format: "json".to_string(),
        };

        let cache = FileCache::new(config, temp_dir.path()).unwrap();

        let entry1 = CacheEntry::new(
            "hash1",
            "/path/to/file1.txt",
            123456,
            "local",
            "fingerprint123",
        );
        let entry2 = CacheEntry::new(
            "hash2",
            "/path/to/file2.txt",
            123456,
            "local",
            "fingerprint123",
        );

        cache.set(&entry1).unwrap();
        cache.set(&entry2).unwrap();

        let entries = cache.list_entries().unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_file_cache_cleanup_orphaned() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: crate::core::models::CacheMode::Local,
            directory: ".cache".to_string(),
            format: "json".to_string(),
        };

        let cache = FileCache::new(config, temp_dir.path()).unwrap();

        let entry1 = CacheEntry::new(
            "hash1",
            "/path/to/file1.txt",
            123456,
            "local",
            "fingerprint123",
        );
        let entry2 = CacheEntry::new(
            "hash2",
            "/path/to/file2.txt",
            123456,
            "local",
            "fingerprint123",
        );

        cache.set(&entry1).unwrap();
        cache.set(&entry2).unwrap();

        let mut existing_hashes = HashMap::new();
        existing_hashes.insert("hash1".to_string(), true);

        let cleaned = cache.cleanup_orphaned(existing_hashes).unwrap();
        assert_eq!(cleaned, 1);

        let entries = cache.list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_hash, "hash1");
    }
}
