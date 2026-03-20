//! Cache validation tests
//!
//! Tests for cache entry validation, fingerprint matching, and consistency checks.

use crate::cache_integration::test_utils::hash_utils;
use codebase_translate::cache::binary::BinaryCache;
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};

#[test]
fn test_cache_fingerprint_validation() {
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

    // Should retrieve with matching fingerprint
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();
    assert_eq!(entry.project_fingerprint, fingerprint);
}

#[test]
fn test_cache_fingerprint_mismatch() {
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

    // Create a new cache instance with a different project path (different fingerprint)
    let temp_dir2 = tempfile::tempdir().unwrap();
    let config2 = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache2 = BinaryCache::new(config2, temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    // Fingerprints should be different
    assert_ne!(fingerprint, fingerprint2);

    // Should not retrieve with different fingerprint
    let retrieved = cache2.get(&hash1).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_cache_fingerprint_consistency() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    // Create two cache instances for the same project
    let cache1 = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
    let cache2 = BinaryCache::new(config, temp_dir.path()).unwrap();

    let fingerprint1 = cache1.project_fingerprint().to_string();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    // Fingerprints should be identical
    assert_eq!(fingerprint1, fingerprint2);

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

    // Close cache1 to ensure data is written
    cache1.close().unwrap();

    // Close cache2 and create a new instance to verify persistence
    cache2.close().unwrap();

    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };
    let cache3 = BinaryCache::new(config, temp_dir.path()).unwrap();

    // Cache3 should retrieve the entry
    let retrieved3 = cache3.get(&hash1).unwrap();
    assert!(retrieved3.is_some());

    let entry3 = retrieved3.unwrap();
    assert_eq!(entry3.file_hash, hash1);
    assert_eq!(entry3.file_path, "/path/to/file1.txt");
    assert!(entry3.is_translated);
}

#[test]
fn test_cache_fingerprint_different_projects() {
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

    // Different projects should have different fingerprints
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

    // Cache1 should retrieve the entry
    let retrieved1 = cache1.get(&hash1).unwrap();
    assert!(retrieved1.is_some());

    // Cache2 should NOT retrieve the entry (different fingerprint)
    let retrieved2 = cache2.get(&hash1).unwrap();
    assert!(retrieved2.is_none());
}

#[test]
fn test_cache_fingerprint_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    // Fingerprint should be a 16-character hex string
    assert_eq!(fingerprint.len(), 16);
    assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));

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

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();
    assert_eq!(entry.project_fingerprint.len(), 16);
}

#[test]
fn test_cache_fingerprint_with_file_modification() {
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

    // Retrieve with same modification time
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();
    assert!(entry.is_valid(123456));

    // Should be invalid with different modification time
    assert!(!entry.is_valid(123457));

    // Create new entry with updated modification time
    let mut entry2 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123457i64,
        "local",
        fingerprint,
    );
    entry2.mark_as_translated();

    cache.set(&entry2).unwrap();

    // Now should be valid with new modification time
    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();
    assert!(entry.is_valid(123457));
}

#[test]
fn test_cache_translation_status_persistence() {
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

    // Initially not translated
    assert!(!entry1.is_translated);
    assert_eq!(entry1.translation_timestamp, 0);

    cache.set(&entry1).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();

    // Should still not be translated
    assert!(!entry.is_translated);
    assert_eq!(entry.translation_timestamp, 0);

    // Mark as translated
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
    let entry = retrieved.unwrap();

    // Now should be translated
    assert!(entry.is_translated);
    assert!(entry.translation_timestamp > 0);
}
