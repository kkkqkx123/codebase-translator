//! Content filter module
//!
//! This module provides filtering capabilities for determining which content
//! should be translated. It supports keyword filtering, pattern matching,
//! placeholder detection, and code pattern detection.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Source languages to translate from (e.g., ["zh", "zh-CN"])
    /// If empty, all languages are accepted
    #[serde(default)]
    pub source_langs: Vec<String>,

    /// Keywords to exclude
    #[serde(default = "default_exclude_keywords")]
    pub exclude_keywords: Vec<String>,

    /// Regex patterns to exclude
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,

    /// Regex patterns to include (if set, only matching content is included)
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Minimum content length
    #[serde(default = "default_min_length")]
    pub min_length: usize,

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

fn default_min_length() -> usize {
    0
}

fn default_max_length() -> usize {
    100000
}

fn default_true() -> bool {
    true
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            source_langs: Vec::new(),
            exclude_keywords: default_exclude_keywords(),
            exclude_patterns: default_exclude_patterns(),
            include_patterns: Vec::new(),
            min_length: 0,
            max_length: 10000,
            allow_placeholders: false,
            detect_code_patterns: true,
        }
    }
}

/// Content filter
pub struct ContentFilter {
    config: FilterConfig,
    exclude_keywords_regex: Vec<Regex>,
    exclude_patterns_regex: Vec<Regex>,
    include_patterns_regex: Vec<Regex>,
    placeholder_regex: Vec<Regex>,
    code_pattern_regex: Vec<Regex>,
    #[allow(dead_code)]
    language_detector: Arc<LanguageDetector>,
}

impl ContentFilter {
    /// Create a new content filter
    pub fn new(config: FilterConfig) -> crate::core::error::Result<Self> {
        let language_detector = Arc::new(LanguageDetector::new());
        Self::with_language_detector(config, language_detector)
    }

    /// Create a new content filter with a language detector
    pub fn with_language_detector(
        config: FilterConfig,
        language_detector: Arc<LanguageDetector>,
    ) -> crate::core::error::Result<Self> {
        // Compile exclude keywords as word-boundary regexes
        let exclude_keywords_regex = config
            .exclude_keywords
            .iter()
            .map(|kw| Regex::new(&format!(r"\b{}\b", regex::escape(kw))))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::core::error::TranslateError::Config(format!(
                    "Invalid exclude keyword regex: {}",
                    e
                ))
            })?;

        // Compile exclude patterns
        let exclude_patterns_regex = config
            .exclude_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::core::error::TranslateError::Config(format!(
                    "Invalid exclude pattern regex: {}",
                    e
                ))
            })?;

        // Compile include patterns
        let include_patterns_regex = config
            .include_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                crate::core::error::TranslateError::Config(format!(
                    "Invalid include pattern regex: {}",
                    e
                ))
            })?;

        // Placeholder patterns
        let placeholder_regex = vec![
            Regex::new(r"%[sdvf]").expect("Invalid placeholder regex"),
            Regex::new(r"\$\d{1,2}\b").expect("Invalid placeholder regex"),
            Regex::new(r"\$\{[^}]*\}").expect("Invalid placeholder regex"),
            Regex::new(r"\{[^}]*\}").expect("Invalid placeholder regex"),
        ];

        // Code pattern detection
        let code_pattern_regex = vec![
            Regex::new(r"\w+\.\w+").expect("Invalid code pattern regex"), // Member access
            Regex::new(r"\w+\([^)]*\)").expect("Invalid code pattern regex"), // Function call
            Regex::new(r"\{[^}]*\}").expect("Invalid code pattern regex"), // Braces
            Regex::new(r"\[[^\]]*\]").expect("Invalid code pattern regex"), // Brackets
        ];

        Ok(Self {
            config,
            exclude_keywords_regex,
            exclude_patterns_regex,
            include_patterns_regex,
            placeholder_regex,
            code_pattern_regex,
            language_detector,
        })
    }

    /// Create a default content filter
    pub fn default() -> crate::core::error::Result<Self> {
        Self::new(FilterConfig::default())
    }

    /// Check if content should be translated
    pub fn should_translate(&self, text: &str) -> bool {
        // Layer 1: O(1) operations - fastest checks first
        // Empty check
        if text.is_empty() {
            debug!(text = %text, reason = "empty", "Text filtered");
            return false;
        }

        // Length check
        let len = text.len();
        if len < self.config.min_length {
            debug!(
                text = %text,
                length = len,
                min_length = self.config.min_length,
                reason = "too_short",
                "Text filtered"
            );
            return false;
        }
        if self.config.max_length > 0 && len > self.config.max_length {
            debug!(
                text = %text,
                length = len,
                max_length = self.config.max_length,
                reason = "too_long",
                "Text filtered"
            );
            return false;
        }

        // Layer 2: Quick language detection (O(k) where k=32)
        // Early filter based on source language configuration
        if !self.config.source_langs.is_empty() {
            if !contains_target_language(text, &self.config.source_langs) {
                debug!(
                    text = %text,
                    source_langs = ?self.config.source_langs,
                    reason = "no_target_language",
                    "Text filtered"
                );
                return false;
            }
        }

        // Layer 3: Regex matching (O(n) where n is number of patterns)
        // Compiled regex patterns are relatively fast
        // Exclude keywords check
        for pattern in &self.exclude_keywords_regex {
            if pattern.is_match(text) {
                debug!(
                    text = %text,
                    reason = "excluded_keyword",
                    "Text filtered"
                );
                return false;
            }
        }

        // Exclude patterns check
        for pattern in &self.exclude_patterns_regex {
            if pattern.is_match(text) {
                debug!(
                    text = %text,
                    reason = "excluded_pattern",
                    "Text filtered"
                );
                return false;
            }
        }

        // Include patterns check
        if !self.include_patterns_regex.is_empty() {
            let included = self.include_patterns_regex.iter().any(|p| p.is_match(text));
            if !included {
                debug!(
                    text = %text,
                    reason = "not_in_include_patterns",
                    "Text filtered"
                );
                return false;
            }
        }

        // Placeholder check
        if !self.config.allow_placeholders {
            for pattern in &self.placeholder_regex {
                if pattern.is_match(text) {
                    debug!(
                        text = %text,
                        reason = "contains_placeholder",
                        "Text filtered"
                    );
                    return false;
                }
            }
        }

        // Code pattern check
        if self.config.detect_code_patterns {
            for pattern in &self.code_pattern_regex {
                if pattern.is_match(text) {
                    debug!(
                        text = %text,
                        reason = "contains_code_pattern",
                        "Text filtered"
                    );
                    return false;
                }
            }
        }

        // Layer 4: O(len) operations - most expensive checks last
        // Symbol-only check
        if is_only_symbols(text) {
            debug!(
                text = %text,
                reason = "only_symbols",
                "Text filtered"
            );
            return false;
        }

        debug!(text = %text, "Text passed filter");
        true
    }

    /// Get filter configuration
    pub fn config(&self) -> &FilterConfig {
        &self.config
    }

    /// Check if text contains placeholders
    pub fn contains_placeholder(&self, text: &str) -> bool {
        self.placeholder_regex.iter().any(|p| p.is_match(text))
    }

    /// Check if text contains code patterns
    pub fn contains_code_pattern(&self, text: &str) -> bool {
        self.code_pattern_regex.iter().any(|p| p.is_match(text))
    }
}

impl std::fmt::Debug for ContentFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContentFilter")
            .field("config", &self.config)
            .finish()
    }
}

/// Check if text contains only symbols/whitespace
fn is_only_symbols(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace() || is_punctuation(c))
}

/// Check if character is punctuation
fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// Quick detection of CJK characters in text
/// Only checks the first 32 characters for performance
fn quick_detect_cjk(text: &str) -> bool {
    text.chars().take(32).any(|c| {
        // Check if character is in CJK Unified Ideographs block
        matches!(c,
            // CJK Unified Ideographs (Chinese, Japanese, Korean)
            '\u{4E00}'..='\u{9FFF}' |
            // CJK Unified Ideographs Extension A
            '\u{3400}'..='\u{4DBF}' |
            // CJK Unified Ideographs Extension B
            '\u{20000}'..='\u{2A6DF}' |
            // CJK Unified Ideographs Extension C, D, E, F, G, H
            '\u{2A700}'..='\u{2B81F}' |
            '\u{2B820}'..='\u{2CEAF}' |
            '\u{2CEB0}'..='\u{2EBEF}' |
            '\u{30000}'..='\u{3134F}' |
            '\u{31350}'..='\u{323AF}' |
            // Hiragana (Japanese)
            '\u{3040}'..='\u{309F}' |
            // Katakana (Japanese)
            '\u{30A0}'..='\u{30FF}' |
            // Hangul Syllables (Korean)
            '\u{AC00}'..='\u{D7AF}' |
            // Hangul Jamo (Korean)
            '\u{1100}'..='\u{11FF}' |
            // CJK Compatibility Ideographs
            '\u{F900}'..='\u{FAFF}' |
            // CJK Radicals Supplement
            '\u{2E80}'..='\u{2EFF}' |
            // Kangxi Radicals
            '\u{2F00}'..='\u{2FDF}' |
            // Ideographic Description Characters
            '\u{2FF0}'..='\u{2FFF}' |
            // CJK Symbols and Punctuation
            '\u{3000}'..='\u{303F}' |
            // Halfwidth and Fullwidth Forms (CJK)
            '\u{FF00}'..='\u{FFEF}'
        )
    })
}

/// Check if text contains target language characters based on source_langs config
fn contains_target_language(text: &str, source_langs: &[String]) -> bool {
    if source_langs.is_empty() {
        return true; // No language restriction
    }

    // Check if AUTO mode is enabled
    if source_langs.iter().any(|lang| lang.to_uppercase() == "AUTO") {
        return true; // AUTO mode accepts all languages
    }

    // Check if any of the source languages require CJK characters
    let requires_cjk = source_langs
        .iter()
        .any(|lang| lang == "zh" || lang == "zh-CN" || lang == "zh-TW" || lang == "ja" || lang == "ko");

    if requires_cjk {
        // If Chinese/Japanese/Korean is required, check for CJK characters
        quick_detect_cjk(text)
    } else {
        // For other languages, we accept all text
        // This could be extended with more specific checks
        true
    }
}

/// Script type for language detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Script {
    #[default]
    Unknown,
    Latin,
    Cjk,
    Arabic,
    Hebrew,
    Greek,
    Cyrillic,
}

/// Language information
#[derive(Debug, Clone, Default)]
pub struct LanguageInfo {
    /// Detected script
    pub script: Script,
    /// Possible languages
    pub langs: Vec<String>,
    /// Whether text has actual characters
    pub has_chars: bool,
}

impl LanguageInfo {
    /// Create new language info
    pub fn new(script: Script, langs: Vec<String>, has_chars: bool) -> Self {
        Self {
            script,
            langs,
            has_chars,
        }
    }

    /// Get primary language
    pub fn primary(&self) -> Option<&str> {
        self.langs.first().map(|s| s.as_str())
    }
}

/// Language detector using whatlang crate
#[derive(Clone)]
pub struct LanguageDetector;

impl LanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        Self
    }

    /// Detect language of text
    pub fn detect(&self, text: &str) -> LanguageInfo {
        use whatlang::{detect_script, Script as WhatlangScript};

        let script = detect_script(text).map_or(Script::Unknown, |s| match s {
            WhatlangScript::Latin => Script::Latin,
            WhatlangScript::Cyrillic => Script::Cyrillic,
            WhatlangScript::Arabic => Script::Arabic,
            WhatlangScript::Hebrew => Script::Hebrew,
            WhatlangScript::Greek => Script::Greek,
            WhatlangScript::Mandarin => Script::Cjk,
            _ => Script::Unknown,
        });

        let has_chars = text.chars().any(|c| c.is_alphabetic());

        LanguageInfo {
            script,
            langs: Vec::new(),
            has_chars,
        }
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced content filter with language detection
pub struct EnhancedContentFilter {
    base: ContentFilter,
    language_detector: Arc<LanguageDetector>,
}

impl EnhancedContentFilter {
    /// Create a new enhanced content filter
    pub fn new(
        config: FilterConfig,
        language_detector: Arc<LanguageDetector>,
    ) -> crate::core::error::Result<Self> {
        Ok(Self {
            base: ContentFilter::new(config)?,
            language_detector,
        })
    }

    /// Check if content should be translated with language awareness
    pub fn should_translate_with_lang(&self, text: &str) -> bool {
        // Basic filtering
        if !self.base.should_translate(text) {
            return false;
        }

        // Language-aware code pattern filtering
        let lang_info = self.language_detector.detect(text);

        if self.base.config().detect_code_patterns {
            if self.base.contains_code_pattern(text) {
                // If text contains non-Latin script, it's likely a code example in comments
                if lang_info.script != Script::Latin && lang_info.script != Script::Unknown {
                    return true;
                }
                // Pure Latin with code patterns is likely code
                return false;
            }
        }

        true
    }

    /// Get base filter
    pub fn base(&self) -> &ContentFilter {
        &self.base
    }
}

/// Create a default filter
pub fn default_filter() -> crate::core::error::Result<ContentFilter> {
    ContentFilter::default()
}

/// Create a filter from project config
pub fn from_project_config(
    config: &crate::config::project::FilterConfig,
    translate_config: &crate::config::project::TranslateConfig,
) -> crate::core::error::Result<ContentFilter> {
    let filter_config = FilterConfig {
        source_langs: translate_config.source_langs.clone(),
        exclude_keywords: config.exclude_keywords.clone(),
        exclude_patterns: config.exclude_patterns.clone(),
        include_patterns: config.include_patterns.clone(),
        min_length: config.min_length,
        max_length: if config.max_length == 0 {
            100000
        } else {
            config.max_length
        },
        allow_placeholders: config.allow_placeholders,
        detect_code_patterns: config.detect_code_patterns,
    };
    ContentFilter::new(filter_config)
}

/// Create a filter from project config with translator max length
pub fn from_project_config_with_translator(
    project_config: &crate::config::project::FilterConfig,
    translate_config: &crate::config::project::TranslateConfig,
    translator_max_length: Option<usize>,
) -> crate::core::error::Result<ContentFilter> {
    let max_length = match (project_config.max_length, translator_max_length) {
        (0, None) => 100000,
        (0, Some(translator_max)) => translator_max,
        (project_max, None) => project_max,
        (project_max, Some(translator_max)) => project_max.min(translator_max),
    };

    let filter_config = FilterConfig {
        source_langs: translate_config.source_langs.clone(),
        exclude_keywords: project_config.exclude_keywords.clone(),
        exclude_patterns: project_config.exclude_patterns.clone(),
        include_patterns: project_config.include_patterns.clone(),
        min_length: project_config.min_length,
        max_length,
        allow_placeholders: project_config.allow_placeholders,
        detect_code_patterns: project_config.detect_code_patterns,
    };
    ContentFilter::new(filter_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_config_default() {
        let config = FilterConfig::default();
        assert!(!config.exclude_keywords.is_empty());
        assert!(!config.exclude_patterns.is_empty());
        assert!(config.include_patterns.is_empty());
        assert_eq!(config.min_length, 0);
        assert_eq!(config.max_length, 10000);
        assert!(!config.allow_placeholders);
        assert!(config.detect_code_patterns);
    }

    #[test]
    fn test_content_filter_basic() {
        let config = FilterConfig::default();
        let filter = ContentFilter::new(config).unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate(""));
        assert!(!filter.should_translate("TODO: fix this"));
        assert!(!filter.should_translate("Copyright 2024"));
    }

    #[test]
    fn test_content_filter_length() {
        let config = FilterConfig {
            min_length: 5,
            max_length: 20,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(!filter.should_translate("Hi"));
        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate("This is a very long text that exceeds the limit"));
    }

    #[test]
    fn test_content_filter_placeholders() {
        let config = FilterConfig {
            allow_placeholders: false,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(!filter.should_translate("Hello %s"));
        assert!(!filter.should_translate("Value: {value}"));
        assert!(!filter.should_translate("Number: $1"));
        assert!(!filter.should_translate("Expression: ${var}"));
        assert!(filter.should_translate("Hello world"));
    }

    #[test]
    fn test_content_filter_code_patterns() {
        let config = FilterConfig {
            detect_code_patterns: true,
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(!filter.should_translate("obj.property"));
        assert!(!filter.should_translate("function()"));
        assert!(!filter.should_translate("{ key: value }"));
        assert!(!filter.should_translate("[1, 2, 3]"));
        assert!(filter.should_translate("This is a comment"));
    }

    #[test]
    fn test_content_filter_include_patterns() {
        let config = FilterConfig {
            include_patterns: vec![r"^[A-Z]".to_string()],
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(filter.should_translate("Hello world"));
        assert!(!filter.should_translate("hello world"));
    }

    #[test]
    fn test_content_filter_exclude_patterns() {
        let config = FilterConfig {
            exclude_patterns: vec![r"https?://[^\s]+".to_string()],
            ..Default::default()
        };
        let filter = ContentFilter::new(config).unwrap();

        assert!(!filter.should_translate("Visit https://example.com for more info"));
        assert!(!filter.should_translate("Check http://test.org"));
        assert!(filter.should_translate("This is a regular comment"));
    }

    #[test]
    fn test_content_filter_url_patterns() {
        let filter = ContentFilter::default().unwrap();

        assert!(!filter.should_translate("https://example.com"));
        assert!(!filter.should_translate("http://test.org/path"));
        assert!(!filter.should_translate("user@example.com"));
        assert!(filter.should_translate("Contact us at email"));
    }

    #[test]
    fn test_is_only_symbols() {
        assert!(is_only_symbols("   "));
        assert!(is_only_symbols("!!!"));
        assert!(is_only_symbols("// "));
        assert!(!is_only_symbols("Hello"));
        assert!(!is_only_symbols("Hello world"));
    }

    #[test]
    fn test_contains_placeholder() {
        let filter = ContentFilter::default().unwrap();

        assert!(filter.contains_placeholder("Hello %s"));
        assert!(filter.contains_placeholder("Value: {value}"));
        assert!(!filter.contains_placeholder("Hello world"));
    }

    #[test]
    fn test_contains_code_pattern() {
        let filter = ContentFilter::default().unwrap();

        assert!(filter.contains_code_pattern("obj.property"));
        assert!(filter.contains_code_pattern("function()"));
        assert!(!filter.contains_code_pattern("Hello world"));
    }

    #[test]
    fn test_language_info() {
        let info = LanguageInfo::new(Script::Latin, vec!["EN".to_string()], true);
        assert_eq!(info.script, Script::Latin);
        assert_eq!(info.langs, vec!["EN"]);
        assert!(info.has_chars);
        assert_eq!(info.primary(), Some("EN"));
    }
}
