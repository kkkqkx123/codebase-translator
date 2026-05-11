//! Directory Cache Integration Tests
//!
//! These tests verify that the directory cache system correctly stores
//! and retrieves cache entries only in the execution directory.

use codebase_translate::{
    cache::{binary::BinaryCache, DirectoryCache},
    config::{global::GlobalConfig, project::ProjectConfig},
    core::models::{CacheConfig, CacheEntry, CacheMode},
    utils::hash::calculate_hash,
};
use std::fs;
use std::io::Write;

fn create_test_config() -> CacheConfig {
    CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".translator".to_string(),
        format: "binary".to_string(),
    }
}

/// Test directory cache creation
#[test]
fn test_directory_cache_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();

    assert!(!cache.project_fingerprint().is_empty());

    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);
}

/// Test directory cache set and get in same directory
#[test]
fn test_directory_cache_set_and_get() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        temp_dir
            .path()
            .join("test.txt")
            .to_string_lossy()
            .to_string(),
        123456i64,
        "local",
        fingerprint,
        "test_config_hash",
    );
    entry.mark_as_translated();

    cache.set(&entry).unwrap();

    let cached = cache.get(&file_hash, "test_config_hash").unwrap();

    assert!(
        cached.is_some(),
        "Should retrieve entry from same directory cache"
    );
    let cached_entry = cached.unwrap();
    assert!(cached_entry.is_translated);
    assert_eq!(cached_entry.file_hash, file_hash);
}

/// Test directory cache isolation between directories
#[test]
fn test_directory_cache_isolation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();

    let root_cache = DirectoryCache::new(config.clone(), temp_dir.path()).unwrap();
    let sub_cache = DirectoryCache::new(config, &subdir).unwrap();

    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        root_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();

    root_cache.set(&entry).unwrap();

    let cached_in_root = root_cache.get(&file_hash, "test_config_hash").unwrap();
    assert!(
        cached_in_root.is_some(),
        "Should find cache in root directory"
    );

    let cached_in_sub = sub_cache.get(&file_hash, "test_config_hash").unwrap();
    assert!(
        cached_in_sub.is_none(),
        "Should NOT find root cache in subdirectory (cache isolation)"
    );
}

/// Test directory cache with config hash mismatch
#[test]
fn test_directory_cache_config_hash_mismatch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();
    let fingerprint = cache.project_fingerprint().to_string();

    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        fingerprint,
        "original_config_hash",
    );
    entry.mark_as_translated();

    cache.set(&entry).unwrap();

    let cached = cache.get(&file_hash, "different_config_hash").unwrap();

    assert!(
        cached.is_none(),
        "Should not find cache with config hash mismatch"
    );
}

/// Test directory cache with WorkflowBuilder integration
#[test]
fn test_directory_cache_workflow_builder_integration() {
    let temp_dir = tempfile::tempdir().unwrap();

    let test_file = temp_dir.path().join("test.txt");
    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all("// This is a test file\nHello world".as_bytes())
        .unwrap();
    drop(file);

    let config = create_test_config();
    let cache = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
    let content = fs::read(&test_file).unwrap();
    let file_hash = calculate_hash(&content);
    let metadata = fs::metadata(&test_file).unwrap();
    let modified_time = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut entry = CacheEntry::new(
        &file_hash,
        test_file.to_string_lossy().to_string(),
        modified_time,
        "local",
        cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    cache.set(&entry).unwrap();
    cache.close().unwrap();

    let global_config = GlobalConfig::default();
    let project_config = ProjectConfig {
        cache: config,
        writer: codebase_translate::config::project::WriterConfig {
            preview_only: true,
            ..Default::default()
        },
        ..Default::default()
    };

    use codebase_translate::workflow::WorkflowBuilder;
    let builder = WorkflowBuilder::new(
        global_config,
        project_config,
        temp_dir.path().to_str().unwrap(),
    );

    let components = builder.build().unwrap();

    let stats = components.cache.stats().unwrap();
    assert!(stats.entry_count > 0, "Directory cache should have entries");

    let cached = components
        .cache
        .get(&file_hash, "test_config_hash")
        .unwrap();

    assert!(cached.is_some(), "Should find cache from same directory");
}

/// Test directory cache clear operation
#[test]
fn test_directory_cache_clear() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();

    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    cache.set(&entry).unwrap();

    let cached = cache.get(&file_hash, "test_config_hash").unwrap();
    assert!(cached.is_some());

    cache.clear().unwrap();

    let cached = cache.get(&file_hash, "test_config_hash").unwrap();
    assert!(cached.is_none(), "Entry should be cleared");
}

/// Test directory cache persistence across instances
#[test]
fn test_directory_cache_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let file_hash = calculate_hash(b"test_content");

    {
        let cache = DirectoryCache::new(config.clone(), temp_dir.path()).unwrap();
        let fingerprint = cache.project_fingerprint().to_string();

        let mut entry = CacheEntry::new(
            &file_hash,
            "/path/to/test.txt",
            123456i64,
            "local",
            fingerprint,
            "test_config_hash",
        );
        entry.mark_as_translated();
        cache.set(&entry).unwrap();
        cache.close().unwrap();
    }

    {
        let cache = DirectoryCache::new(config, temp_dir.path()).unwrap();
        let cached = cache.get(&file_hash, "test_config_hash").unwrap();

        assert!(
            cached.is_some(),
            "Should retrieve entry from new cache instance"
        );
        assert!(cached.unwrap().is_translated);
    }
}
