use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::global::LoggingConfig;
use crate::core::models::CacheConfig;
use crate::translator::ProviderType;
use tracing::debug;

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
    /// Logging settings (optional, overrides global config)
    #[serde(default)]
    pub logging: Option<LoggingConfig>,
}

impl ProjectConfig {
    /// Merge another configuration into this one
    pub fn merge(&mut self, other: ProjectConfig) {
        debug!("Merging project configuration");
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
        if other.filter.max_length > 0 {
            self.filter.max_length = other.filter.max_length;
        }
        self.filter.allow_placeholders = other.filter.allow_placeholders;
        self.filter.detect_code_patterns = other.filter.detect_code_patterns;
        self.filter.force_extract_by_language = other.filter.force_extract_by_language;
        if !other.filter.extract_languages.is_empty() {
            self.filter.extract_languages = other.filter.extract_languages;
        }
        if !other.cache.directory.is_empty() {
            self.cache.directory = other.cache.directory;
        }
        if !other.cache.format.is_empty() {
            self.cache.format = other.cache.format;
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

        debug!("Project configuration merged successfully");
    }

    /// Validate the project configuration
    pub fn validate(&mut self) -> Result<(), String> {
        // Normalize language codes before validation
        self.translate.normalize();

        debug!(
            provider = %self.translate.provider,
            target_lang = %self.translate.target_lang,
            cache_format = %self.cache.format,
            "Validating project configuration"
        );

        if self.translate.target_lang.is_empty() {
            return Err("target language is required".to_string());
        }

        if self.translate.target_lang == "auto" {
            return Err("target language cannot be auto".to_string());
        }

        debug!("Project configuration validated successfully");
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

impl TranslateConfig {
    /// Normalize language codes to lowercase
    ///
    /// This ensures consistent language code format across all translators.
    /// Different translation APIs have different requirements for language code casing:
    /// - Tencent: requires lowercase ("en", "zh")
    /// - DeepLX: accepts uppercase ("EN", "ZH")
    /// - LLM: flexible, uses language names
    pub fn normalize(&mut self) {
        // Normalize target language to lowercase
        self.target_lang = self.target_lang.to_lowercase();

        // Normalize source languages to lowercase
        for lang in &mut self.source_langs {
            *lang = lang.to_lowercase();
        }
    }
}

impl Default for TranslateConfig {
    fn default() -> Self {
        Self {
            source_langs: vec!["auto".to_string()],
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

fn default_false() -> bool {
    false
}

fn default_exclude_patterns() -> Vec<String> {
    vec![".translator/**".to_string(), ".translator.toml".to_string()]
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
    #[serde(default = "default_exclude_patterns")]
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
            patterns: default_exclude_patterns(),
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
    /// Maximum text length
    #[serde(default)]
    pub max_length: usize,
    /// Allow placeholders in text
    #[serde(default)]
    pub allow_placeholders: bool,
    /// Detect code patterns
    #[serde(default = "default_true")]
    pub detect_code_patterns: bool,
    /// Force extract text containing specific language characters
    ///
    /// When enabled, all other filtering rules (patterns, length, placeholders, etc.)
    /// are skipped, and only language characteristics are checked.
    #[serde(default)]
    pub force_extract_by_language: bool,
    /// List of languages to extract
    ///
    /// Only effective when `force_extract_by_language` is true.
    #[serde(default)]
    pub extract_languages: Vec<String>,
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
            max_length: 0,
            allow_placeholders: true,
            detect_code_patterns: true,
            force_extract_by_language: false,
            extract_languages: Vec::new(),
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
    /// Extract string literals
    #[serde(default)]
    pub string_literals: StringLiteralConfig,
    /// Custom regex patterns (simple regex-based extraction)
    #[serde(default)]
    pub custom_patterns: Vec<CustomRegexPattern>,
    /// Advanced state machine patterns
    #[serde(default)]
    pub state_machine_patterns: Vec<StateMachinePattern>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            comments: true,
            doc_strings: true,
            error_messages: true,
            format_strings: true,
            string_literals: StringLiteralConfig::default(),
            custom_patterns: Vec::new(),
            state_machine_patterns: Vec::new(),
        }
    }
}

/// String literal extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteralConfig {
    /// Enable string literal extraction
    #[serde(default)]
    pub enabled: bool,
    /// Categories to enable
    #[serde(default = "default_categories")]
    pub categories: StringLiteralCategories,
}

impl Default for StringLiteralConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            categories: default_categories(),
        }
    }
}

/// Categories of string literals that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteralCategories {
    /// Error handling: panic, Error, throw, etc.
    #[serde(default = "default_true")]
    pub error_handling: bool,
    /// Output/logging: print, console, logging, etc.
    #[serde(default = "default_false")]
    pub output: bool,
    /// Variable assignments matching specific patterns
    #[serde(default = "default_false")]
    pub variables: bool,
    /// Object properties with specific keys
    #[serde(default = "default_false")]
    pub properties: bool,
}

impl Default for StringLiteralCategories {
    fn default() -> Self {
        Self {
            error_handling: true,
            output: false,
            variables: false,
            properties: false,
        }
    }
}

fn default_categories() -> StringLiteralCategories {
    StringLiteralCategories::default()
}

/// Custom regex pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRegexPattern {
    /// Pattern name for identification
    pub name: String,
    /// File extensions this pattern applies to
    /// Empty means applies to all files
    #[serde(default)]
    pub file_extensions: Vec<String>,
    /// Category this pattern belongs to
    #[serde(default)]
    pub category: StringLiteralCategory,
    /// Regex pattern
    pub regex: String,
    /// Capture group index (0 = full match)
    #[serde(default)]
    pub group: usize,
}

/// Extraction rule for state machine patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Default)]
pub enum ExtractionRule {
    /// No extraction, use raw content as is
    #[default]
    None,
    /// Remove surrounding quotes (single or double)
    RemoveQuotes,
    /// Extract using regex pattern
    Regex {
        /// Regex pattern to match
        pattern: String,
        /// Capture group index (0 = full match)
        #[serde(default)]
        group: usize,
    },
    /// Remove comment markers
    RemoveCommentMarkers {
        /// Type of comment: "line", "block", or "doc"
        comment_type: String,
    },
    /// Remove surrounding brackets
    RemoveBrackets {
        /// Type of brackets: "round", "square", or "curly"
        bracket_type: String,
    },
}

/// State machine pattern for complex extraction
/// Allows matching sequences of tokens with conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachinePattern {
    /// Pattern name for identification
    pub name: String,
    /// File extensions this pattern applies to
    /// Empty means applies to all files
    #[serde(default)]
    pub file_extensions: Vec<String>,
    /// Category this pattern belongs to
    #[serde(default)]
    pub category: StringLiteralCategory,
    /// Extraction rule
    #[serde(default)]
    pub extraction_rule: ExtractionRule,
    /// States and their transitions
    pub states: Vec<PatternState>,
    /// Initial state name
    pub initial_state: String,
    /// Accepting state names
    pub accepting_states: Vec<String>,
}

/// A state in the pattern state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternState {
    /// State name
    pub name: String,
    /// Regex to match at this state
    pub regex: String,
    /// Capture group to extract (if any)
    #[serde(default)]
    pub capture_group: Option<usize>,
    /// Transitions to other states
    #[serde(default)]
    pub transitions: Vec<StateTransition>,
    /// Whether this state can be the final state
    #[serde(default)]
    pub is_final: bool,
}

/// Transition between states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Target state name
    pub target: String,
    /// Condition regex (if empty, always matches)
    #[serde(default)]
    pub condition: Option<String>,
}

/// Category for custom patterns
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StringLiteralCategory {
    #[default]
    ErrorHandling,
    Output,
    Variables,
    Properties,
    Other,
}

/// Get default patterns for a category by language
pub fn get_default_patterns_for_language(lang: &str, category: &str) -> Vec<String> {
    match category {
        "error_handling" => match lang {
            "javascript" | "typescript" | "jsx" | "tsx" => vec![
                "Error".to_string(),
                "TypeError".to_string(),
                "ReferenceError".to_string(),
                "SyntaxError".to_string(),
                "RangeError".to_string(),
                "URIError".to_string(),
                "EvalError".to_string(),
                "AggregateError".to_string(),
            ],
            "python" => vec![
                "Exception".to_string(),
                "ValueError".to_string(),
                "TypeError".to_string(),
                "KeyError".to_string(),
                "IndexError".to_string(),
                "RuntimeError".to_string(),
                "AssertionError".to_string(),
                "NotImplementedError".to_string(),
            ],
            "rust" => vec![
                "panic".to_string(),
                "todo".to_string(),
                "unimplemented".to_string(),
                "unreachable".to_string(),
                "assert".to_string(),
                "assert_eq".to_string(),
                "assert_ne".to_string(),
            ],
            "go" => vec![
                "errors.New".to_string(),
                "fmt.Errorf".to_string(),
                "panic".to_string(),
            ],
            "java" => vec![
                "IllegalArgumentException".to_string(),
                "IllegalStateException".to_string(),
                "NullPointerException".to_string(),
                "RuntimeException".to_string(),
                "Exception".to_string(),
            ],
            "csharp" => vec![
                "ArgumentException".to_string(),
                "InvalidOperationException".to_string(),
                "NullReferenceException".to_string(),
                "Exception".to_string(),
            ],
            _ => Vec::new(),
        },
        "output" => match lang {
            "javascript" | "typescript" | "jsx" | "tsx" => vec![
                "console.log".to_string(),
                "console.error".to_string(),
                "console.warn".to_string(),
                "console.info".to_string(),
                "console.debug".to_string(),
                "console.trace".to_string(),
                "console.dir".to_string(),
            ],
            "python" => vec![
                "print".to_string(),
                "logging.debug".to_string(),
                "logging.info".to_string(),
                "logging.warning".to_string(),
                "logging.error".to_string(),
                "logging.critical".to_string(),
                "logging.exception".to_string(),
            ],
            "rust" => vec![
                "println".to_string(),
                "eprintln".to_string(),
                "print".to_string(),
                "eprint".to_string(),
                "dbg".to_string(),
            ],
            "go" => vec![
                "fmt.Println".to_string(),
                "fmt.Printf".to_string(),
                "fmt.Print".to_string(),
                "fmt.Fprintf".to_string(),
                "fmt.Fprintln".to_string(),
                "log.Println".to_string(),
                "log.Printf".to_string(),
                "log.Print".to_string(),
            ],
            "java" => vec![
                "System.out.println".to_string(),
                "System.out.print".to_string(),
                "System.err.println".to_string(),
                "System.err.print".to_string(),
                "Logger.getLogger".to_string(),
            ],
            "csharp" => vec![
                "Console.WriteLine".to_string(),
                "Console.Write".to_string(),
                "Console.Error.WriteLine".to_string(),
                "Console.Error.Write".to_string(),
                "Debug.WriteLine".to_string(),
            ],
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// Default variable name patterns
pub fn default_variable_patterns() -> Vec<String> {
    vec![
        ".*Message$".to_string(),
        ".*Text$".to_string(),
        ".*Title$".to_string(),
        ".*Label$".to_string(),
        ".*Hint$".to_string(),
        ".*Placeholder$".to_string(),
    ]
}

/// Default property name patterns
pub fn default_property_patterns() -> Vec<String> {
    vec![
        "message".to_string(),
        "title".to_string(),
        "description".to_string(),
        "text".to_string(),
        "content".to_string(),
        "label".to_string(),
        "placeholder".to_string(),
        "tooltip".to_string(),
        "hint".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert_eq!(config.translate.target_lang, "en");
        assert_eq!(config.translate.source_langs, vec!["auto"]);
        assert_eq!(config.cache.format, "binary");
        assert_eq!(config.cache.directory, ".translator");
        assert!(config.cache.enabled);
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
            logging: None,
        };

        base.merge(other);

        assert_eq!(base.translate.target_lang, "zh");
        assert_eq!(base.translate.source_langs, vec!["ZH"]);
        assert_eq!(base.include.patterns, vec!["**/*.rs"]);
        assert_eq!(base.exclude.patterns, vec!["vendor/**"]);
    }

    #[test]
    fn test_validate_project_config() {
        let mut config = ProjectConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = ProjectConfig::default();
        invalid_config.translate.target_lang = "".to_string();
        assert!(invalid_config.validate().is_err());

        invalid_config.translate.target_lang = "AUTO".to_string();
        assert!(invalid_config.validate().is_err());

        // Cache directory has default value, so empty string won't happen in practice
        // But if it does, validation should still pass (default will be used)
        invalid_config.translate.target_lang = "en".to_string();
        invalid_config.cache.directory = ".translator".to_string();
        assert!(invalid_config.validate().is_ok());
    }

    #[test]
    fn test_cache_directory_default() {
        // Verify that cache.directory has a default value
        let config = ProjectConfig::default();
        assert_eq!(config.cache.directory, ".translator");
        assert!(!config.cache.directory.is_empty());

        // Validation should pass with default value
        let mut config = ProjectConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_directory_custom() {
        // Verify that custom cache directory works
        let mut config = ProjectConfig::default();
        config.cache.directory = ".custom_cache".to_string();
        assert_eq!(config.cache.directory, ".custom_cache");
        assert!(config.validate().is_ok());
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
        assert_eq!(config.get_source_langs(), vec!["auto"]);

        let mut config = ProjectConfig::default();
        config.translate.source_langs = vec!["zh".to_string(), "en".to_string()];
        assert_eq!(config.get_source_langs(), vec!["zh", "en"]);
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
        let patterns = config.get_exclude_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&".translator/**".to_string()));
        assert!(patterns.contains(&".translator.toml".to_string()));

        let mut config = ProjectConfig::default();
        config.exclude.patterns = vec!["vendor/**".to_string()];
        assert_eq!(config.get_exclude_patterns(), vec!["vendor/**"]);
    }
}
