//! Cache hit integration tests
//!
//! Tests for cache hit scenarios in the translation workflow.
//! These tests verify that cached translations are correctly reused
//! to skip redundant translation API calls.

use crate::cache_integration::test_utils::{hash_utils, TEST_CONFIG_HASH};
use codebase_translate::cache::binary::BinaryCache;
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};

/// Test basic cache hit scenario
/// Verifies that a translated file can be retrieved from cache
#[test]
fn test_cache_hit_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );

    // Mark entry as translated
    entry.mark_as_translated();

    // Store in cache
    cache.set(&entry).unwrap();

    // Simulate cache hit check
    let cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap();
    assert!(cached_entry.is_some(), "Cache should contain the entry");

    let cached_entry = cached_entry.unwrap();
    assert!(
        cached_entry.is_translated,
        "Entry should be marked as translated"
    );
    assert!(
        cached_entry.is_valid(123456),
        "Entry should be valid with same modification time"
    );
}

/// Test cache hit with translation status persistence
/// Verifies that is_translated flag is correctly persisted and retrieved
#[test]
fn test_cache_hit_translation_status() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");

    // Create entry without marking as translated
    let entry_not_translated = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );

    // Should NOT be translated initially
    assert!(!entry_not_translated.is_translated);
    assert_eq!(entry_not_translated.translation_timestamp, 0);

    cache.set(&entry_not_translated).unwrap();

    // Retrieve and verify not translated
    let retrieved = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    assert!(!retrieved.is_translated);

    // Now mark as translated and update
    let mut entry_translated = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry_translated.mark_as_translated();
    cache.set(&entry_translated).unwrap();

    // Retrieve and verify translated
    let retrieved = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    assert!(
        retrieved.is_translated,
        "Entry should be marked as translated"
    );
    assert!(
        retrieved.translation_timestamp > 0,
        "Translation timestamp should be set"
    );
}

/// Test cache miss scenarios
#[test]
fn test_cache_miss_scenarios() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
    let fingerprint1 = cache.project_fingerprint().to_string();
    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint1.clone(),
        TEST_CONFIG_HASH,
    );
    entry.mark_as_translated();
    cache.set(&entry).unwrap();

    // Create cache with different fingerprint (different project dir)
    let temp_dir2 = tempfile::tempdir().unwrap();
    let cache2 = BinaryCache::new(config.clone(), temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();
    assert_ne!(
        fingerprint1, fingerprint2,
        "Different projects should have different fingerprints"
    );

    // Should not retrieve entry from different project
    let result = cache2.get(&file_hash, TEST_CONFIG_HASH).unwrap();
    assert!(
        result.is_none(),
        "Should not retrieve entry from different project"
    );
}

/// Test cache hit with modification time validation
#[test]
fn test_cache_hit_with_modified_time() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry.mark_as_translated();
    cache.set(&entry).unwrap();

    // Cache hit with same modification time
    let cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    assert!(
        cached_entry.is_valid(123456),
        "Should be valid with same mtime"
    );

    // Cache miss with different modification time (file was modified)
    assert!(
        !cached_entry.is_valid(999999),
        "Should be invalid with different mtime"
    );
}

/// Test cache persistence across cache instance recreation
#[test]
fn test_cache_hit_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    // Create cache and add entry
    let cache1 = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
    let fingerprint = cache1.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry.mark_as_translated();
    cache1.set(&entry).unwrap();

    // Close first cache
    cache1.close().unwrap();

    // Create new cache instance (simulating program restart)
    let cache2 = BinaryCache::new(config, temp_dir.path()).unwrap();

    // Should retrieve cached entry
    let cached_entry = cache2.get(&file_hash, TEST_CONFIG_HASH).unwrap();
    assert!(
        cached_entry.is_some(),
        "Cache should persist across instances"
    );

    let cached_entry = cached_entry.unwrap();
    assert!(
        cached_entry.is_translated,
        "Translation status should persist"
    );
    assert_eq!(cached_entry.file_hash, file_hash);
}

/// Test multiple cache hits in sequence
#[test]
fn test_multiple_cache_hits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    // Add multiple entries
    let mut entries = Vec::new();
    for i in 0..5 {
        let file_hash = hash_utils::generate_test_hash(&format!("file_{}", i));
        let mut entry = CacheEntry::new(
            &file_hash,
            format!("/path/to/file_{}.txt", i),
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
            TEST_CONFIG_HASH,
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
        entries.push((file_hash, entry));
    }

    // Verify all entries can be retrieved (cache hits)
    for (file_hash, original_entry) in &entries {
        let cached_entry = cache.get(file_hash, TEST_CONFIG_HASH).unwrap();
        assert!(
            cached_entry.is_some(),
            "Should retrieve entry for {}",
            file_hash
        );

        let cached_entry = cached_entry.unwrap();
        assert!(cached_entry.is_translated, "Entry should be translated");
        assert_eq!(cached_entry.file_path, original_entry.file_path);
    }
}

/// Test cache hit with disabled cache
#[test]
fn test_cache_hit_with_disabled_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: false,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry.mark_as_translated();

    // Set should succeed but not actually cache
    cache.set(&entry).unwrap();

    // Get should return None when cache is disabled
    let result = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap();
    assert!(result.is_none(), "Disabled cache should always return None");
}

/// Test cache entry with translation timestamp
#[test]
fn test_cache_entry_translation_timestamp() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );

    let before_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    entry.mark_as_translated();

    let after_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    cache.set(&entry).unwrap();

    let retrieved = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    assert!(
        retrieved.translation_timestamp >= before_timestamp
            && retrieved.translation_timestamp <= after_timestamp,
        "Translation timestamp should be set to current time"
    );
}

/// Test cache hit rate calculation
#[test]
fn test_cache_hit_rate_calculation() {
    use codebase_translate::reporter::stats::TranslationStats;

    let mut stats = TranslationStats::new();

    // Initially 0% hit rate
    assert_eq!(stats.get_cache_hit_rate(), 0.0);

    // Add some cache hits and misses
    stats.record_cache_hit();
    stats.record_cache_hit();
    stats.record_cache_miss();
    stats.record_cache_miss();

    // 2 hits out of 4 = 50%
    assert!((stats.get_cache_hit_rate() - 50.0).abs() < 0.01);

    // Add more hits
    stats.record_cache_hit();
    stats.record_cache_hit();

    // 4 hits out of 6 = 66.67%
    assert!((stats.get_cache_hit_rate() - 66.67).abs() < 0.1);
}

/// Test cache statistics tracking
#[test]
fn test_cache_statistics_tracking() {
    use codebase_translate::reporter::stats::TranslationStats;

    let mut stats = TranslationStats::new();

    // Simulate translation workflow with cache
    // File 1: cache miss
    stats.record_cache_miss();
    stats.record_translated(1);

    // File 2: cache hit
    stats.record_cache_hit();
    stats.record_skipped();

    // File 3: cache miss
    stats.record_cache_miss();
    stats.record_translated(1);

    // File 4: cache hit
    stats.record_cache_hit();
    stats.record_skipped();

    // File 5: cache hit
    stats.record_cache_hit();
    stats.record_skipped();

    // Verify statistics
    assert_eq!(stats.cache_hit_count, 3);
    assert_eq!(stats.cache_miss_count, 2);
    assert_eq!(stats.translated_units, 2);
    assert_eq!(stats.skipped_files, 3);

    // Hit rate should be 3/5 = 60%
    assert!((stats.get_cache_hit_rate() - 60.0).abs() < 0.01);
}

/// Test cache invalidation when file is modified
#[test]
fn test_cache_invalidation_on_file_modification() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");

    // Initial cache entry with modification time 1000
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        1000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry.mark_as_translated();
    cache.set(&entry).unwrap();

    // Cache hit with same modification time
    let cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    assert!(
        cached_entry.is_valid(1000),
        "Should be valid with same mtime"
    );

    // File is modified, new modification time 2000
    // Cache should be considered invalid
    assert!(
        !cached_entry.is_valid(2000),
        "Should be invalid after file modification"
    );

    // Update cache entry with new modification time
    let mut updated_entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        2000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    updated_entry.mark_as_translated();
    cache.set(&updated_entry).unwrap();

    // New cache entry should be valid with new mtime
    let new_cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    assert!(
        new_cached_entry.is_valid(2000),
        "Should be valid with updated mtime"
    );
    assert!(
        !new_cached_entry.is_valid(1000),
        "Should be invalid with old mtime"
    );
}

/// Test cache invalidation when content hash changes
#[test]
fn test_cache_invalidation_on_content_change() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    // Original content hash
    let original_hash = hash_utils::generate_test_hash("original_content");
    let mut original_entry = CacheEntry::new(
        &original_hash,
        "/path/to/test.txt",
        1000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    original_entry.mark_as_translated();
    cache.set(&original_entry).unwrap();

    // Verify original content is cached
    assert!(cache
        .get(&original_hash, TEST_CONFIG_HASH)
        .unwrap()
        .is_some());

    // Content changes, new hash
    let modified_hash = hash_utils::generate_test_hash("modified_content");

    // Old hash should still exist (not automatically invalidated)
    assert!(cache
        .get(&original_hash, TEST_CONFIG_HASH)
        .unwrap()
        .is_some());

    // New hash should not exist yet (cache miss)
    assert!(cache
        .get(&modified_hash, TEST_CONFIG_HASH)
        .unwrap()
        .is_none());

    // Add new entry for modified content
    let mut modified_entry = CacheEntry::new(
        &modified_hash,
        "/path/to/test.txt",
        2000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    modified_entry.mark_as_translated();
    cache.set(&modified_entry).unwrap();

    // Now new hash should exist
    assert!(cache
        .get(&modified_hash, TEST_CONFIG_HASH)
        .unwrap()
        .is_some());
}

/// Test cache cleanup orphaned entries
#[test]
fn test_cache_cleanup_orphaned_entries() {
    use std::collections::HashMap;

    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    // Add 3 cache entries
    let hash1 = hash_utils::generate_test_hash("file1");
    let hash2 = hash_utils::generate_test_hash("file2");
    let hash3 = hash_utils::generate_test_hash("file3");

    for hash in [&hash1, &hash2, &hash3] {
        let mut entry = CacheEntry::new(
            hash,
            "/path/to/file.txt",
            1000i64,
            "local",
            fingerprint.clone(),
            TEST_CONFIG_HASH,
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
    }

    // Verify all entries exist
    assert_eq!(cache.stats().unwrap().entry_count, 3);

    // Simulate file2 and file3 being deleted (only file1 still exists)
    let mut existing_hashes = HashMap::new();
    existing_hashes.insert(hash1.clone(), true);

    // Cleanup orphaned entries
    let cleaned_count = cache.cleanup_orphaned(existing_hashes).unwrap();

    // Should clean up 2 orphaned entries (file2 and file3)
    assert_eq!(cleaned_count, 2);

    // Verify only file1 remains
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 1);

    assert!(cache.get(&hash1, TEST_CONFIG_HASH).unwrap().is_some());
    assert!(cache.get(&hash2, TEST_CONFIG_HASH).unwrap().is_none());
    assert!(cache.get(&hash3, TEST_CONFIG_HASH).unwrap().is_none());
}

/// Test cache entry validity check
#[test]
fn test_cache_entry_validity_check() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        1000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry.mark_as_translated();
    cache.set(&entry).unwrap();

    let cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();

    // Valid: same modification time
    assert!(cached_entry.is_valid(1000));

    // Invalid: different modification time
    assert!(!cached_entry.is_valid(999));
    assert!(!cached_entry.is_valid(1001));
    assert!(!cached_entry.is_valid(0));
}

/// Test cache hit requires both validity and translated status
#[test]
fn test_cache_hit_requires_valid_and_translated() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = hash_utils::generate_test_hash("test_file");

    // Create entry that is NOT translated
    let entry_not_translated = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        1000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    // Don't mark as translated
    cache.set(&entry_not_translated).unwrap();

    let cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();

    // Entry exists but is not translated
    assert!(!cached_entry.is_translated);
    assert!(cached_entry.is_valid(1000));

    // In workflow, this would NOT be considered a cache hit
    // because is_translated is false
    // Cache hit condition: entry.is_valid(mtime) && entry.is_translated
    let is_cache_hit = cached_entry.is_valid(1000) && cached_entry.is_translated;
    assert!(!is_cache_hit, "Should not be cache hit when not translated");

    // Now mark as translated
    let mut entry_translated = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        1000i64,
        "local",
        fingerprint.clone(),
        TEST_CONFIG_HASH,
    );
    entry_translated.mark_as_translated();
    cache.set(&entry_translated).unwrap();

    // Now it should be a cache hit
    let cached_entry = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap().unwrap();
    let is_cache_hit = cached_entry.is_valid(1000) && cached_entry.is_translated;
    assert!(
        is_cache_hit,
        "Should be cache hit when translated and valid"
    );
}
