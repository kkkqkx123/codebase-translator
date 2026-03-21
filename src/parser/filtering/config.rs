//! Filter configuration module
//!
//! This module provides configuration structures for filters.

use serde::{Deserialize, Serialize};

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Source languages to translate from (e.g., ["zh", "zh-CN"])
    /// If empty, all languages are accepted
    #[serde(default)]
    pub source_langs: Vec<String>,

    /// Target language for translation (e.g., "EN", "ZH")
    /// Used to avoid translating already-translated content
    #[serde(default = "default_target_lang")]
    pub target_lang: String,

    /// Keywords to exclude
    #[serde(default = "default_exclude_keywords")]
    pub exclude_keywords: Vec<String>,

    /// Regex patterns to exclude
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    /// Regex patterns to include (if set, only matching content is included)
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Maximum content length (0 means no limit)
    #[serde(default = "default_max_length")]
    pub max_length: usize,

    /// Allow placeholders (e.g., %s, {})
    #[serde(default)]
    pub allow_placeholders: bool,

    /// Detect and filter code patterns
    #[serde(default = "default_true")]
    pub detect_code_patterns: bool,
}

fn default_exclude_keywords() -> Vec<String> {
    vec![
        "TODO".to_string(),
        "FIXME".to_string(),
        "NOTE".to_string(),
        "XXX".to_string(),
        "HACK".to_string(),
        "Copyright".to_string(),
        "License".to_string(),
        "Author".to_string(),
        "Licensed".to_string(),
    ]
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        r"https?://[^\s]+".to_string(),
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
        r"\[[^\]]+\]\([^)]+\)".to_string(),
        r"!\[[^\]]*\]\([^)]+\)".to_string(),
        r"<[^>]+>".to_string(),
        r"`[^`]+`".to_string(),
    ]
}

fn default_max_length() -> usize {
    100000
}

fn default_target_lang() -> String {
    "EN".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            source_langs: Vec::new(),
            target_lang: default_target_lang(),
            exclude_keywords: default_exclude_keywords(),
            exclude_patterns: default_exclude_patterns(),
            include_patterns: Vec::new(),
            max_length: 10000,
            allow_placeholders: false,
            detect_code_patterns: true,
        }
    }
}
