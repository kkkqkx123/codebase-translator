//! Factory module for creating translation components
//!
//! Provides factory functions for creating cache, translator, parser, and writer instances.

use crate::{
    cache::binary::BinaryCache,
    cache::Cache,
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    core::models::{CacheConfig as CoreCacheConfig, CacheMode},
    parser::coordinator::ParserCoordinator,
    parser::tree_sitter::ParserConfig,
    translator::factory::TranslatorConfig,
    translator::service::TranslationService,
    writer::file::{FileWriter, WriterConfig},
};

/// Create cache instance
pub fn create_cache(
    cache_config: &crate::config::project::CacheConfig,
    project_path: &str,
) -> Result<Box<dyn Cache>> {
    let cache_dir = std::path::PathBuf::from(project_path).join(&cache_config.cache_dir);

    // Create a core CacheConfig from project CacheConfig
    let core_cache_config = CoreCacheConfig {
        enabled: cache_config.cache_type != "none",
        mode: CacheMode::Local,
        directory: cache_dir.to_string_lossy().to_string(),
        format: cache_config.cache_type.clone(),
    };

    let cache: Box<dyn Cache> = match cache_config.cache_type.as_str() {
        "file" | "json" => Box::new(BinaryCache::new(core_cache_config, project_path)?),
        "binary" => Box::new(BinaryCache::new(core_cache_config, project_path)?),
        _ => Box::new(BinaryCache::new(core_cache_config, project_path)?),
    };

    Ok(cache)
}

/// Create translator instance
pub fn create_translator(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    let translator_config = TranslatorConfig {
        provider: project_config.translate.provider,
        deeplx: Some(crate::translator::common::DeepLXConfig {
            api_url: global_config.deeplx.api_url.clone(),
            api_key: global_config.deeplx.api_key.clone(),
            proxy_url: global_config.deeplx.proxy_url.clone(),
            max_retries: global_config.deeplx.max_retries as usize,
        }),
        llm: None,
        tencent: None,
    };

    TranslationService::new(translator_config)
}

/// Create parser coordinator
pub fn create_parser(project_config: &ProjectConfig) -> Result<ParserCoordinator> {
    let parser_config = ParserConfig {
        extract_comments: project_config.extraction.comments,
        extract_docstrings: project_config.extraction.doc_strings,
        extract_strings: project_config.extraction.format_strings,
        min_content_length: 2,
        max_content_length: 10000,
        trim_content: true,
    };

    ParserCoordinator::from_project_config(parser_config, project_config)
}

/// Create file writer
pub fn create_writer(project_config: &ProjectConfig, project_path: Option<&str>) -> Result<FileWriter> {
    let writer_config = WriterConfig {
        preview_only: project_config.writer.dry_run,
        backup: project_config.writer.backup,
        backup_dir: project_config
            .writer
            .backup_dir
            .as_ref()
            .map(std::path::PathBuf::from),
        strict_encoding: false,
    };

    writer_config.validate()?;
    
    if let Some(path) = project_path {
        Ok(FileWriter::with_project_path(writer_config, std::path::PathBuf::from(path)))
    } else {
        Ok(FileWriter::new(writer_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
