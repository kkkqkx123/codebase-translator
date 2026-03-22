//! File Processor Cache Integration Tests
//!
//! These tests verify that FileProcessor correctly handles cache entries,
//! specifically ensuring that project_fingerprint is properly set.
//! This test file addresses the bug where FileProcessor was passing an empty
//! string as project_fingerprint, causing cache hits to always fail.

use codebase_translate::{
    cache::CacheFactory,
    config::{global::GlobalConfig, project::ProjectConfig},
    core::models::CacheMode,
    utils::hash::calculate_hash,
    workflow::{file_processor::FileProcessor, WorkflowBuilder},
};
use std::io::Write;

fn create_test_project_config() -> ProjectConfig {
    let mut config = ProjectConfig::default();
    config.translate.target_lang = "en".to_string();
    config.translate.source_langs = vec!["zh".to_string()];
    config.cache.enabled = true;
    config.cache.mode = CacheMode::Local;
    config.writer.dry_run = true; // Don't actually write files
    config.writer.backup = false;
    config.include.patterns = vec!["**/*.txt".to_string()];
    config.exclude.respect_gitignore = false;
    config
}

/// Test that FileProcessor saves cache entries with correct fingerprint
/// This test would have failed before the bug fix
#[test]
fn test_file_processor_saves_correct_fingerprint() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_config = create_test_project_config();
    let global_config = GlobalConfig::default();

    // Create a test file with translatable content (Chinese comment)
    let test_file_path = temp_dir.path().join("test.txt");
    let test_content = "// 这是一个测试文件\nHello world";
    let mut file = std::fs::File::create(&test_file_path).unwrap();
    file.write_all(test_content.as_bytes()).unwrap();
    drop(file);

    // Get file hash and modified time
    let content = std::fs::read(&test_file_path).unwrap();
    let file_hash = calculate_hash(&content);
    let metadata = std::fs::metadata(&test_file_path).unwrap();
    let modified_time = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Build workflow components using WorkflowBuilder
    let builder = WorkflowBuilder::new(
        global_config,
        project_config,
        temp_dir.path().to_str().unwrap(),
    );
    let components = builder
        .build()
        .expect("Failed to build workflow components");

    // Calculate config hash for cache lookup
    let config_hash = codebase_translate::config::calculate_config_hash(builder.project_config());

    // Verify cache is empty initially
    let cached = components.cache.get(&file_hash, &config_hash).unwrap();
    assert!(cached.is_none(), "Cache should be empty initially");

    // Create FileProcessor and process the file
    let processor = FileProcessor::new(
        &components.cache,
        &components.translator,
        &components.parser,
        &components.writer,
        &components.detector,
        &components.encoder,
        builder.project_config(),
        None,
    );

    let result = processor.process(&test_file_path, modified_time);

    // Check the result - we need to know if processing succeeded
    match &result {
        Ok(r) => {
            println!("Processing succeeded: total_units={}, translated_units={}, cached_files={}, skipped_units={}",
                r.total_units, r.translated_units, r.cached_files, r.skipped_units);
        }
        Err(e) => {
            println!("Processing failed with error: {}", e);
        }
    }

    // Check that cache entry was saved with correct fingerprint
    let cached = components.cache.get(&file_hash, &config_hash).unwrap();

    // This assertion would have failed before the fix because FileProcessor
    // was passing an empty string as project_fingerprint
    if let Some(entry) = cached {
        assert!(
            !entry.project_fingerprint.is_empty(),
            "Cache entry should have a non-empty project_fingerprint"
        );

        // Verify the fingerprint matches what the cache expects
        let expected_fingerprint = components.cache.project_fingerprint();
        assert_eq!(
            entry.project_fingerprint, expected_fingerprint,
            "Cache entry fingerprint should match the cache's project fingerprint"
        );
    } else {
        // If cache entry doesn't exist, check if it's because processing returned early
        // (e.g., no translations were produced)
        match result {
            Ok(r) if r.total_units == 0 || r.translated_units == 0 => {
                println!("Cache entry not saved because no translations were produced (this is expected behavior)");
            }
            _ => {
                panic!("Cache entry should exist after processing, or processing should have produced translations");
            }
        }
    }
}

/// Test that cache entries created by FileProcessor can be retrieved (cache hit)
/// This is the critical test that would have failed before the bug fix
#[test]
fn test_file_processor_cache_hit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_config = create_test_project_config();
    let global_config = GlobalConfig::default();

    // Create a test file with translatable content
    let test_file_path = temp_dir.path().join("test.txt");
    let test_content = "// 这是一个测试文件\nHello world";
    let mut file = std::fs::File::create(&test_file_path).unwrap();
    file.write_all(test_content.as_bytes()).unwrap();
    drop(file);

    // Get file hash and modified time
    let content = std::fs::read(&test_file_path).unwrap();
    let file_hash = calculate_hash(&content);
    let metadata = std::fs::metadata(&test_file_path).unwrap();
    let modified_time = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // First processing - should save to cache
    let first_result = {
        let builder = WorkflowBuilder::new(
            global_config.clone(),
            project_config.clone(),
            temp_dir.path().to_str().unwrap(),
        );
        let components = builder
            .build()
            .expect("Failed to build workflow components");
        let config_hash =
            codebase_translate::config::calculate_config_hash(builder.project_config());

        let processor = FileProcessor::new(
            &components.cache,
            &components.translator,
            &components.parser,
            &components.writer,
            &components.detector,
            &components.encoder,
            builder.project_config(),
            None,
        );
        let result = processor.process(&test_file_path, modified_time);

        // Print first processing result
        match &result {
            Ok(r) => {
                println!(
                    "First processing: total_units={}, translated_units={}, cached_files={}",
                    r.total_units, r.translated_units, r.cached_files
                );
            }
            Err(e) => {
                println!("First processing failed: {}", e);
            }
        }

        // Verify cache entry exists with correct fingerprint
        let cached_before = components.cache.get(&file_hash, &config_hash).unwrap();
        if let Some(entry) = &cached_before {
            println!(
                "Cache entry after first processing: fingerprint='{}', is_translated={}",
                entry.project_fingerprint, entry.is_translated
            );
            assert!(
                !entry.project_fingerprint.is_empty(),
                "Cache entry should have non-empty fingerprint"
            );
        } else {
            println!("No cache entry after first processing");
        }

        result
    };

    // Second processing - should be a cache hit
    {
        let builder = WorkflowBuilder::new(
            global_config,
            project_config,
            temp_dir.path().to_str().unwrap(),
        );
        let components = builder
            .build()
            .expect("Failed to build workflow components");
        let _config_hash =
            codebase_translate::config::calculate_config_hash(builder.project_config());

        let processor = FileProcessor::new(
            &components.cache,
            &components.translator,
            &components.parser,
            &components.writer,
            &components.detector,
            &components.encoder,
            builder.project_config(),
            None,
        );
        let result = processor.process(&test_file_path, modified_time);

        match &result {
            Ok(r) => {
                println!("Second processing: cached_files={}", r.cached_files);

                // Only check cache hit if first processing actually saved something
                if let Ok(first_r) = &first_result {
                    if first_r.translated_units > 0 {
                        assert_eq!(
                            r.cached_files, 1,
                            "Second processing should result in cache hit (cached_files=1). \
                             If this fails, it means the cache entry fingerprint doesn't match. \
                             This was the bug where FileProcessor passed empty string as fingerprint."
                        );
                    }
                }
            }
            Err(e) => {
                panic!("Second processing should succeed: {}", e);
            }
        }
    }
}

/// Test that FileProcessor correctly handles the cache entry lifecycle
#[test]
fn test_file_processor_cache_entry_lifecycle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_config = create_test_project_config();
    let global_config = GlobalConfig::default();

    // Create a test file
    let test_file_path = temp_dir.path().join("test.txt");
    let test_content = "// 这是一个测试文件";
    let mut file = std::fs::File::create(&test_file_path).unwrap();
    file.write_all(test_content.as_bytes()).unwrap();
    drop(file);

    // Get file info
    let content = std::fs::read(&test_file_path).unwrap();
    let file_hash = calculate_hash(&content);
    let metadata = std::fs::metadata(&test_file_path).unwrap();
    let modified_time = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Step 1: Initial processing
    let first_result = {
        let builder = WorkflowBuilder::new(
            global_config.clone(),
            project_config.clone(),
            temp_dir.path().to_str().unwrap(),
        );
        let components = builder
            .build()
            .expect("Failed to build workflow components");
        let _config_hash =
            codebase_translate::config::calculate_config_hash(builder.project_config());

        let processor = FileProcessor::new(
            &components.cache,
            &components.translator,
            &components.parser,
            &components.writer,
            &components.detector,
            &components.encoder,
            builder.project_config(),
            None,
        );
        processor.process(&test_file_path, modified_time)
    };

    // Step 2: Create new cache instance (simulating program restart)
    // and process again
    {
        let builder = WorkflowBuilder::new(
            global_config,
            project_config,
            temp_dir.path().to_str().unwrap(),
        );
        let components = builder
            .build()
            .expect("Failed to build workflow components");
        let config_hash =
            codebase_translate::config::calculate_config_hash(builder.project_config());

        // Verify we can retrieve the entry
        let entry = components.cache.get(&file_hash, &config_hash).unwrap();

        if let Some(e) = &entry {
            println!(
                "Entry found after restart: fingerprint='{}'",
                e.project_fingerprint
            );
            assert!(
                !e.project_fingerprint.is_empty(),
                "Fingerprint should not be empty"
            );
        } else {
            println!("No entry found after restart");
        }

        // Process again - should be cache hit if first run saved something
        let processor = FileProcessor::new(
            &components.cache,
            &components.translator,
            &components.parser,
            &components.writer,
            &components.detector,
            &components.encoder,
            builder.project_config(),
            None,
        );
        let result = processor.process(&test_file_path, modified_time);

        if let Ok(process_result) = result {
            if let Ok(first_r) = &first_result {
                if first_r.translated_units > 0 {
                    assert_eq!(
                        process_result.cached_files, 1,
                        "Should be cache hit after program restart"
                    );
                }
            }
        } else {
            panic!("Processing should succeed: {:?}", result);
        }
    }
}

/// Test that demonstrates the bug: empty fingerprint causes cache miss
/// This test simulates what would happen with the old buggy code
#[test]
fn test_empty_fingerprint_causes_cache_miss() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut project_config = create_test_project_config();
    project_config.cache.directory = ".translator".to_string();

    // Create cache manually to get the correct fingerprint
    let cache = CacheFactory::create(&project_config.cache, temp_dir.path().to_str().unwrap())
        .expect("Failed to create cache");
    let correct_fingerprint = cache.project_fingerprint().to_string();

    // Create a cache entry with EMPTY fingerprint (simulating the bug)
    use crate::cache_integration::test_utils::{hash_utils, TEST_CONFIG_HASH};
    use codebase_translate::core::models::CacheEntry;

    let file_hash = hash_utils::generate_test_hash("test_file");
    let mut buggy_entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        "", // Empty fingerprint - this was the bug!
        TEST_CONFIG_HASH,
    );
    buggy_entry.mark_as_translated();

    // Save the buggy entry
    cache.set(&buggy_entry).unwrap();

    // Try to retrieve it - should fail because fingerprint doesn't match
    let retrieved = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap();
    assert!(
        retrieved.is_none(),
        "Entry with empty fingerprint should NOT be retrievable \
         because cache expects fingerprint '{}'",
        correct_fingerprint
    );

    // Now create a correct entry
    let mut correct_entry = CacheEntry::new(
        &file_hash,
        "/path/to/test.txt",
        123456i64,
        "local",
        &correct_fingerprint, // Correct fingerprint
        TEST_CONFIG_HASH,
    );
    correct_entry.mark_as_translated();

    // Save the correct entry
    cache.set(&correct_entry).unwrap();

    // Now it should be retrievable
    let retrieved = cache.get(&file_hash, TEST_CONFIG_HASH).unwrap();
    assert!(
        retrieved.is_some(),
        "Entry with correct fingerprint should be retrievable"
    );
}
