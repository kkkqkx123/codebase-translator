//! Hierarchical Cache Integration Tests
//!
//! These tests verify that the hierarchical cache system correctly searches
//! for cache files in parent directories and reuses translations from
//! subdirectories when translating from higher-level directories.

use codebase_translate::{
    cache::{binary::BinaryCache, HierarchicalCache},
    config::{global::GlobalConfig, project::ProjectConfig},
    core::models::{CacheConfig, CacheEntry, CacheMode},
    utils::hash::calculate_hash,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn create_test_config() -> CacheConfig {
    CacheConfig {
        enabled: true,
        mode: CacheMode::Local,
        directory: ".translator".to_string(),
        format: "binary".to_string(),
    }
}

/// Test hierarchical cache creation with root cache only
#[test]
fn test_hierarchical_cache_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();

    assert!(!cache.project_fingerprint().is_empty());

    // Root cache should exist
    let stats = cache.stats().unwrap();
    assert_eq!(stats.entry_count, 0);
}

/// Test loading cache from a subdirectory
#[test]
fn test_hierarchical_cache_load_from_subdirectory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    // Create a subdirectory with cache
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();

    let translator_dir = subdir.join(".translator");
    fs::create_dir_all(&translator_dir).unwrap();

    // Create a cache entry in the subdirectory
    let sub_cache = BinaryCache::new(config.clone(), &subdir).unwrap();
    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        sub_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    sub_cache.set(&entry).unwrap();
    sub_cache.close().unwrap();

    // Create hierarchical cache and load from subdirectory
    let mut hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
    hierarchical_cache
        .load_cache_from_dir(&translator_dir)
        .unwrap();

    // Verify cache can be retrieved from subdirectory
    let cached = hierarchical_cache
        .get(&subdir.join("test.txt"), &file_hash, "test_config_hash")
        .unwrap();

    assert!(
        cached.is_some(),
        "Should retrieve entry from subdirectory cache"
    );
    let cached_entry = cached.unwrap();
    assert!(cached_entry.is_translated);
    assert_eq!(cached_entry.file_hash, file_hash);
}

/// Test hierarchical cache lookup from file's directory
#[test]
fn test_hierarchical_cache_lookup_from_file_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    // Create directory structure: root/subdir/file.txt
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();

    let translator_dir = subdir.join(".translator");
    fs::create_dir_all(&translator_dir).unwrap();

    // Create cache entry in subdirectory
    let sub_cache = BinaryCache::new(config.clone(), &subdir).unwrap();
    let file_hash = calculate_hash(b"file_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        subdir.join("file.txt").to_string_lossy().to_string(),
        123456i64,
        "local",
        sub_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    sub_cache.set(&entry).unwrap();
    sub_cache.close().unwrap();

    // Create hierarchical cache
    let mut hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
    hierarchical_cache
        .load_cache_from_dir(&translator_dir)
        .unwrap();

    // Test lookup from file path
    let file_path = subdir.join("file.txt");
    let cached = hierarchical_cache
        .get(&file_path, &file_hash, "test_config_hash")
        .unwrap();

    assert!(cached.is_some(), "Should find cache from file's directory");
}

/// Test hierarchical cache with multiple subdirectories
#[test]
fn test_hierarchical_cache_multiple_subdirectories() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    // Create multiple subdirectories with caches
    let subdirs = vec!["src", "tests", "docs"];
    let mut file_hashes = Vec::new();

    for subdir_name in &subdirs {
        let subdir = temp_dir.path().join(subdir_name);
        fs::create_dir_all(&subdir).unwrap();

        let translator_dir = subdir.join(".translator");
        fs::create_dir_all(&translator_dir).unwrap();

        // Create cache entry in each subdirectory
        let sub_cache = BinaryCache::new(config.clone(), &subdir).unwrap();
        let file_hash = calculate_hash(format!("content_{}", subdir_name).as_bytes());
        file_hashes.push(file_hash.clone());

        let mut entry = CacheEntry::new(
            &file_hash,
            subdir.join("file.txt").to_string_lossy().to_string(),
            123456i64,
            "local",
            sub_cache.project_fingerprint().to_string(),
            "test_config_hash",
        );
        entry.mark_as_translated();
        sub_cache.set(&entry).unwrap();
        sub_cache.close().unwrap();
    }

    // Create hierarchical cache and load all subdirectories
    let mut hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
    for subdir_name in &subdirs {
        let translator_dir = temp_dir.path().join(subdir_name).join(".translator");
        hierarchical_cache
            .load_cache_from_dir(&translator_dir)
            .unwrap();
    }

    // Verify all subdirectory caches can be accessed
    for (i, subdir_name) in subdirs.iter().enumerate() {
        let subdir = temp_dir.path().join(subdir_name);
        let file_path = subdir.join("file.txt");
        let cached = hierarchical_cache
            .get(&file_path, &file_hashes[i], "test_config_hash")
            .unwrap();

        assert!(
            cached.is_some(),
            "Should find cache from {} subdirectory",
            subdir_name
        );
    }
}

/// Test hierarchical cache priority (closest directory wins)
#[test]
fn test_hierarchical_cache_priority() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    // Create nested directory structure: root/subdir/nested/
    let subdir = temp_dir.path().join("subdir");
    let nested_dir = subdir.join("nested");
    fs::create_dir_all(&nested_dir).unwrap();

    // Create caches at both levels
    let translator_dir_subdir = subdir.join(".translator");
    let translator_dir_nested = nested_dir.join(".translator");
    fs::create_dir_all(&translator_dir_subdir).unwrap();
    fs::create_dir_all(&translator_dir_nested).unwrap();

    // Create cache entry in parent subdirectory
    let parent_cache = BinaryCache::new(config.clone(), &subdir).unwrap();
    let file_hash = calculate_hash(b"test_content");
    let mut parent_entry = CacheEntry::new(
        &file_hash,
        nested_dir.join("file.txt").to_string_lossy().to_string(),
        123456i64,
        "local",
        parent_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    parent_entry.mark_as_translated();
    parent_cache.set(&parent_entry).unwrap();
    parent_cache.close().unwrap();

    // Create cache entry in nested directory with same hash but different content
    let nested_cache = BinaryCache::new(config.clone(), &nested_dir).unwrap();
    let mut nested_entry = CacheEntry::new(
        &file_hash,
        nested_dir.join("file.txt").to_string_lossy().to_string(),
        123456i64,
        "local",
        nested_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    nested_entry.mark_as_translated();
    nested_cache.set(&nested_entry).unwrap();
    nested_cache.close().unwrap();

    // Create hierarchical cache and load both levels
    let mut hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
    hierarchical_cache
        .load_cache_from_dir(&translator_dir_subdir)
        .unwrap();
    hierarchical_cache
        .load_cache_from_dir(&translator_dir_nested)
        .unwrap();

    // Test that nested directory cache is prioritized (closest to file)
    let file_path = nested_dir.join("file.txt");
    let cached = hierarchical_cache
        .get(&file_path, &file_hash, "test_config_hash")
        .unwrap();

    assert!(cached.is_some(), "Should find cache");
    // The nested directory cache should be found first
    assert_eq!(
        cached.unwrap().project_fingerprint,
        nested_cache.project_fingerprint()
    );
}

/// Test hierarchical cache with root cache fallback
#[test]
fn test_hierarchical_cache_root_fallback() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    // Create cache entry in root directory
    let root_cache = BinaryCache::new(config.clone(), temp_dir.path()).unwrap();
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
    root_cache.close().unwrap();

    // Create hierarchical cache (root cache is automatically created)
    let hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();

    // Verify root cache can be accessed
    let file_path = PathBuf::from("/path/to/test.txt");
    let cached = hierarchical_cache
        .get(&file_path, &file_hash, "test_config_hash")
        .unwrap();

    assert!(cached.is_some(), "Should find cache from root directory");
}

/// Test hierarchical cache with config hash mismatch
#[test]
fn test_hierarchical_cache_config_hash_mismatch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();

    let translator_dir = subdir.join(".translator");
    fs::create_dir_all(&translator_dir).unwrap();

    // Create cache entry with specific config hash
    let sub_cache = BinaryCache::new(config.clone(), &subdir).unwrap();
    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        sub_cache.project_fingerprint().to_string(),
        "original_config_hash",
    );
    entry.mark_as_translated();
    sub_cache.set(&entry).unwrap();
    sub_cache.close().unwrap();

    // Create hierarchical cache
    let mut hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
    hierarchical_cache
        .load_cache_from_dir(&translator_dir)
        .unwrap();

    // Test with different config hash - should return None
    let cached = hierarchical_cache
        .get(
            &subdir.join("test.txt"),
            &file_hash,
            "different_config_hash",
        )
        .unwrap();

    assert!(
        cached.is_none(),
        "Should not find cache with config hash mismatch"
    );
}

/// Test hierarchical cache with WorkflowBuilder integration
#[test]
fn test_hierarchical_cache_workflow_builder_integration() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create subdirectory with cache
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();

    let translator_dir = subdir.join(".translator");
    fs::create_dir_all(&translator_dir).unwrap();

    // Create a test file in subdirectory
    let test_file = subdir.join("test.txt");
    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all("// 这是一个测试文件\nHello world".as_bytes())
        .unwrap();
    drop(file);

    // Create cache entry in subdirectory
    let config = create_test_config();
    let sub_cache = BinaryCache::new(config.clone(), &subdir).unwrap();
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
        sub_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    sub_cache.set(&entry).unwrap();
    sub_cache.close().unwrap();

    // Create workflow components using WorkflowBuilder
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

    // Verify hierarchical cache was created and subdirectory cache was loaded
    let stats = components.cache.stats().unwrap();
    assert!(
        stats.entry_count > 0,
        "Hierarchical cache should have entries"
    );

    // Test that cache can be retrieved
    let cached = components
        .cache
        .get(&test_file, &file_hash, "test_config_hash")
        .unwrap();

    assert!(cached.is_some(), "Should find cache from subdirectory");
}

/// Test hierarchical cache with deep directory structure
#[test]
fn test_hierarchical_cache_deep_directory_structure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    // Create deep directory structure: root/a/b/c/d/
    let deep_dir = temp_dir.path().join("a").join("b").join("c").join("d");
    fs::create_dir_all(&deep_dir).unwrap();

    let translator_dir = deep_dir.join(".translator");
    fs::create_dir_all(&translator_dir).unwrap();

    // Create cache entry at deep level
    let deep_cache = BinaryCache::new(config.clone(), &deep_dir).unwrap();
    let file_hash = calculate_hash(b"deep_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        deep_dir.join("file.txt").to_string_lossy().to_string(),
        123456i64,
        "local",
        deep_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    deep_cache.set(&entry).unwrap();
    deep_cache.close().unwrap();

    // Create hierarchical cache
    let mut hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();
    hierarchical_cache
        .load_cache_from_dir(&translator_dir)
        .unwrap();

    // Test lookup from deep directory
    let file_path = deep_dir.join("file.txt");
    let cached = hierarchical_cache
        .get(&file_path, &file_hash, "test_config_hash")
        .unwrap();

    assert!(
        cached.is_some(),
        "Should find cache from deep directory structure"
    );
}

/// Test hierarchical cache with non-existent cache directory
#[test]
fn test_hierarchical_cache_nonexistent_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let mut cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();

    let nonexistent_dir = temp_dir.path().join("nonexistent").join(".translator");

    // Should not fail, just not load anything
    let result = cache.load_cache_from_dir(&nonexistent_dir);
    assert!(
        result.is_ok(),
        "Loading nonexistent directory should not fail"
    );
}

/// Test hierarchical cache set operation (writes to root cache)
#[test]
fn test_hierarchical_cache_set_operation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();

    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        hierarchical_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();

    // Set should succeed
    let result = hierarchical_cache.set(&entry);
    assert!(result.is_ok(), "Set operation should succeed");

    // Verify entry is in root cache
    let cached = hierarchical_cache
        .get(
            &PathBuf::from("/path/to/test.txt"),
            &file_hash,
            "test_config_hash",
        )
        .unwrap();

    assert!(cached.is_some(), "Entry should be in root cache after set");
}

/// Test hierarchical cache clear operation
#[test]
fn test_hierarchical_cache_clear() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config();

    let hierarchical_cache = HierarchicalCache::new(config, temp_dir.path()).unwrap();

    let file_hash = calculate_hash(b"test_content");
    let mut entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        hierarchical_cache.project_fingerprint().to_string(),
        "test_config_hash",
    );
    entry.mark_as_translated();
    hierarchical_cache.set(&entry).unwrap();

    // Verify entry exists
    let cached = hierarchical_cache
        .get(
            &PathBuf::from("/path/to/test.txt"),
            &file_hash,
            "test_config_hash",
        )
        .unwrap();
    assert!(cached.is_some());

    // Clear cache
    hierarchical_cache.clear().unwrap();

    // Verify entry is gone
    let cached = hierarchical_cache
        .get(
            &PathBuf::from("/path/to/test.txt"),
            &file_hash,
            "test_config_hash",
        )
        .unwrap();
    assert!(cached.is_none(), "Entry should be cleared");
}
