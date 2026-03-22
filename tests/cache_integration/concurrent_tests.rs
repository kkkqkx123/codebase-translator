//! Concurrent access tests
//!
//! Tests BinaryCache behavior under concurrent access scenarios.

use crate::cache_integration::test_utils::hash_utils;
use codebase_translate::cache::binary::BinaryCache;
use codebase_translate::core::models::{CacheConfig, CacheEntry, CacheMode};
use std::sync::Arc;
use std::thread;

#[test]
fn test_concurrent_reads() {
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    let mut handles = vec![];
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let hash_clone = hash1.clone();
        let handle = thread::spawn(move || {
            let retrieved = cache_clone.get(&hash_clone).unwrap();
            assert!(retrieved.is_some());
            let entry = retrieved.unwrap();
            assert_eq!(entry.file_hash, hash_clone);
            assert!(entry.is_translated);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    // Write entries sequentially to avoid concurrent write conflicts
    for i in 0..10 {
        let hash = hash_utils::generate_test_hash(&format!("file{}", i));
        let mut entry = CacheEntry::new(
            &hash,
            format!("/path/to/file{}.txt", i),
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
    }

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 10);
}

#[test]
fn test_concurrent_read_write() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    let mut write_handles = vec![];

    // First, write all entries
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let fingerprint_clone = fingerprint.clone();
        let handle = thread::spawn(move || {
            let hash = hash_utils::generate_test_hash(&format!("file{}", i));
            let mut entry = CacheEntry::new(
                &hash,
                format!("/path/to/file{}.txt", i),
                123456i64 + i as i64,
                "local",
                fingerprint_clone,
            );
            entry.mark_as_translated();
            cache_clone.set(&entry).unwrap();
        });
        write_handles.push(handle);
    }

    // Wait for all writes to complete
    for handle in write_handles {
        handle.join().unwrap();
    }

    // Then, read all entries
    let mut read_handles = vec![];
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let hash = hash_utils::generate_test_hash(&format!("file{}", i));
            let retrieved = cache_clone.get(&hash).unwrap();
            assert!(retrieved.is_some());
        });
        read_handles.push(handle);
    }

    for handle in read_handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_invalidate() {
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
    let mut entry1 = CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint.clone(),
    );
    entry1.mark_as_translated();

    cache.set(&entry1).unwrap();

    let mut handles = vec![];
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let hash_clone = hash1.clone();
        let handle = thread::spawn(move || {
            cache_clone.invalidate(&hash_clone).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_concurrent_clear() {
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
        let hash = hash_utils::generate_test_hash(&format!("file{}", i));
        let mut entry = CacheEntry::new(
            &hash,
            format!("/path/to/file{}.txt", i),
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
    }

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 10);

    // Clear cache once
    cache.clear().unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);
}

#[test]
fn test_concurrent_stats() {
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
        let hash = hash_utils::generate_test_hash(&format!("file{}", i));
        let mut entry = CacheEntry::new(
            &hash,
            format!("/path/to/file{}.txt", i),
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let stats = cache_clone.stats().unwrap();
            assert_eq!(stats.entry_count, 10);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_list_entries() {
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
        let hash = hash_utils::generate_test_hash(&format!("file{}", i));
        let mut entry = CacheEntry::new(
            &hash,
            format!("/path/to/file{}.txt", i),
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];
    for _ in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let entries = cache_clone.list_entries().unwrap();
            assert_eq!(entries.len(), 10);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_mixed_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = Arc::new(BinaryCache::new(config, temp_dir.path()).unwrap());
    let fingerprint = cache.project_fingerprint().to_string();

    // Write all entries first
    for i in 0..5 {
        let hash = hash_utils::generate_test_hash(&format!("file{}", i));
        let mut entry = CacheEntry::new(
            &hash,
            format!("/path/to/file{}.txt", i),
            123456i64 + i as i64,
            "local",
            fingerprint.clone(),
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
    }

    let mut handles = vec![];

    // Concurrent reads
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let hash = hash_utils::generate_test_hash(&format!("file{}", i));
            let _ = cache_clone.get(&hash).unwrap();
        });
        handles.push(handle);
    }

    // Concurrent stats
    for _ in 0..3 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let _ = cache_clone.stats().unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 5);
}
