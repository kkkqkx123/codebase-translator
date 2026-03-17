//! Cache module integration tests

use codebase_translate::cache::{binary::BinaryCache, file::FileCache};
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};
use codebase_translate::Cache;

#[test]
fn test_binary_cache_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    // Test set and get
    let entry = CacheEntry::new(
        "test_hash",
        "/path/to/file.txt",
        123456,
        "local",
        fingerprint,
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
fn test_file_cache_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    // Test set and get
    let entry = CacheEntry::new(
        "test_hash",
        "/path/to/file.txt",
        123456,
        "local",
        fingerprint,
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

    // Test close
    cache.close().unwrap();
}
