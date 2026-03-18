//! Cache mode integration tests
//!
//! Tests cache behavior with different cache modes (Local vs Global).

use crate::cache_integration::test_utils::hash_utils;
use codebase_translate::cache::util;
use codebase_translate::cache::{binary::BinaryCache, file::FileCache};
use codebase_translate::core::models::{CacheConfig, CacheMode};
use codebase_translate::Cache;
use std::path::PathBuf;

#[test]
fn test_file_cache_local_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".translator-cache".to_string(),
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

    let cache_dir = temp_dir.path().join(".translator-cache");
    assert!(cache_dir.exists());

    let cache_file = cache_dir.join("hash1.json");
    assert!(cache_file.exists());

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_binary_cache_local_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".translator-cache".to_string(),
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

    let cache_dir = temp_dir.path().join(".translator-cache");
    assert!(cache_dir.exists());

    let cache_file = cache_dir.join("cache.bin");
    assert!(cache_file.exists());

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_file_cache_global_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: "translator-cache".to_string(),
        format: "json".to_string(),
    };

    let cache = FileCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let entry = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "global",
        fingerprint,
    );
    cache.set(&entry).unwrap();

    let global_cache_dir = util::get_global_cache_dir();
    let project_id = util::generate_project_id(temp_dir.path());
    let cache_dir = global_cache_dir.join(&project_id).join("translator-cache");

    eprintln!("Global cache dir: {:?}", global_cache_dir);
    eprintln!("Project ID: {:?}", project_id);
    eprintln!("Cache dir: {:?}", cache_dir);
    eprintln!("Cache dir exists: {}", cache_dir.exists());

    assert!(cache_dir.exists());

    let cache_file = cache_dir.join("hash1.json");
    assert!(cache_file.exists());

    let retrieved = cache.get("hash1").unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_binary_cache_global_mode() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: "translator-cache".to_string(),
        format: "binary".to_string(),
    };

    let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "global",
        fingerprint,
    );
    cache.set(&entry).unwrap();

    let global_cache_dir = util::get_global_cache_dir();
    let project_id = util::generate_project_id(temp_dir.path());
    let cache_dir = global_cache_dir.join(&project_id).join("translator-cache");

    assert!(cache_dir.exists());

    let cache_file = cache_dir.join("cache.bin");
    assert!(cache_file.exists());

    let retrieved = cache.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_file_cache_local_mode_isolation() {
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
        "/path/to/file2.txt",
        123457i64,
        "local",
        fingerprint2,
    );
    cache2.set(&entry2).unwrap();

    let retrieved1 = cache1.get("hash1").unwrap();
    assert!(retrieved1.is_some());
    assert_eq!(retrieved1.unwrap().file_path, "/path/to/file1.txt");

    let retrieved2 = cache2.get("hash1").unwrap();
    assert!(retrieved2.is_some());
    assert_eq!(retrieved2.unwrap().file_path, "/path/to/file2.txt");
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
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let cache2 = BinaryCache::new(config2, temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

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
        "/path/to/file2.txt",
        123457i64,
        "local",
        fingerprint2,
    );
    cache2.set(&entry2).unwrap();

    let retrieved1 = cache1.get(&hash1).unwrap();
    assert!(retrieved1.is_some());
    assert_eq!(retrieved1.unwrap().file_path, "/path/to/file1.txt");

    let retrieved2 = cache2.get(&hash1).unwrap();
    assert!(retrieved2.is_some());
    assert_eq!(retrieved2.unwrap().file_path, "/path/to/file2.txt");
}

#[test]
fn test_file_cache_global_mode_isolation() {
    let temp_dir1 = tempfile::tempdir().unwrap();
    let temp_dir2 = tempfile::tempdir().unwrap();

    let config1 = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let config2 = CacheConfig {
        enabled: true,
        mode: CacheMode::Global,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache1 = FileCache::new(config1, temp_dir1.path()).unwrap();
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let cache2 = FileCache::new(config2, temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    let entry1 = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "global",
        fingerprint1,
    );
    cache1.set(&entry1).unwrap();

    let entry2 = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file2.txt",
        123457i64,
        "global",
        fingerprint2,
    );
    cache2.set(&entry2).unwrap();

    let retrieved1 = cache1.get("hash1").unwrap();
    assert!(retrieved1.is_some());
    assert_eq!(retrieved1.unwrap().file_path, "/path/to/file1.txt");

    let retrieved2 = cache2.get("hash1").unwrap();
    assert!(retrieved2.is_some());
    assert_eq!(retrieved2.unwrap().file_path, "/path/to/file2.txt");
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
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let cache2 = BinaryCache::new(config2, temp_dir2.path()).unwrap();
    let fingerprint2 = cache2.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry1 = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "global",
        fingerprint1,
    );
    cache1.set(&entry1).unwrap();

    let entry2 = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file2.txt",
        123457i64,
        "global",
        fingerprint2,
    );
    cache2.set(&entry2).unwrap();

    let retrieved1 = cache1.get(&hash1).unwrap();
    assert!(retrieved1.is_some());
    assert_eq!(retrieved1.unwrap().file_path, "/path/to/file1.txt");

    let retrieved2 = cache2.get(&hash1).unwrap();
    assert!(retrieved2.is_some());
    assert_eq!(retrieved2.unwrap().file_path, "/path/to/file2.txt");
}

#[test]
fn test_file_cache_mode_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".cache".to_string(),
        format: "json".to_string(),
    };

    let cache1 = FileCache::new(config.clone(), temp_dir.path()).unwrap();
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let entry = codebase_translate::core::models::CacheEntry::new(
        "hash1",
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint1,
    );
    cache1.set(&entry).unwrap();
    cache1.close().unwrap();

    let cache2 = FileCache::new(config, temp_dir.path()).unwrap();

    let retrieved = cache2.get("hash1").unwrap();
    assert!(retrieved.is_some());
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
    let fingerprint1 = cache1.project_fingerprint().to_string();

    let hash1 = hash_utils::generate_test_hash("file1");
    let entry = codebase_translate::core::models::CacheEntry::new(
        &hash1,
        "/path/to/file1.txt",
        123456i64,
        "local",
        fingerprint1,
    );
    cache1.set(&entry).unwrap();
    cache1.close().unwrap();

    let cache_file = temp_dir.path().join(".cache").join("cache.bin");
    eprintln!("Cache file exists: {}", cache_file.exists());
    eprintln!("Cache file path: {:?}", cache_file);

    let cache2 = BinaryCache::new(config, temp_dir.path()).unwrap();

    let retrieved = cache2.get(&hash1).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_file_cache_custom_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: "custom-cache-dir".to_string(),
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

    let cache_dir = temp_dir.path().join("custom-cache-dir");
    assert!(cache_dir.exists());

    let cache_file = cache_dir.join("hash1.json");
    assert!(cache_file.exists());
}

#[test]
fn test_binary_cache_custom_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: "custom-cache-dir".to_string(),
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

    let cache_dir = temp_dir.path().join("custom-cache-dir");
    assert!(cache_dir.exists());

    let cache_file = cache_dir.join("cache.bin");
    assert!(cache_file.exists());
}

fn get_global_cache_dir() -> PathBuf {
    if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Local"))
                    .unwrap_or_else(|_| PathBuf::from("."))
            })
            .join("translator")
            .join("cache")
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join("Library").join("Caches"))
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("translator")
            .join("cache")
    } else {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(|p| PathBuf::from(p).join(".cache"))
                    .unwrap_or_else(|_| PathBuf::from("/tmp"))
            })
            .join("translator")
            .join("cache")
    }
}
