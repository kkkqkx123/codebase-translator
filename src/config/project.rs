use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::translator::ProviderType;

/// Project-level configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Translation settings
    #[serde(default)]
    pub translate: TranslateConfig,
    /// Include patterns
    #[serde(default)]
    pub include: IncludeConfig,
    /// Exclude patterns
    #[serde(default)]
    pub exclude: ExcludeConfig,
    /// Filter settings
    #[serde(default)]
    pub filter: FilterConfig,
    /// Cache settings
    #[serde(default)]
    pub cache: CacheConfig,
    /// Writer settings
    #[serde(default)]
    pub writer: WriterConfig,
    /// Encoding detection settings
    #[serde(default)]
    pub encoding: EncodingConfig,
    /// Extraction settings
    #[serde(default)]
    pub extraction: ExtractionConfig,
}

impl ProjectConfig {
    /// Merge another configuration into this one
    pub fn merge(&mut self, other: ProjectConfig) {
        if !other.translate.source_langs.is_empty() {
            self.translate.source_langs = other.translate.source_langs;
        }
        if !other.translate.target_lang.is_empty() {
            self.translate.target_lang = other.translate.target_lang;
        }
        if !other.include.patterns.is_empty() {
            self.include.patterns = other.include.patterns;
        }
        if !other.exclude.patterns.is_empty() {
            self.exclude.patterns = other.exclude.patterns;
        }
        if !other.filter.exclude_keywords.is_empty() {
            self.filter.exclude_keywords = other.filter.exclude_keywords;
        }
        if !other.filter.exclude_patterns.is_empty() {
            self.filter.exclude_patterns = other.filter.exclude_patterns;
        }
        if !other.filter.include_patterns.is_empty() {
            self.filter.include_patterns = other.filter.include_patterns;
        }
        if other.filter.min_length > 0 {
            self.filter.min_length = other.filter.min_length;
        }
        if other.filter.max_length > 0 {
            self.filter.max_length = other.filter.max_length;
        }
        self.filter.allow_placeholders = other.filter.allow_placeholders;
        self.filter.detect_code_patterns = other.filter.detect_code_patterns;
        if !other.cache.cache_dir.is_empty() {
            self.cache.cache_dir = other.cache.cache_dir;
        }
        if !other.cache.cache_type.is_empty() {
            self.cache.cache_type = other.cache.cache_type;
        }
        if other.writer.dry_run {
            self.writer.dry_run = other.writer.dry_run;
        }
        if other.writer.backup {
            self.writer.backup = other.writer.backup;
        }
        if other.writer.backup_dir.is_some() {
            self.writer.backup_dir = other.writer.backup_dir;
        }
        if !other.encoding.detect_encodings.is_empty() {
            self.encoding.detect_encodings = other.encoding.detect_encodings;
        }
        if other.encoding.min_confidence > 0.0 {
            self.encoding.min_confidence = other.encoding.min_confidence;
        }
        self.encoding.convert_to_utf8 = other.encoding.convert_to_utf8;
        self.extraction.comments = other.extraction.comments;
        self.extraction.doc_strings = other.extraction.doc_strings;
        self.extraction.error_messages = other.extraction.error_messages;
        self.extraction.format_strings = other.extraction.format_strings;
        if !other.extraction.custom_patterns.is_empty() {
            self.extraction.custom_patterns = other.extraction.custom_patterns;
        }
    }

    /// Validate the project configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.translate.target_lang.is_empty() {
            return Err("target language is required".to_string());
        }

        if self.translate.target_lang == "AUTO" {
            return Err("target language cannot be AUTO".to_string());
        }

        if self.cache.cache_dir.is_empty() {
            return Err("cache directory is required".to_string());
        }

        Ok(())
    }

    /// Normalize file patterns by trimming whitespace
    pub fn normalize_patterns(&mut self) {
        for pattern in &mut self.include.patterns {
            *pattern = pattern.trim().to_string();
        }
        for pattern in &mut self.exclude.patterns {
            *pattern = pattern.trim().to_string();
        }
    }

    /// Get source languages list
    pub fn get_source_langs(&self) -> Vec<String> {
        if !self.translate.source_langs.is_empty() {
            self.translate.source_langs.clone()
        } else {
            vec!["AUTO".to_string()]
        }
    }

    /// Get include patterns, returning defaults if empty
    pub fn get_include_patterns(&self) -> Vec<String> {
        if !self.include.patterns.is_empty() {
            self.include.patterns.clone()
        } else {
            default_include_patterns()
        }
    }

    /// Get exclude patterns, returning defaults if empty
    pub fn get_exclude_patterns(&self) -> Vec<String> {
        if !self.exclude.patterns.is_empty() {
            self.exclude.patterns.clone()
        } else {
            Vec::new()
        }
    }
}

/// Translation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateConfig {
    /// Source language codes to translate from
    #[serde(default)]
    pub source_langs: Vec<String>,
    /// Target language code to translate to
    pub target_lang: String,
    /// Translation provider to use
    #[serde(default)]
    pub provider: ProviderType,
    /// Language-specific settings
    #[serde(default)]
    pub lang_settings: HashMap<String, LanguageSettings>,
    /// Batch size for translation API calls
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Concurrent translation requests
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
}

impl Default for TranslateConfig {
    fn default() -> Self {
        Self {
            source_langs: vec!["AUTO".to_string()],
            target_lang: "en".to_string(),
            provider: ProviderType::DeepLX,
            lang_settings: HashMap::new(),
            batch_size: default_batch_size(),
            concurrency: default_concurrency(),
        }
    }
}

/// Language-specific settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageSettings {
    /// File extensions for this language
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Whether to translate comments
    #[serde(default = "default_true")]
    pub translate_comments: bool,
    /// Whether to translate doc strings
    #[serde(default = "default_true")]
    pub translate_docstrings: bool,
    /// Whether to translate error messages
    #[serde(default = "default_true")]
    pub translate_errors: bool,
    /// Whether to translate format strings
    #[serde(default = "default_true")]
    pub translate_formats: bool,
    /// Whether to translate log messages
    #[serde(default = "default_true")]
    pub translate_logs: bool,
    /// Comment patterns (for languages with custom comment syntax)
    #[serde(default)]
    pub comment_patterns: Vec<String>,
    /// Doc comment patterns
    #[serde(default)]
    pub doc_patterns: Vec<String>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache type: "file", "binary", "none"
    #[serde(default = "default_cache_type")]
    pub cache_type: String,
    /// Cache directory
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
    /// Cache file name (for file cache)
    #[serde(default = "default_cache_file")]
    pub cache_file: String,
    /// Binary cache file name
    #[serde(default = "default_binary_cache_file")]
    pub binary_cache_file: String,
    /// Max cache age in days (0 = no limit)
    #[serde(default)]
    pub max_age_days: u32,
    /// Max cache size in MB (0 = no limit)
    #[serde(default)]
    pub max_size_mb: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: default_cache_type(),
            cache_dir: default_cache_dir(),
            cache_file: default_cache_file(),
            binary_cache_file: default_binary_cache_file(),
            max_age_days: 0,
            max_size_mb: 0,
        }
    }
}

/// Writer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterConfig {
    /// Create backups before writing
    #[serde(default = "default_true")]
    pub backup: bool,
    /// Backup directory
    #[serde(default)]
    pub backup_dir: Option<String>,
    /// Dry run mode (don't actually write)
    #[serde(default)]
    pub dry_run: bool,
    /// Max concurrent writes
    #[serde(default = "default_max_concurrent_writes")]
    pub max_concurrent_writes: usize,
    /// Preserve original formatting
    #[serde(default = "default_true")]
    pub preserve_formatting: bool,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            backup: true,
            backup_dir: None,
            dry_run: false,
            max_concurrent_writes: default_max_concurrent_writes(),
            preserve_formatting: true,
        }
    }
}

fn default_include_patterns() -> Vec<String> {
    vec![
        "**/*.c".to_string(),
        "**/*.cpp".to_string(),
        "**/*.h".to_string(),
        "**/*.hpp".to_string(),
        "**/*.go".to_string(),
        "**/*.java".to_string(),
        "**/*.js".to_string(),
        "**/*.ts".to_string(),
        "**/*.tsx".to_string(),
        "**/*.jsx".to_string(),
        "**/*.py".to_string(),
        "**/*.rs".to_string(),
    ]
}

fn default_cache_type() -> String {
    "file".to_string()
}

fn default_cache_dir() -> String {
    ".translator".to_string()
}

fn default_cache_file() -> String {
    "translation_cache.json".to_string()
}

fn default_binary_cache_file() -> String {
    "translation_cache.bin".to_string()
}

fn default_batch_size() -> usize {
    50
}

fn default_concurrency() -> usize {
    5
}

fn default_max_concurrent_writes() -> usize {
    10
}

fn default_true() -> bool {
    true
}

/// Include file patterns configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeConfig {
    /// File patterns to include (glob)
    #[serde(default = "default_include_patterns")]
    pub patterns: Vec<String>,
}

impl Default for IncludeConfig {
    fn default() -> Self {
        Self {
            patterns: default_include_patterns(),
        }
    }
}

/// Exclude file patterns configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeConfig {
    /// File patterns to exclude (glob)
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Whether to respect .gitignore files
    #[serde(default = "default_true")]
    pub respect_gitignore: bool,
    /// Additional .gitignore-style patterns (gitignore syntax)
    #[serde(default)]
    pub gitignore_patterns: Vec<String>,
}

impl Default for ExcludeConfig {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            respect_gitignore: true,
            gitignore_patterns: Vec::new(),
        }
    }
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Keywords to exclude
    #[serde(default)]
    pub exclude_keywords: Vec<String>,
    /// Regex patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// Regex patterns to include (higher priority than exclude)
    #[serde(default)]
    pub include_patterns: Vec<String>,
    /// Minimum text length
    #[serde(default)]
    pub min_length: usize,
    /// Maximum text length
    #[serde(default)]
    pub max_length: usize,
    /// Allow placeholders in text
    #[serde(default)]
    pub allow_placeholders: bool,
    /// Detect code patterns
    #[serde(default = "default_true")]
    pub detect_code_patterns: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            exclude_keywords: vec![
                "TODO".to_string(),
                "FIXME".to_string(),
                "NOTE".to_string(),
                "XXX".to_string(),
                "HACK".to_string(),
                "Copyright".to_string(),
                "License".to_string(),
                "Author".to_string(),
                "Licensed".to_string(),
            ],
            exclude_patterns: vec![
                r"https?://[^\s]+".to_string(),
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
            ],
            include_patterns: Vec::new(),
            min_length: 0,
            max_length: 0,
            allow_placeholders: false,
            detect_code_patterns: true,
        }
    }
}

/// Encoding detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    /// Encodings to detect
    #[serde(default = "default_detect_encodings")]
    pub detect_encodings: Vec<String>,
    /// Minimum confidence threshold (0-1)
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
    /// Automatically convert to UTF-8
    #[serde(default = "default_true")]
    pub convert_to_utf8: bool,
}

impl Default for EncodingConfig {
    fn default() -> Self {
        Self {
            detect_encodings: default_detect_encodings(),
            min_confidence: default_min_confidence(),
            convert_to_utf8: true,
        }
    }
}

fn default_detect_encodings() -> Vec<String> {
    vec![
        "UTF-8".to_string(),
        "GBK".to_string(),
        "Big5".to_string(),
        "Shift_JIS".to_string(),
    ]
}

fn default_min_confidence() -> f64 {
    0.7
}

/// Extraction strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Extract comments
    #[serde(default = "default_true")]
    pub comments: bool,
    /// Extract doc strings
    #[serde(default = "default_true")]
    pub doc_strings: bool,
    /// Extract error messages
    #[serde(default = "default_true")]
    pub error_messages: bool,
    /// Extract format strings
    #[serde(default = "default_true")]
    pub format_strings: bool,
    /// Custom regex patterns
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            comments: true,
            doc_strings: true,
            error_messages: true,
            format_strings: true,
            custom_patterns: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert_eq!(config.translate.target_lang, "en");
        assert_eq!(config.translate.source_langs, vec!["AUTO"]);
        assert_eq!(config.cache.cache_type, "file");
        assert_eq!(config.cache.cache_dir, ".translator");
        assert!(!config.writer.dry_run);
        assert!(config.writer.backup);
    }

    #[test]
    fn test_merge_project_config() {
        let mut base = ProjectConfig::default();
        base.translate.target_lang = "en".to_string();
        base.translate.source_langs = vec!["AUTO".to_string()];

        let other = ProjectConfig {
            translate: TranslateConfig {
                source_langs: vec!["ZH".to_string()],
                target_lang: "zh".to_string(),
                provider: ProviderType::LLM,
                lang_settings: std::collections::HashMap::new(),
                batch_size: 100,
                concurrency: 10,
            },
            include: IncludeConfig {
                patterns: vec!["**/*.rs".to_string()],
            },
            exclude: ExcludeConfig {
                patterns: vec!["vendor/**".to_string()],
                respect_gitignore: true,
                gitignore_patterns: Vec::new(),
            },
            filter: FilterConfig::default(),
            cache: CacheConfig::default(),
            writer: WriterConfig::default(),
            encoding: EncodingConfig::default(),
            extraction: ExtractionConfig::default(),
        };

        base.merge(other);

        assert_eq!(base.translate.target_lang, "zh");
        assert_eq!(base.translate.source_langs, vec!["ZH"]);
        assert_eq!(base.include.patterns, vec!["**/*.rs"]);
        assert_eq!(base.exclude.patterns, vec!["vendor/**"]);
    }

    #[test]
    fn test_validate_project_config() {
        let config = ProjectConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = ProjectConfig::default();
        invalid_config.translate.target_lang = "".to_string();
        assert!(invalid_config.validate().is_err());

        invalid_config.translate.target_lang = "AUTO".to_string();
        assert!(invalid_config.validate().is_err());

        invalid_config.translate.target_lang = "en".to_string();
        invalid_config.cache.cache_dir = "".to_string();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_normalize_patterns() {
        let mut config = ProjectConfig::default();
        config.include.patterns = vec!["  **/*.rs  ".to_string(), "  **/*.go  ".to_string()];
        config.exclude.patterns = vec!["  vendor/**  ".to_string()];

        config.normalize_patterns();

        assert_eq!(config.include.patterns, vec!["**/*.rs", "**/*.go"]);
        assert_eq!(config.exclude.patterns, vec!["vendor/**"]);
    }

    #[test]
    fn test_get_source_langs() {
        let config = ProjectConfig::default();
        assert_eq!(config.get_source_langs(), vec!["AUTO"]);

        let mut config = ProjectConfig::default();
        config.translate.source_langs = vec!["ZH".to_string(), "EN".to_string()];
        assert_eq!(config.get_source_langs(), vec!["ZH", "EN"]);
    }

    #[test]
    fn test_get_include_patterns() {
        let config = ProjectConfig::default();
        let patterns = config.get_include_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&"**/*.rs".to_string()));
    }

    #[test]
    fn test_get_exclude_patterns() {
        let config = ProjectConfig::default();
        assert_eq!(config.get_exclude_patterns(), Vec::<String>::new());

        let mut config = ProjectConfig::default();
        config.exclude.patterns = vec!["vendor/**".to_string()];
        assert_eq!(config.get_exclude_patterns(), vec!["vendor/**"]);
    }
}
