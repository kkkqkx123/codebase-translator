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
    translator::ProviderType,
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

    let translator_config = match project_config.translate.provider {
        ProviderType::DeepLX => {
            TranslatorConfig {
                provider: ProviderType::DeepLX,
                deeplx: Some(crate::translator::common::DeepLXConfig {
                    api_url: global_config.deeplx.api_url.clone(),
                    api_key: global_config.deeplx.api_key.clone(),
                    proxy_url: global_config.deeplx.proxy_url.clone(),
                    max_retries: global_config.deeplx.max_retries as usize,
                }),
                llm: None,
                tencent: None,
            }
        }
        ProviderType::LLM => {
            if global_config.llm.providers.is_empty() {
                return Err(crate::core::error::TranslateError::Config(
                    "At least one LLM provider is required".to_string(),
                ));
            }

            let provider = &global_config.llm.providers[0];
            let api_key = provider.api_keys.first().cloned().unwrap_or_default();
            let model = if provider.model.is_empty() {
                provider.model_list.first().cloned().unwrap_or_default()
            } else {
                provider.model.clone()
            };

            TranslatorConfig {
                provider: ProviderType::LLM,
                deeplx: None,
                llm: Some(crate::translator::common::LLMConfig {
                    base_url: provider.base_url.clone(),
                    api_key,
                    model,
                    max_tokens: provider.max_tokens as i32,
                    temperature: provider.temperature as f64,
                    top_p: None,
                    proxy_url: provider.proxy_url.clone(),
                    timeout: provider.timeout,
                    max_retries: 3,
                    extra_headers: Some(provider.extra_headers.clone()),
                    extra_params: Some(serde_json::to_value(&provider.extra_params).unwrap_or_default()),
                }),
                tencent: None,
            }
        }
        ProviderType::Tencent => {
            TranslatorConfig {
                provider: ProviderType::Tencent,
                deeplx: None,
                llm: None,
                tencent: Some(crate::translator::common::TencentConfig {
                    secret_id: global_config.tencent.secret_id.clone().unwrap_or_default(),
                    secret_key: global_config.tencent.secret_key.clone().unwrap_or_default(),
                    region: global_config.tencent.region.clone(),
                    project_id: global_config.tencent.project_id as i64,
                    proxy_url: global_config.tencent.proxy_url.clone(),
                    timeout: global_config.tencent.timeout,
                    max_retries: global_config.tencent.max_retries as usize,
                    untranslated_text: global_config.tencent.untranslated_text.clone(),
                    term_repo_id_list: global_config.tencent.term_repo_id_list.clone(),
                    sent_repo_id_list: global_config.tencent.sent_repo_id_list.clone(),
                }),
            }
        }
    };

    let translator_impl = crate::translator::factory::create_translator_from_config(&translator_config)?;

    let batch_options = crate::translator::common::BatchOptions {
        rate_limit: match project_config.translate.provider {
            ProviderType::DeepLX => global_config.deeplx.rate_limit,
            ProviderType::LLM => global_config.limits.rate_limit,
            ProviderType::Tencent => global_config.tencent.rate_limit,
        },
        workers: 5,
        max_retries: match project_config.translate.provider {
            ProviderType::DeepLX => global_config.deeplx.max_retries as usize,
            ProviderType::LLM => 3,
            ProviderType::Tencent => global_config.tencent.max_retries as usize,
        },
        limit_policy: Some(match project_config.translate.provider {
            ProviderType::DeepLX => crate::translator::common::LimitPolicy {
                rate_limit: global_config.deeplx.rate_limit,
                max_char_count: 5000,
                split_max_chars: 4000,
            },
            ProviderType::LLM => crate::translator::common::LimitPolicy {
                rate_limit: global_config.limits.rate_limit,
                max_char_count: global_config.llm.providers
                    .first()
                    .map(|p| p.max_tokens as usize)
                    .unwrap_or(4096),
                split_max_chars: global_config.limits.split_max_chars as usize,
            },
            ProviderType::Tencent => crate::translator::common::LimitPolicy {
                rate_limit: global_config.tencent.rate_limit,
                max_char_count: 6000,
                split_max_chars: 5000,
            },
        }),
    };

    let batch_translator = crate::translator::BatchTranslator::new(
        std::sync::Arc::new(translator_impl),
        batch_options,
    );

    let translator = TranslationService::with_batch_translator(std::sync::Arc::new(batch_translator))?;
    debug!("Translator instance created successfully with batch translator");
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
