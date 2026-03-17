//! Project fingerprint integration tests
//!
//! Tests cache behavior with project fingerprint validation.

use codebase_translate::cache::{binary::BinaryCache, file::FileCache};
use codebase_translate::core::models::{CacheConfig, CacheMode};
use codebase_translate::Cache;
use crate::test_utils::hash_utils;

#[test]
fn test_file_cache_fingerprint_validation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );
    cache.set(&entry).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_binary_cache_fingerprint_validation() {
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
    let entry = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint,
    );
    cache.set(&entry).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_file_cache_fingerprint_mismatch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        &fingerprint,
    );
    cache.set(&entry).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());

    let cache_path = temp_dir.path().join(".cache").join("hash1.json");
    let mut cache_content = std::fs::read_to_string(&cache_path).unwrap();
    cache_content = cache_content.replace(&fingerprint, "different_fingerprint");
    std::fs::write(&cache_path, cache_content).unwrap();

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_binary_cache_fingerprint_mismatch() {
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
    let entry = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        &fingerprint,
    );
    cache.set(&entry).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());

    let cache_path = temp_dir.path().join(".cache").join("cache.bin");
    let mut cache_data = std::fs::read(&cache_path).unwrap();
    let fingerprint_bytes = fingerprint.as_bytes();

    for i in 0..cache_data.len().saturating_sub(fingerprint_bytes.len()) {
        if &cache_data[i..i + fingerprint_bytes.len()] == fingerprint_bytes {
            for (j, byte) in "different_fingerprint".as_bytes().iter().enumerate() {
                if i + j < cache_data.len() {
                    cache_data[i + j] = *byte;
                }
            }
            break;
        }
    }
    std::fs::write(&cache_path, cache_data).unwrap();

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_file_cache_fingerprint_consistency() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache1 = FileCache::new(config.clone(), temp_dir.path()).unwrap();
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let cache2 = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    assert_eq!(fingerprint1, fingerprint2);

    let entry = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint1,
    );
    cache1.set(&entry).unwrap();

    let retrieved = cache2.get("hash1").unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_binary_cache_fingerprint_consistency() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache1 = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        &fingerprint1,
    );
    cache1.set(&entry).unwrap();
    cache1.close().unwrap();

    let cache2 = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    assert_eq!(fingerprint1, fingerprint2);

    let retrieved = cache2.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_file_cache_fingerprint_different_projects() {
    let temp_dir1 = tempfile::tempdir().unwrap();
    let temp_dir2 = tempfile::tempdir().unwrap();

    let config1 = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let config2 = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache1 = FileCache::new(config1, temp_dir1.path()).unwrap();
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let cache2 = FileCache::new(config2, temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    assert_ne!(fingerprint1, fingerprint2);

    let entry1 = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint1,
    );
    cache1.set(&entry1).unwrap();

    let entry2 = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint2,
    );
    cache2.set(&entry2).unwrap();

    let retrieved1 = cache1.get("hash1").unwrap();
    assert!(retrieved1.is_some());

    let retrieved2 = cache2.get("hash1").unwrap();
    assert!(retrieved2.is_some());
}

#[test]
fn test_binary_cache_fingerprint_different_projects() {
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
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let cache2 = BinaryCache::new(config2, temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    assert_ne!(fingerprint1, fingerprint2);

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry1 = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint1,
    );
    cache1.set(&entry1).unwrap();

    let entry2 = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint2,
    );
    cache2.set(&entry2).unwrap();

    let retrieved1 = cache1.get(&hash1).unwrap();
    assert!(retrieved1.is_some());

    let retrieved2 = cache2.get(&hash1).unwrap();
    assert!(retrieved2.is_some());
}

#[test]
fn test_file_cache_fingerprint_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    assert_eq!(fingerprint.len(), 16);
    assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_binary_cache_fingerprint_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    assert_eq!(fingerprint.len(), 16);
    assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_file_cache_fingerprint_with_file_modification() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint1 = cache.project_fingerprint().to_string();

    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "initial content").unwrap();

    let cache2 = FileCache::new(
        CacheConfig {
            enabled: true,
            mode: CacheMode::Local,
            directory: ".cache".to_string(),
            format: "json".to_string(),
        },
        temp_dir.path(),
    )
    .unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    assert_eq!(fingerprint1, fingerprint2);
}

#[test]
fn test_binary_cache_fingerprint_with_file_modification() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint1 = cache.project_fingerprint().to_string();

    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "initial content").unwrap();

    let cache2 = BinaryCache::new(
        CacheConfig {
            enabled: true,
            mode: CacheMode::Local,
            directory: ".cache".to_string(),
            format: "binary".to_string(),
        },
        temp_dir.path(),
    )
    .unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    assert_eq!(fingerprint1, fingerprint2);
}
