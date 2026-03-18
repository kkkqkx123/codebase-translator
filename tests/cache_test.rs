//! Cache module integration tests

use codebase_translate::cache::binary::BinaryCache;
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

    // Test set and get - use a 64-character hex hash
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let mut entry = CacheEntry::new(hash, "/path/to/file.txt", 123456, "local", fingerprint);
    entry.mark_as_translated();

    cache.set(&entry).unwrap();

    let retrieved = cache.get(hash).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.file_hash, hash);
    assert_eq!(retrieved.file_path, "/path/to/file.txt");
    assert!(retrieved.is_translated);
    assert!(retrieved.translation_timestamp > 0);

    // Test stats
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 1);

    // Test invalidate
    cache.invalidate(hash).unwrap();

    let retrieved = cache.get(hash).unwrap();
    assert!(retrieved.is_none());

    // Test close
    cache.close().unwrap();
}
