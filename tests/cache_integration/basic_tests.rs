//! Basic cache integration tests
//!
//! Tests core functionality of BinaryCache with new cache structure.

use crate::cache_integration::test_utils::hash_utils;
use codebase_translate::cache::binary::BinaryCache;
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};
use std::collections::HashMap;

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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.file_hash, hash1);
    assert_eq!(retrieved.file_path, "/path/to/file1.txt");
    assert_eq!(retrieved.last_modified, 123456);
    assert!(retrieved.is_translated);
    assert!(retrieved.translation_timestamp > 0);
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    // Verify entry exists
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());

    // Invalidate
    cache.invalidate(&hash1).unwrap();

    // Verify entry is gone
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let mut entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();
    entry2.mark_as_translated();

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    // Verify entries exist
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 2);

    // Clear cache
    cache.clear().unwrap();

    // Verify entries are gone
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);

    let retrieved1 = cache.get(&hash1).unwrap();
    let retrieved2 = cache.get(&hash2).unwrap();
    assert!(retrieved1.is_none());
    assert!(retrieved2.is_none());
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let mut entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457i64,
        "local",
        fingerprint.clone(),
    );
    let mut entry3 = CacheEntry::new(
        &hash3,
        "/path/to/file3.txt",
        123458i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();
    entry2.mark_as_translated();
    entry3.mark_as_translated();

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

    // Initially empty
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);

    // Add entries
    let hash1 = hash_utils::generate_test_hash("file1");
    let hash2 = hash_utils::generate_test_hash("file2");
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let mut entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();
    entry2.mark_as_translated();

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    // Check stats
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 2);
    assert!(stats.total_size > 0);
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    let mut entry2 = CacheEntry::new(
        &hash2,
        "/path/to/file2.txt",
        123457i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();
    entry2.mark_as_translated();

    cache.set(&entry1).unwrap();
    cache.set(&entry2).unwrap();

    let mut existing_hashes = HashMap::new();
    existing_hashes.insert(hash1.clone(), true);

    let cleaned = cache.cleanup_orphaned(existing_hashes).unwrap();
    assert_eq!(cleaned, 1);

    let entries = cache.list_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_hash, hash1);
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );
    entry1.mark_as_translated();

    // Should not fail even when disabled
    cache.set(&entry1).unwrap();

    // Should return None when disabled
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    // Update with new modification time
    let mut entry2 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123457i64,
        "local",
        fingerprint,
    );
    entry2.mark_as_translated();

    cache.set(&entry2).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.last_modified, 123457);
}

#[test]
fn test_binary_cache_is_valid() {
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
        fingerprint,
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();

    // Same modification time - valid
    assert!(entry.is_valid(123456));

    // Different modification time - invalid
    assert!(!entry.is_valid(123457));
}

#[test]
fn test_binary_cache_mark_as_translated() {
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
        fingerprint,
    );

    // Initially not translated
    assert!(!entry1.is_translated);
    assert_eq!(entry1.translation_timestamp, 0);

    // Mark as translated
    entry1.mark_as_translated();

    // Now translated
    assert!(entry1.is_translated);
    assert!(entry1.translation_timestamp > 0);

    cache.set(&entry1).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();
    assert!(entry.is_translated);
    assert!(entry.translation_timestamp > 0);
}
