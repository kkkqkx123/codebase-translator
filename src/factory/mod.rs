//! Factory module for creating translation components
//!
//! Provides factory functions for creating cache, translator, parser, and writer instances.

use tracing::{debug, info};

use crate::{
    cache::binary::BinaryCache,
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
    core::models::CacheConfig,
    parser::coordinator::ParserCoordinator,
    parser::tree_sitter::ParserConfig,
    translator::factory::TranslatorConfig,
    translator::service::TranslationService,
    writer::file::{FileWriter, WriterConfig},
};

/// Create cache instance
pub fn create_cache(cache_config: &CacheConfig, project_path: &str) -> Result<BinaryCache> {
    info!(
        cache_type = %cache_config.mode,
        cache_dir = %project_path,
        "Creating cache instance"
    );
    let cache = BinaryCache::new(cache_config.clone(), project_path)?;
    debug!("Cache instance created successfully");
    Ok(cache)
}

/// Create translator instance
pub fn create_translator(
    global_config: &GlobalConfig,
    project_config: &ProjectConfig,
) -> Result<TranslationService> {
    info!(
        provider = %project_config.translate.provider,
        "Creating translator instance"
    );

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

    let translator = TranslationService::new(translator_config)?;
    debug!("Translator instance created successfully");
    Ok(translator)
}

/// Create parser coordinator
pub fn create_parser(project_config: &ProjectConfig) -> Result<ParserCoordinator> {
    info!(
        extract_comments = project_config.extraction.comments,
        extract_docstrings = project_config.extraction.doc_strings,
        extract_strings = project_config.extraction.format_strings,
        "Creating parser coordinator"
    );

    let parser_config = ParserConfig {
        extract_comments: project_config.extraction.comments,
        extract_docstrings: project_config.extraction.doc_strings,
        extract_strings: project_config.extraction.format_strings,
        min_content_length: 2,
        max_content_length: 10000,
        trim_content: true,
    };

    let parser = ParserCoordinator::from_project_config(parser_config, project_config)?;
    debug!("Parser coordinator created successfully");
    Ok(parser)
}

/// Create file writer
pub fn create_writer(
    project_config: &ProjectConfig,
    project_path: Option<&str>,
) -> Result<FileWriter> {
    info!(
        dry_run = project_config.writer.dry_run,
        backup = project_config.writer.backup,
        "Creating file writer"
    );

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

    let writer = if let Some(path) = project_path {
        FileWriter::with_project_path(writer_config, std::path::PathBuf::from(path))
    } else {
        FileWriter::new(writer_config)
    };

    debug!("File writer created successfully");
    Ok(writer)
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
