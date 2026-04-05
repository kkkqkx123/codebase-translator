use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Cache mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CacheMode {
    /// Local mode: cache in target project directory
    #[default]
    Local,
    /// Global mode: cache in global storage directory
    Global,
}

impl std::fmt::Display for CacheMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Global => write!(f, "global"),
        }
    }
}

/// Cache entry info (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryInfo {
    /// File hash
    pub file_hash: String,
    /// File path
    pub file_path: String,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether cache is enabled
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    /// Cache mode: local or global
    #[serde(default)]
    pub mode: CacheMode,
    /// Cache directory (relative to project dir for local mode, subdir name for global mode)
    #[serde(default = "default_cache_dir")]
    pub directory: String,
    /// Cache format: json or binary (default binary)
    #[serde(default = "default_cache_format")]
    pub format: String,
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_dir() -> String {
    ".translator".to_string()
}

fn default_cache_format() -> String {
    "binary".to_string()
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            mode: CacheMode::Local,
            directory: default_cache_dir(),
            format: default_cache_format(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries
    pub entry_count: usize,
    /// Total size in bytes
    pub total_size: u64,
}

/// Represents a file to be processed
#[derive(Debug, Clone)]
pub struct File {
    /// Absolute path to the file
    pub path: PathBuf,
    /// File content as bytes
    pub content: Vec<u8>,
    /// Detected or specified encoding
    pub encoding: String,
}

impl File {
    /// Create a new File instance
    pub fn new(path: PathBuf, content: Vec<u8>, encoding: impl Into<String>) -> Self {
        Self {
            path,
            content,
            encoding: encoding.into(),
        }
    }

    /// Get content as string (UTF-8)
    pub fn content_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.content.clone())
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.path.extension()?.to_str()
    }
}

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset from start of file
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

/// Type of node that can be translated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Regular comment
    Comment,
    /// Documentation string
    DocString,
    /// Error message string
    ErrorMessage,
    /// Format string
    FormatString,
    /// Log message
    LogMessage,
    /// String literal
    StringLiteral,
    /// Test description (it, describe, test, etc.)
    TestDescription,
}

impl NodeType {
    /// Get the priority for translation ordering
    pub fn priority(&self) -> u8 {
        match self {
            NodeType::DocString => 1,
            NodeType::Comment => 2,
            NodeType::ErrorMessage => 3,
            NodeType::LogMessage => 4,
            NodeType::FormatString => 5,
            NodeType::StringLiteral => 6,
            NodeType::TestDescription => 7,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Comment => write!(f, "comment"),
            NodeType::DocString => write!(f, "doc_string"),
            NodeType::ErrorMessage => write!(f, "error_message"),
            NodeType::FormatString => write!(f, "format_string"),
            NodeType::LogMessage => write!(f, "log_message"),
            NodeType::StringLiteral => write!(f, "string_literal"),
            NodeType::TestDescription => write!(f, "test_description"),
        }
    }
}

/// Pattern type classification for extraction rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PatternType {
    #[default]
    Builtin,
    CustomRegex,
    StateMachine,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::Builtin => write!(f, "Builtin"),
            PatternType::CustomRegex => write!(f, "CustomRegex"),
            PatternType::StateMachine => write!(f, "StateMachine"),
        }
    }
}

/// A unit of text that can be translated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnit {
    /// Unique identifier
    pub id: String,
    /// Type of node
    pub node_type: NodeType,
    /// Original text content (cleaned for translation)
    pub content: String,
    /// Start position in source
    pub start_pos: Position,
    /// End position in source
    pub end_pos: Position,
    /// Detected language (if any)
    pub language: Option<String>,
    /// Whether this unit should be translated
    pub should_translate: bool,
    /// Translated content (filled after translation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated: Option<String>,
    /// Pattern type (if extracted by custom pattern)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_type: Option<PatternType>,
    /// Pattern name (if extracted by custom pattern)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_name: Option<String>,
    /// Raw match content (for regex-based extraction)
    /// Contains complete matched text including any prefixes/suffixes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_match: Option<String>,
}

impl TranslationUnit {
    /// Create a new translation unit
    pub fn new(
        id: impl Into<String>,
        node_type: NodeType,
        content: impl Into<String>,
        start_pos: Position,
        end_pos: Position,
    ) -> Self {
        let id_str = id.into();
        let content_str = content.into();

        debug!(
            id = %id_str,
            node_type = %node_type,
            content_len = content_str.len(),
            start_line = start_pos.line,
            end_line = end_pos.line,
            "Creating translation unit"
        );

        Self {
            id: id_str,
            node_type,
            content: content_str,
            start_pos,
            end_pos,
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: None,
            pattern_name: None,
            raw_match: None,
        }
    }

    /// Create a new translation unit with pattern info
    pub fn new_with_pattern(
        id: impl Into<String>,
        node_type: NodeType,
        content: impl Into<String>,
        start_pos: Position,
        end_pos: Position,
        pattern_type: PatternType,
        pattern_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            node_type,
            content: content.into(),
            start_pos,
            end_pos,
            language: None,
            should_translate: true,
            translated: None,
            pattern_type: Some(pattern_type),
            pattern_name: Some(pattern_name.into()),
            raw_match: None,
        }
    }

    /// Mark as translated
    pub fn set_translated(&mut self, translated: impl Into<String>) {
        self.translated = Some(translated.into());
    }

    /// Get the content to translate (original or placeholder)
    pub fn content_for_translation(&self) -> &str {
        &self.content
    }
}

/// Language detection result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Detected languages (sorted by confidence)
    pub langs: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Whether the text is reliable (not too short/mixed)
    pub is_reliable: bool,
}

impl LanguageInfo {
    /// Create new language info
    pub fn new(langs: Vec<String>, confidence: f64, is_reliable: bool) -> Self {
        Self {
            langs,
            confidence,
            is_reliable,
        }
    }

    /// Get primary language
    pub fn primary(&self) -> Option<&str> {
        self.langs.first().map(|s| s.as_str())
    }
}

/// File entry for scanning
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Absolute path
    pub path: PathBuf,
    /// Path relative to scan root
    pub relative_path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: std::time::SystemTime,
}

/// Translated unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedUnit {
    /// Unit ID
    pub unit_id: String,
    /// Original text
    pub original: String,
    /// Translated text
    pub translated: String,
    /// Source language
    pub source_lang: String,
    /// Target language
    pub target_lang: String,
}

impl TranslatedUnit {
    /// Create a new translated unit
    pub fn new(
        unit_id: impl Into<String>,
        original: impl Into<String>,
        translated: impl Into<String>,
        source_lang: impl Into<String>,
        target_lang: impl Into<String>,
    ) -> Self {
        Self {
            unit_id: unit_id.into(),
            original: original.into(),
            translated: translated.into(),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
        }
    }
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// File hash
    pub file_hash: String,
    /// File path
    pub file_path: String,
    /// Last modified time (Unix timestamp)
    pub last_modified: i64,
    /// Whether the file has been translated
    pub is_translated: bool,
    /// Translation timestamp (when the file was last translated)
    pub translation_timestamp: i64,
    /// Created at
    #[serde(with = "serde_timestamp")]
    pub created_at: std::time::SystemTime,
    /// Cache mode (when this cache was created)
    pub cache_mode: String,
    /// Project directory fingerprint
    pub project_fingerprint: String,
    /// Configuration hash (for cache invalidation when config changes)
    pub config_hash: String,
}

mod serde_timestamp {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        let secs = duration.as_secs() as i64;
        serializer.serialize_i64(secs)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = i64::deserialize(deserializer)?;
        let duration = Duration::from_secs(secs as u64);
        UNIX_EPOCH
            .checked_add(duration)
            .ok_or(serde::de::Error::custom("timestamp overflow"))
    }
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(
        file_hash: impl Into<String>,
        file_path: impl Into<String>,
        last_modified: i64,
        cache_mode: impl Into<String>,
        project_fingerprint: impl Into<String>,
        config_hash: impl Into<String>,
    ) -> Self {
        Self {
            file_hash: file_hash.into(),
            file_path: file_path.into(),
            last_modified,
            is_translated: false,
            translation_timestamp: 0,
            created_at: SystemTime::now(),
            cache_mode: cache_mode.into(),
            project_fingerprint: project_fingerprint.into(),
            config_hash: config_hash.into(),
        }
    }

    /// Mark the file as translated
    pub fn mark_as_translated(&mut self) {
        self.is_translated = true;
        self.translation_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
    }

    /// Check if cache entry is valid (file not modified)
    pub fn is_valid(&self, current_modified_time: i64) -> bool {
        self.last_modified == current_modified_time
    }

    /// Check if cache entry config hash matches current config
    pub fn is_config_valid(&self, current_config_hash: &str) -> bool {
        self.config_hash == current_config_hash
    }
}
