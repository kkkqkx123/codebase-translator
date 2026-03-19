//! Cache mode integration tests
//!
//! Tests BinaryCache behavior with different cache modes (Local vs Global).

use crate::cache_integration::test_utils::hash_utils;
use codebase_translate::cache::binary::BinaryCache;
use codebase_translate::cache::util;
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};
use codebase_translate::Cache;

#[test]
fn test_binary_cache_local_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    // Cache file should be in translator subdirectory
    let cache_file = temp_dir.path().join("translator").join("translator-cache.bin");
    assert!(cache_file.exists());

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_binary_cache_global_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "global",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    // Cache file should be in global cache directory under translator subdirectory
    let global_cache_dir = util::get_global_cache_dir();
    let project_id = util::generate_project_id(temp_dir.path());
    let _cache_file = global_cache_dir
        .join(&project_id)
        .join("translator")
        .join("translator-cache.bin");

    // The cache file should exist in global directory
    assert!(global_cache_dir.exists());
}

#[test]
fn test_binary_cache_local_mode_isolation() {
    let temp_dir1 = tempfile::tempdir().unwrap();
    let temp_dir2 = tempfile::tempdir().unwrap();

    let config1 = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let config2 = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache1 = BinaryCache::new(config1, temp_dir1.path()).unwrap();
    let cache2 = BinaryCache::new(config2, temp_dir2.path()).unwrap();

    let fingerprint1 = cache1.project_fingerprint().to_string();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    // Different directories should have different fingerprints
    assert_ne!(fingerprint1, fingerprint2);

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint1.clone(),
    );
    entry1.mark_as_translated();

    cache1.set(&entry1).unwrap();

    // Cache1 should have the entry
    let retrieved1 = cache1.get(&hash1).unwrap();
    assert!(retrieved1.is_some());

    // Cache2 should NOT have the entry (different project)
    let retrieved2 = cache2.get(&hash1).unwrap();
    assert!(retrieved2.is_none());
}

#[test]
fn test_binary_cache_global_mode_isolation() {
    let temp_dir1 = tempfile::tempdir().unwrap();
    let temp_dir2 = tempfile::tempdir().unwrap();

    let config1 = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let config2 = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache1 = BinaryCache::new(config1, temp_dir1.path()).unwrap();
    let cache2 = BinaryCache::new(config2, temp_dir2.path()).unwrap();

    let fingerprint1 = cache1.project_fingerprint().to_string();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    // Different directories should have different fingerprints
    assert_ne!(fingerprint1, fingerprint2);

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "global",
        fingerprint1.clone(),
    );
    entry1.mark_as_translated();

    cache1.set(&entry1).unwrap();

    // Cache1 should have the entry
    let retrieved1 = cache1.get(&hash1).unwrap();
    assert!(retrieved1.is_some());

    // Cache2 should NOT have the entry (different project)
    let retrieved2 = cache2.get(&hash1).unwrap();
    assert!(retrieved2.is_none());
}

#[test]
fn test_binary_cache_mode_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache1 = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
    let fingerprint = cache1.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache1.set(&entry1).unwrap();
    cache1.close().unwrap();

    // Create a new cache instance
    let cache2 = BinaryCache::new(config, temp_dir.path()).unwrap();

    // Should retrieve the same entry
    let retrieved = cache2.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();
    assert_eq!(entry.file_hash, hash1);
    assert!(entry.is_translated);
}

#[test]
fn test_binary_cache_custom_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let custom_dir = "custom_cache_dir";
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: custom_dir.to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    // Cache file should be in translator subdirectory (directory parameter is now ignored)
    let cache_file = temp_dir.path().join("translator").join("translator-cache.bin");
    assert!(cache_file.exists());

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_binary_cache_mode_disabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: false,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();

    let hash1 = hash_utils::generate_test_hash("file1");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        cache.project_fingerprint().to_string(),
    );
    entry1.mark_as_translated();

    // Should not create cache file when disabled
    cache.set(&entry1).unwrap();

    // Should return None when disabled
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());
}
