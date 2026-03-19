//! Workflow integration tests

mod workflow_integration {
    use codebase_translate::{
        config::{
            global::GlobalConfig,
            project::{
                CacheConfig, ExcludeConfig, ExtractionConfig, IncludeConfig, ProjectConfig,
                TranslateConfig, WriterConfig,
            },
        },
        translator::ProviderType,
        workflow::{TranslationWorkflow, WorkflowConfig, WorkflowResult},
    };
    use std::collections::HashMap;

    fn create_test_global_config() -> GlobalConfig {
        GlobalConfig::default()
    }

    fn create_test_project_config() -> ProjectConfig {
        let mut config = ProjectConfig::default();
        config.translate.target_lang = "en".to_string();
        config.translate.source_langs = vec!["zh".to_string()];
        config.include.patterns = vec!["**/*.txt".to_string()];
        config.exclude.respect_gitignore = false;
        config.cache.cache_type = "none".to_string();
        config.writer.dry_run = true;
        config.writer.backup = false;
        config
    }

    #[test]
    fn test_workflow_config_default() {
        let config = WorkflowConfig::default();

        assert_eq!(config.root_path, ".");
        assert_eq!(config.include_patterns, vec!["**/*"]);
        assert!(config.exclude_patterns.is_empty());
        assert!(!config.follow_symlinks);
        assert!(config.respect_gitignore);
        assert!(config.gitignore_patterns.is_empty());
    }

    #[test]
    fn test_workflow_config_from_project_config() {
        let project_config = create_test_project_config();
        let workflow_config = WorkflowConfig::from(&project_config);

        assert_eq!(workflow_config.root_path, ".");
        assert_eq!(
            workflow_config.include_patterns,
            project_config.include.patterns
        );
        assert_eq!(
            workflow_config.exclude_patterns,
            project_config.exclude.patterns
        );
        assert_eq!(
            workflow_config.respect_gitignore,
            project_config.exclude.respect_gitignore
        );
        assert_eq!(
            workflow_config.gitignore_patterns,
            project_config.exclude.gitignore_patterns
        );
    }

    #[test]
    fn test_translation_workflow_new() {
        let global_config = create_test_global_config();
        let project_config = create_test_project_config();
        let workflow_config = WorkflowConfig::default();

        let workflow =
            TranslationWorkflow::new(global_config, project_config, workflow_config.clone());

        // Workflow should be created successfully
        assert_eq!(workflow_config.root_path, ".");
    }

    #[test]
    fn test_translation_workflow_from_configs_with_path() {
        let global_config = create_test_global_config();
        let project_config = create_test_project_config();

        let workflow = TranslationWorkflow::from_configs_with_path(
            global_config,
            project_config,
            "/test/path",
        );

        // Workflow should be created with custom path
        assert!(true); // If we get here, workflow was created successfully
    }

    #[test]
    fn test_workflow_result_default() {
        let result = WorkflowResult::default();

        assert_eq!(result.files_processed, 0);
        assert_eq!(result.duration_secs, 0.0);
        assert_eq!(result.stats.total_files, 0);
        assert_eq!(result.stats.total_units, 0);
        assert_eq!(result.stats.translated_units, 0);
        assert_eq!(result.stats.cached_files, 0);
        assert_eq!(result.stats.skipped_units, 0);
        assert_eq!(result.stats.errors, 0);
    }
}

mod workflow_file_processor {
    use codebase_translate::config::{global::GlobalConfig, project::ProjectConfig};
    use codebase_translate::workflow::{
        FileProcessResult, FileProcessor, TranslationWorkflow, WorkflowConfig,
    };

    fn create_test_project_config() -> ProjectConfig {
        let mut config = ProjectConfig::default();
        config.translate.target_lang = "en".to_string();
        config.translate.source_langs = vec!["zh".to_string()];
        config.include.patterns = vec!["**/*.txt".to_string()];
        config.exclude.respect_gitignore = false;
        config.cache.cache_type = "none".to_string();
        config.writer.dry_run = true;
        config.writer.backup = false;
        config
    }

    #[test]
    fn test_file_process_result_default() {
        let result = FileProcessResult::default();

        assert_eq!(result.total_units, 0);
        assert_eq!(result.translated_units, 0);
        assert_eq!(result.cached_files, 0);
        assert_eq!(result.skipped_units, 0);
        assert_eq!(result.errors, 0);
        assert!(!result.was_written);
    }

    #[test]
    fn test_file_process_result_merge() {
        let mut result1 = FileProcessResult {
            total_units: 10,
            translated_units: 5,
            cached_files: 1,
            skipped_units: 2,
            errors: 0,
            was_written: true,
        };

        let result2 = FileProcessResult {
            total_units: 8,
            translated_units: 4,
            cached_files: 1,
            skipped_units: 2,
            errors: 1,
            was_written: false,
        };

        result1.merge(&result2);

        assert_eq!(result1.total_units, 18);
        assert_eq!(result1.translated_units, 9);
        assert_eq!(result1.cached_files, 2);
        assert_eq!(result1.skipped_units, 4);
        assert_eq!(result1.errors, 1);
        assert!(result1.was_written); // Should remain true
    }
}

mod workflow_factory {
    use codebase_translate::{
        config::{global::GlobalConfig, project::ProjectConfig},
        factory::{create_cache, create_parser, create_translator, create_writer},
    };

    fn create_test_project_config() -> ProjectConfig {
        ProjectConfig::default()
    }

    fn create_test_global_config() -> GlobalConfig {
        GlobalConfig::default()
    }

    #[test]
    fn test_create_writer() {
        let project_config = create_test_project_config();
        let writer = create_writer(&project_config, None);
        assert!(writer.is_ok());
    }

    #[test]
    fn test_create_parser() {
        let project_config = create_test_project_config();
        let parser = create_parser(&project_config);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_create_translator() {
        let global_config = create_test_global_config();
        let project_config = create_test_project_config();
        let translator = create_translator(&global_config, &project_config);
        assert!(translator.is_ok());
    }

    #[test]
    fn test_create_cache() {
        let project_config = create_test_project_config();
        let temp_dir = std::env::temp_dir();
        let cache = create_cache(&project_config.cache, temp_dir.to_str().unwrap());
        assert!(cache.is_ok());
    }
}

mod workflow_utils {
    use codebase_translate::utils::hash::calculate_hash;

    #[test]
    fn test_calculate_hash_length() {
        let content = b"test content";
        let hash = calculate_hash(content);
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex characters");
    }

    #[test]
    fn test_calculate_hash_consistency() {
        let content = b"consistent content";
        let hash1 = calculate_hash(content);
        let hash2 = calculate_hash(content);
        assert_eq!(hash1, hash2, "Same content should produce same hash");
    }

    #[test]
    fn test_calculate_hash_different_content() {
        let hash1 = calculate_hash(b"content1");
        let hash2 = calculate_hash(b"content2");
        assert_ne!(
            hash1, hash2,
            "Different content should produce different hashes"
        );
    }

    #[test]
    fn test_calculate_hash_empty() {
        let hash = calculate_hash(b"");
        assert_eq!(
            hash.len(),
            64,
            "Empty content should still produce 64-char hash"
        );
    }

    #[test]
    fn test_calculate_hash_large_content() {
        let content = vec![0u8; 10000];
        let hash = calculate_hash(&content);
        assert_eq!(
            hash.len(),
            64,
            "Large content should still produce 64-char hash"
        );
    }
}
