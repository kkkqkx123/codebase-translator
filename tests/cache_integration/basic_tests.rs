//! Basic cache integration tests
//!
//! Tests the core functionality of both FileCache and BinaryCache implementations.

use codebase_translate::cache::{binary::BinaryCache, file::FileCache};
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};
use codebase_translate::Cache;
use std::collections::HashMap;
use crate::cache_integration::test_utils::hash_utils;

#[test]
fn test_file_cache_set_and_get() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );

    cache.set(&entry).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.file_hash, "hash1");
    assert_eq!(retrieved.file_path, "/path/to/file1.txt");
    assert_eq!(retrieved.last_modified, 123456);
}

#[test]
fn test_binary_cache_set_and_get() {
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
    let entry = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint,
    );

    cache.set(&entry).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.file_hash, hash1);
    assert_eq!(retrieved.file_path, "/path/to/file1.txt");
    assert_eq!(retrieved.last_modified, 123456);
}

#[test]
fn test_file_cache_get_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();

    let retrieved = cache.get("nonexistent_hash").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_binary_cache_get_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();

    let retrieved = cache.get("nonexistent_hash").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_file_cache_invalidate() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );

    cache.set(&entry).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());

    cache.invalidate("hash1").unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_binary_cache_invalidate() {
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
    let entry = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint,
    );

    cache.set(&entry).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());

    cache.invalidate(&hash1).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_file_cache_clear() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry1 = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        "hash2",
        "/path/to/file2.txt",
        123457i64,
        "local",
        &fingerprint,
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 2);

    cache.clear().unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);

    let retrieved1 = cache.get("hash1").unwrap();
    assert!(retrieved1.is_none());

    let retrieved2 = cache.get("hash2").unwrap();
    assert!(retrieved2.is_none());
}

#[test]
fn test_binary_cache_clear() {
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
    let hash2 = hash_utils::generate_test_hash("file2");
    let entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457,
        "local",
        fingerprint,
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 2);

    cache.clear().unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);

    let retrieved1 = cache.get(&hash1).unwrap();
    assert!(retrieved1.is_none());

    let retrieved2 = cache.get(&hash2).unwrap();
    assert!(retrieved2.is_none());
}

#[test]
fn test_file_cache_list_entries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry1 = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        "hash2",
        "/path/to/file2.txt",
        123457i64,
        "local",
        &fingerprint,
    );
    let entry3 = CacheEntry::new(
        "hash3",
        "/path/to/file3.txt",
        123458i64,
        "local",
        fingerprint.clone(),
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();
    cache.set(&entry3).unwrap();

    let entries = cache.list_entries().unwrap();
    assert_eq!(entries.len(), 3);

    let file_paths: Vec<_> = entries.iter().map(|e| &e.file_path).collect();
    assert!(file_paths.contains(&&"/path/to/file1.txt".to_string()));
    assert!(file_paths.contains(&&"/path/to/file2.txt".to_string()));
    assert!(file_paths.contains(&&"/path/to/file3.txt".to_string()));
}

#[test]
fn test_binary_cache_list_entries() {
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
    let hash2 = hash_utils::generate_test_hash("file2");
    let hash3 = hash_utils::generate_test_hash("file3");
    let entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457i64,
        "local",
        &fingerprint,
    );
    let entry3 = CacheEntry::new(
        &hash3,
        "/path/to/file3.txt",
        123458i64,
        "local",
        fingerprint.clone(),
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();
    cache.set(&entry3).unwrap();

    let entries = cache.list_entries().unwrap();
    assert_eq!(entries.len(), 3);

    let file_paths: Vec<_> = entries.iter().map(|e| &e.file_path).collect();
    assert!(file_paths.contains(&&"/path/to/file1.txt".to_string()));
    assert!(file_paths.contains(&&"/path/to/file2.txt".to_string()));
    assert!(file_paths.contains(&&"/path/to/file3.txt".to_string()));
}

#[test]
fn test_file_cache_stats() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry1 = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        "hash2",
        "/path/to/file2.txt",
        123457i64,
        "local",
        &fingerprint,
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 2);
    assert!(stats.total_size > 0);
}

#[test]
fn test_binary_cache_stats() {
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
    let hash2 = hash_utils::generate_test_hash("file2");
    let entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457,
        "local",
        fingerprint,
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 2);
    assert!(stats.total_size > 0);
}

#[test]
fn test_file_cache_cleanup_orphaned() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry1 = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        "hash2",
        "/path/to/file2.txt",
        123457i64,
        "local",
        &fingerprint,
    );
    let entry3 = CacheEntry::new(
        "hash3",
        "/path/to/file3.txt",
        123458i64,
        "local",
        fingerprint.clone(),
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();
    cache.set(&entry3).unwrap();

    let mut existing_hashes = HashMap::new();
    existing_hashes.insert("hash1".to_string(), true);
    existing_hashes.insert("hash3".to_string(), true);

    let cleaned = cache.cleanup_orphaned(existing_hashes).unwrap();
    assert_eq!(cleaned, 1);

    let entries = cache.list_entries().unwrap();
    assert_eq!(entries.len(), 2);

    let hashes: Vec<_> = entries.iter().map(|e| &e.file_hash).collect();
    assert!(hashes.contains(&&"hash1".to_string()));
    assert!(hashes.contains(&&"hash3".to_string()));
}

#[test]
fn test_binary_cache_cleanup_orphaned() {
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
    let hash2 = hash_utils::generate_test_hash("file2");
    let hash3 = hash_utils::generate_test_hash("file3");
    let entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457i64,
        "local",
        &fingerprint,
    );
    let entry3 = CacheEntry::new(
        &hash3,
        "/path/to/file3.txt",
        123458i64,
        "local",
        fingerprint.clone(),
    );

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();
    cache.set(&entry3).unwrap();

    let mut existing_hashes = HashMap::new();
    existing_hashes.insert(hash1.clone(), true);
    existing_hashes.insert(hash3.clone(), true);

    let cleaned = cache.cleanup_orphaned(existing_hashes).unwrap();
    assert_eq!(cleaned, 1);

    let entries = cache.list_entries().unwrap();
    assert_eq!(entries.len(), 2);

    let hashes: Vec<_> = entries.iter().map(|e| &e.file_hash).collect();
    assert!(hashes.contains(&&hash1));
    assert!(hashes.contains(&&hash3));
}

#[test]
fn test_file_cache_disabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: false,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );

    cache.set(&entry).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_none());

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);
}

#[test]
fn test_binary_cache_disabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: false,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint,
    );

    cache.set(&entry).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);
}

#[test]
fn test_file_cache_update_existing_entry() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry1 = CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    cache.set(&entry1).unwrap();

    let entry2 = CacheEntry::new(
        "hash1",
        "/path/to/file1_updated.txt",
        123457i64,
        "local",
        fingerprint,
    );
    cache.set(&entry2).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.file_path, "/path/to/file1_updated.txt");
    assert_eq!(retrieved.last_modified, 123457);
}

#[test]
fn test_binary_cache_update_existing_entry() {
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
    let entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    cache.set(&entry1).unwrap();

    let entry2 = CacheEntry::new(
        &hash1,
        "/path/to/file1_updated.txt",
        123457i64,
        "local",
        fingerprint,
    );
    cache.set(&entry2).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.file_path, "/path/to/file1_updated.txt");
    assert_eq!(retrieved.last_modified, 123457);
}
