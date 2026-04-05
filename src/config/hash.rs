//! Configuration hash calculation
//!
//! This module provides functionality to calculate a hash of configuration
//! settings that affect translation behavior. When these settings change,
//! the cache should be invalidated.

use crate::config::project::ProjectConfig;
use sha2::{Digest, Sha256};

/// Calculate a hash of configuration settings that affect translation behavior
///
/// This hash is stored in cache entries and used to invalidate cache when
/// relevant configuration changes. Only settings that affect what gets
/// translated and how are included in the hash.
///
/// Settings that affect the hash:
/// - Translation: source_langs, target_lang
/// - Include/Exclude: patterns
/// - Filter: exclude_keywords, exclude_patterns, include_patterns, max_length, allow_placeholders, detect_code_patterns
/// - Extraction: comments, doc_strings, string_literals, custom_patterns, state_machine_patterns
///
/// Settings that do NOT affect the hash:
/// - Cache settings (cache.*)
/// - Writer settings (writer.*)
/// - Logging settings (logging.*)
/// - Provider selection (translate.provider)
/// - Batch/concurrency settings (translate.batch_size, translate.concurrency)
/// - Encoding settings (encoding.*)
/// - Gitignore settings (exclude.respect_gitignore, exclude.gitignore_patterns)
pub fn calculate_config_hash(config: &ProjectConfig) -> String {
    let mut hasher = Sha256::new();

    // Translation settings
    hasher.update(b"source_langs:");
    for lang in &config.translate.source_langs {
        hasher.update(lang.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|target_lang:");
    hasher.update(config.translate.target_lang.as_bytes());

    // Include patterns
    hasher.update(b"|include:");
    for pattern in &config.include.patterns {
        hasher.update(pattern.as_bytes());
        hasher.update(b",");
    }

    // Exclude patterns
    hasher.update(b"|exclude:");
    for pattern in &config.exclude.patterns {
        hasher.update(pattern.as_bytes());
        hasher.update(b",");
    }

    // Filter settings
    hasher.update(b"|filter_keywords:");
    for keyword in &config.filter.exclude_keywords {
        hasher.update(keyword.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|filter_exclude_patterns:");
    for pattern in &config.filter.exclude_patterns {
        hasher.update(pattern.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|filter_include_patterns:");
    for pattern in &config.filter.include_patterns {
        hasher.update(pattern.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"|filter_max_length:");
    hasher.update(config.filter.max_length.to_string().as_bytes());
    hasher.update(b"|filter_allow_placeholders:");
    hasher.update(if config.filter.allow_placeholders {
        b"1"
    } else {
        b"0"
    });
    hasher.update(b"|filter_detect_code_patterns:");
    hasher.update(if config.filter.detect_code_patterns {
        b"1"
    } else {
        b"0"
    });

    // Extraction settings
    hasher.update(b"|extraction_comments:");
    hasher.update(if config.extraction.comments {
        b"1"
    } else {
        b"0"
    });
    hasher.update(b"|extraction_doc_strings:");
    hasher.update(if config.extraction.doc_strings {
        b"1"
    } else {
        b"0"
    });

    // String literal extraction settings
    hasher.update(b"|string_literals:");
    hasher.update(if config.extraction.string_literals {
        b"1"
    } else {
        b"0"
    });

    // Custom patterns
    hasher.update(b"|custom_patterns:");
    for pattern in &config.extraction.custom_patterns {
        hasher.update(pattern.name.as_bytes());
        hasher.update(b":");
        hasher.update(pattern.regex.as_bytes());
        hasher.update(b",");
    }

    // State machine patterns
    hasher.update(b"|state_machine_patterns:");
    for pattern in &config.extraction.state_machine_patterns {
        hasher.update(pattern.name.as_bytes());
        hasher.update(b",");
    }

    let hash = hasher.finalize();
    hex::encode(hash)[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_hash_consistency() {
        let config1 = ProjectConfig::default();
        let config2 = ProjectConfig::default();

        let hash1 = calculate_config_hash(&config1);
        let hash2 = calculate_config_hash(&config2);

        assert_eq!(hash1, hash2, "Same config should produce same hash");
    }

    #[test]
    fn test_config_hash_changes_with_target_lang() {
        let config1 = ProjectConfig::default();
        let mut config2 = ProjectConfig::default();
        config2.translate.target_lang = "zh".to_string();

        let hash1 = calculate_config_hash(&config1);
        let hash2 = calculate_config_hash(&config2);

        assert_ne!(
            hash1, hash2,
            "Different target_lang should produce different hash"
        );
    }

    #[test]
    fn test_config_hash_unchanged_with_cache_settings() {
        let config1 = ProjectConfig::default();
        let mut config2 = ProjectConfig::default();
        config2.cache.enabled = false;
        config2.cache.directory = "different".to_string();

        let hash1 = calculate_config_hash(&config1);
        let hash2 = calculate_config_hash(&config2);

        assert_eq!(hash1, hash2, "Cache settings should not affect hash");
    }

    #[test]
    fn test_config_hash_unchanged_with_writer_settings() {
        let config1 = ProjectConfig::default();
        let mut config2 = ProjectConfig::default();
        config2.writer.preview_only = true;
        config2.writer.backup = false;

        let hash1 = calculate_config_hash(&config1);
        let hash2 = calculate_config_hash(&config2);

        assert_eq!(hash1, hash2, "Writer settings should not affect hash");
    }

    #[test]
    fn test_config_hash_changes_with_extraction_settings() {
        let config1 = ProjectConfig::default();
        let mut config2 = ProjectConfig::default();
        config2.extraction.comments = false;

        let hash1 = calculate_config_hash(&config1);
        let hash2 = calculate_config_hash(&config2);

        assert_ne!(
            hash1, hash2,
            "Different extraction settings should produce different hash"
        );
    }

    #[test]
    fn test_config_hash_changes_with_filter_settings() {
        let config1 = ProjectConfig::default();
        let mut config2 = ProjectConfig::default();
        config2.filter.exclude_keywords.push("TEST".to_string());

        let hash1 = calculate_config_hash(&config1);
        let hash2 = calculate_config_hash(&config2);

        assert_ne!(
            hash1, hash2,
            "Different filter settings should produce different hash"
        );
    }
}
