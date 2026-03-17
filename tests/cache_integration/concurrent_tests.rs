//! Concurrent cache integration tests
//!
//! Tests cache behavior under concurrent access scenarios.

use codebase_translate::cache::{binary::BinaryCache, file::FileCache};
use codebase_translate::core::models::{CacheConfig, CacheMode};
use codebase_translate::Cache;
use std::sync::Arc;
use std::thread;
use crate::test_utils::hash_utils;

#[test]
fn test_file_cache_concurrent_reads() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = Arc::new(FileCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint,
    );
    cache.set(&entry).unwrap();

    let mut handles = vec![];

    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let retrieved = cache_clone.get("hash1").unwrap();
            assert!(retrieved.is_some());
            retrieved.unwrap().file_hash
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert_eq!(result, "hash1");
    }
}

#[test]
fn test_binary_cache_concurrent_reads() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456,
        "local",
        fingerprint,
    );
    cache.set(&entry).unwrap();

    let mut handles = vec![];

    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let hash1_clone = hash1.clone();
        let handle = thread::spawn(move || {
            let retrieved = cache_clone.get(&hash1_clone).unwrap();
            assert!(retrieved.is_some());
            retrieved.unwrap().file_hash
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert_eq!(result, hash1);
    }
}

#[test]
fn test_file_cache_concurrent_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = Arc::new(FileCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let fingerprint_clone = fingerprint.clone();
        let handle = thread::spawn(move || {
            let hash = format!("hash{}", i);
            let path = format!("/path/to/file{}.txt", i);
            let entry = codebase_translate::core::models::CacheEntry::new(
                &hash,
                &path,
                123456i64 + i as i64,
                "local",
                fingerprint_clone,
            );
            cache_clone.set(&entry).unwrap();
            hash
        });
        handles.push(handle);
    }

    let mut hashes = vec![];
    for handle in handles {
        let hash = handle.join().unwrap();
        hashes.push(hash);
    }

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 10);

    for hash in &hashes {
        let retrieved = cache.get(hash).unwrap();
        assert!(retrieved.is_some());
    }
}

#[test]
fn test_binary_cache_concurrent_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let fingerprint_clone = fingerprint.clone();
        let handle = thread::spawn(move || {
            let hash = hash_utils::generate_test_hash(&format!("file{}", i));
            let path = format!("/path/to/file{}.txt", i);
            let entry = codebase_translate::core::models::CacheEntry::new(
                &hash,
                &path,
                123456i64 + i as i64,
                "local",
                fingerprint_clone,
            );
            cache_clone.set(&entry).unwrap();
            hash
        });
        handles.push(handle);
    }

    let mut hashes = vec![];
    for handle in handles {
        let hash = handle.join().unwrap();
        hashes.push(hash);
    }

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 10);

    for hash in &hashes {
        let retrieved = cache.get(hash).unwrap();
        assert!(retrieved.is_some());
    }
}

#[test]
fn test_file_cache_concurrent_mixed_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = Arc::new(FileCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let mut handles = vec![];

    for i in 0..20 {
        let cache_clone = Arc::clone(&cache);
        let fingerprint_clone = fingerprint.clone();
        let handle = thread::spawn(move || {
            if i % 3 == 0 {
                let hash = format!("hash{}", i);
                let path = format!("/path/to/file{}.txt", i);
                let entry = codebase_translate::core::models::CacheEntry::new(
                    &hash,
                    &path,
                    123456i64 + i as i64,
                    "local",
                    fingerprint_clone,
                );
                cache_clone.set(&entry).unwrap();
            } else if i % 3 == 1 {
                let hash = format!("hash{}", i - 1);
                cache_clone.get(&hash).unwrap();
            } else {
                let hash = format!("hash{}", i - 2);
                cache_clone.invalidate(&hash).unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats().unwrap();
    assert!(stats.entry_count > 0);
}

#[test]
fn test_binary_cache_concurrent_mixed_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let mut handles = vec![];

    for i in 0..20 {
        let cache_clone = Arc::clone(&cache);
        let fingerprint_clone = fingerprint.clone();
        let handle = thread::spawn(move || {
            if i % 3 == 0 {
                let hash = format!("hash{:02}_123456789012345678", i);
                let path = format!("/path/to/file{}.txt", i);
                let entry = codebase_translate::core::models::CacheEntry::new(
                    &hash,
                    &path,
                    123456i64 + i as i64,
                    "local",
                    fingerprint_clone,
                );
                cache_clone.set(&entry).unwrap();
            } else if i % 3 == 1 {
                let hash = format!("hash{:02}_123456789012345678", i - 1);
                cache_clone.get(&hash).unwrap();
            } else {
                let hash = format!("hash{:02}_123456789012345678", i - 2);
                cache_clone.invalidate(&hash).unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats().unwrap();
    assert!(stats.entry_count > 0);
}

#[test]
fn test_file_cache_concurrent_list_entries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = Arc::new(FileCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    for i in 0..10 {
        let hash = format!("hash{}", i);
        let path = format!("/path/to/file{}.txt", i);
        let entry = codebase_translate::core::models::CacheEntry::new(
            &hash,
            &path,
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];

    for _ in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let entries = cache_clone.list_entries().unwrap();
            entries.len()
        });
        handles.push(handle);
    }

    for handle in handles {
        let count = handle.join().unwrap();
        assert_eq!(count, 10);
    }
}

#[test]
fn test_binary_cache_concurrent_list_entries() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    for i in 0..10 {
        let hash = format!("hash{:02}_123456789012345678", i);
        let path = format!("/path/to/file{}.txt", i);
        let entry = codebase_translate::core::models::CacheEntry::new(
            &hash,
            &path,
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];

    for _ in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let entries = cache_clone.list_entries().unwrap();
            entries.len()
        });
        handles.push(handle);
    }

    for handle in handles {
        let count = handle.join().unwrap();
        assert_eq!(count, 10);
    }
}

#[test]
fn test_file_cache_concurrent_stats() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache = Arc::new(FileCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    for i in 0..10 {
        let hash = format!("hash{}", i);
        let path = format!("/path/to/file{}.txt", i);
        let entry = codebase_translate::core::models::CacheEntry::new(
            &hash,
            &path,
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];

    for _ in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let stats = cache_clone.stats().unwrap();
            stats.entry_count
        });
        handles.push(handle);
    }

    for handle in handles {
        let count = handle.join().unwrap();
        assert_eq!(count, 10);
    }
}

#[test]
fn test_binary_cache_concurrent_stats() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    for i in 0..10 {
        let hash = format!("hash{:02}_123456789012345678", i);
        let path = format!("/path/to/file{}.txt", i);
        let entry = codebase_translate::core::models::CacheEntry::new(
            &hash,
            &path,
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];

    for _ in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let stats = cache_clone.stats().unwrap();
            stats.entry_count
        });
        handles.push(handle);
    }

    for handle in handles {
        let count = handle.join().unwrap();
        assert_eq!(count, 10);
    }
}
