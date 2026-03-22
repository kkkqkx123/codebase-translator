//! Language detection module
//!
//! This module provides language detection capabilities for determining
//! the language of text content. It supports multiple scripts and languages.

use std::collections::HashSet;

/// Script type for language detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Script {
    /// Unknown script
    #[default]
    Unknown,
    /// Latin script (English, German, French, etc.)
    Latin,
    /// CJK script (Chinese, Japanese, Korean)
    Cjk,
    /// Arabic script
    Arabic,
    /// Hebrew script
    Hebrew,
    /// Greek script
    Greek,
    /// Cyrillic script (Russian, Ukrainian, etc.)
    Cyrillic,
}

impl Script {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Latin => "Latin",
            Self::Cjk => "CJK",
            Self::Arabic => "Arabic",
            Self::Hebrew => "Hebrew",
            Self::Greek => "Greek",
            Self::Cyrillic => "Cyrillic",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Language information
#[derive(Debug, Clone, Default)]
pub struct LanguageInfo {
    /// Detected script
    pub script: Script,
    /// Possible languages (sorted by confidence)
    pub langs: Vec<String>,
    /// Whether text has actual characters (not just symbols)
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

    /// Check if language is in the list
    pub fn contains_lang(&self, lang: &str) -> bool {
        self.langs.iter().any(|l| l.eq_ignore_ascii_case(lang))
    }
}

/// Language detector
pub struct LanguageDetector {
    allowed_langs: Option<HashSet<String>>,
    allowed_scripts: Option<HashSet<Script>>,
}

impl LanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        Self {
            allowed_langs: None,
            allowed_scripts: None,
        }
    }

    /// Create with allowed languages
    pub fn with_allowed_langs(allowed: Vec<String>) -> Self {
        let mut scripts = HashSet::new();
        let mut langs = HashSet::new();

        for lang in &allowed {
            // Check if it's a script name
            match lang.as_str() {
                "Latin" => {
                    scripts.insert(Script::Latin);
                    langs.extend(
                        vec!["EN", "DE", "FR", "ES", "IT", "PT", "NL"]
                            .into_iter()
                            .map(String::from),
                    );
                }
                "CJK" | "Cjk" => {
                    scripts.insert(Script::Cjk);
                    langs.extend(vec!["ZH", "JA", "KO"].into_iter().map(String::from));
                }
                "Arabic" => {
                    scripts.insert(Script::Arabic);
                    langs.insert("AR".to_string());
                }
                "Hebrew" => {
                    scripts.insert(Script::Hebrew);
                    langs.insert("HE".to_string());
                }
                "Greek" => {
                    scripts.insert(Script::Greek);
                    langs.insert("EL".to_string());
                }
                "Cyrillic" => {
                    scripts.insert(Script::Cyrillic);
                    langs.extend(vec!["RU", "UK", "BG"].into_iter().map(String::from));
                }
                _ => {
                    langs.insert(lang.clone());
                    if let Some(script) = lang_to_script(lang) {
                        scripts.insert(script);
                    }
                }
            }
        }

        Self {
            allowed_langs: Some(langs),
            allowed_scripts: Some(scripts),
        }
    }

    /// Detect language of text
    pub fn detect(&self, text: &str) -> LanguageInfo {
        // Check if only symbols
        if is_only_symbols(text) {
            return LanguageInfo::new(Script::Unknown, Vec::new(), false);
        }

        // Detect script and language
        self.detect_with_constraints(text)
    }

    /// Detect with constraints
    fn detect_with_constraints(&self, text: &str) -> LanguageInfo {
        // Check CJK
        if self.should_check_script(Script::Cjk) {
            if let Some(langs) = detect_cjk(text, self.allowed_langs.as_ref()) {
                return LanguageInfo::new(Script::Cjk, langs, true);
            }
        }

        // Check Arabic
        if self.should_check_script(Script::Arabic)
            && contains_arabic(text) {
                return LanguageInfo::new(Script::Arabic, vec!["AR".to_string()], true);
            }

        // Check Hebrew
        if self.should_check_script(Script::Hebrew)
            && contains_hebrew(text) {
                return LanguageInfo::new(Script::Hebrew, vec!["HE".to_string()], true);
            }

        // Check Greek
        if self.should_check_script(Script::Greek)
            && contains_greek(text) {
                return LanguageInfo::new(Script::Greek, vec!["EL".to_string()], true);
            }

        // Check Cyrillic
        if self.should_check_script(Script::Cyrillic)
            && contains_cyrillic(text) {
                let langs = detect_cyrillic_langs(text, self.allowed_langs.as_ref());
                return LanguageInfo::new(Script::Cyrillic, langs, true);
            }

        // Check Latin (default)
        if self.should_check_script(Script::Latin)
            && contains_latin(text) {
                let langs = detect_latin_langs(self.allowed_langs.as_ref());
                return LanguageInfo::new(Script::Latin, langs, true);
            }

        LanguageInfo::new(Script::Unknown, Vec::new(), false)
    }

    /// Check if we should check a script
    fn should_check_script(&self, script: Script) -> bool {
        match &self.allowed_scripts {
            None => true,
            Some(scripts) => scripts.contains(&script),
        }
    }

    /// Check if text should be translated based on source and target languages
    pub fn should_translate(&self, text: &str, source_langs: &[String], target_lang: &str) -> bool {
        let info = self.detect(text);

        // If no actual characters, don't translate
        if !info.has_chars {
            return false;
        }

        // If target language is empty or AUTO, translate by default
        if target_lang.is_empty() || target_lang == "AUTO" {
            return true;
        }

        let target_script = lang_to_script(target_lang).unwrap_or(Script::Unknown);

        // If no source languages configured, translate all non-target scripts
        if source_langs.is_empty() {
            return info.script != target_script || !info.contains_lang(target_lang);
        }

        // Check if matches source language configuration
        for src_lang in source_langs {
            if src_lang == "AUTO" {
                // AUTO means auto-detected non-target languages
                if info.script != target_script {
                    return true;
                }
                if !info.langs.is_empty() && !info.contains_lang(target_lang) {
                    return true;
                }
                continue;
            }

            // Check language code match
            if info.contains_lang(src_lang) {
                return true;
            }

            // Check script match
            if let Some(script) = lang_to_script(src_lang) {
                if script == info.script {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert language code to script
fn lang_to_script(lang: &str) -> Option<Script> {
    match lang.to_uppercase().as_str() {
        "ZH" | "JA" | "KO" => Some(Script::Cjk),
        "AR" => Some(Script::Arabic),
        "HE" => Some(Script::Hebrew),
        "EL" => Some(Script::Greek),
        "RU" | "UK" | "BG" | "SR" | "BE" => Some(Script::Cyrillic),
        "EN" | "DE" | "FR" | "ES" | "IT" | "PT" | "NL" | "SV" | "PL" | "TR" | "CS" | "RO"
        | "HU" | "DA" | "NO" => Some(Script::Latin),
        _ => None,
    }
}

/// Detect CJK languages
fn detect_cjk(text: &str, allowed_langs: Option<&HashSet<String>>) -> Option<Vec<String>> {
    let mut has_han = false;
    let mut has_hiragana = false;
    let mut has_katakana = false;
    let mut has_hangul = false;

    // Check what features we need to detect
    let check_japanese = allowed_langs.map_or(true, |l| l.contains("JA"));
    let check_korean = allowed_langs.map_or(true, |l| l.contains("KO"));
    let check_chinese = allowed_langs.map_or(true, |l| l.contains("ZH"));

    for c in text.chars() {
        if check_chinese && is_cjk_unified(c) {
            has_han = true;
        }
        if check_japanese && is_hiragana(c) {
            has_hiragana = true;
        }
        if check_japanese && is_katakana(c) {
            has_katakana = true;
        }
        if check_korean && is_hangul(c) {
            has_hangul = true;
        }
    }

    let mut langs = Vec::new();

    if has_hiragana || has_katakana {
        langs.push("JA".to_string());
    }
    if has_hangul {
        langs.push("KO".to_string());
    }
    if has_han {
        if !has_hiragana && !has_katakana {
            langs.push("ZH".to_string());
        } else if !langs.contains(&"JA".to_string()) {
            langs.push("JA".to_string());
        }
    }

    if langs.is_empty() {
        None
    } else {
        Some(langs)
    }
}

/// Detect Cyrillic languages
fn detect_cyrillic_langs(text: &str, allowed_langs: Option<&HashSet<String>>) -> Vec<String> {
    let check_ukrainian = allowed_langs.map_or(true, |l| l.contains("UK"));
    let check_belarusian = allowed_langs.map_or(true, |l| l.contains("BE"));
    let check_bulgarian = allowed_langs.map_or(true, |l| l.contains("BG"));
    let check_russian = allowed_langs.map_or(true, |l| l.contains("RU"));

    let mut has_ukrainian = false;
    let mut has_belarusian = false;
    let mut has_bulgarian = false;

    for c in text.chars() {
        if check_ukrainian && is_ukrainian_specific(c) {
            has_ukrainian = true;
        }
        if check_belarusian && is_belarusian_specific(c) {
            has_belarusian = true;
        }
        if check_bulgarian && is_bulgarian_specific(c) {
            has_bulgarian = true;
        }
    }

    let mut langs = Vec::new();
    if has_ukrainian {
        langs.push("UK".to_string());
    }
    if has_belarusian {
        langs.push("BE".to_string());
    }
    if has_bulgarian {
        langs.push("BG".to_string());
    }

    if langs.is_empty() && check_russian {
        langs.push("RU".to_string());
    }

    langs
}

/// Detect Latin languages
fn detect_latin_langs(allowed_langs: Option<&HashSet<String>>) -> Vec<String> {
    let all_latin = vec![
        "EN", "DE", "FR", "ES", "IT", "PT", "NL", "SV", "PL", "TR", "CS", "RO", "HU", "DA", "NO",
    ];

    match allowed_langs {
        None => all_latin.into_iter().map(String::from).collect(),
        Some(allowed) => all_latin
            .into_iter()
            .filter(|l| allowed.contains(*l))
            .map(String::from)
            .collect(),
    }
}

/// Check if character is CJK unified
fn is_cjk_unified(c: char) -> bool {
    unicode_blocks::is_cjk(c)
}

/// Check if character is Hiragana
fn is_hiragana(c: char) -> bool {
    unicode_blocks::is_hiragana(c)
}

/// Check if character is Katakana
fn is_katakana(c: char) -> bool {
    unicode_blocks::is_katakana(c)
}

/// Check if character is Hangul
fn is_hangul(c: char) -> bool {
    unicode_blocks::is_hangul(c)
}

/// Check if text contains Arabic
fn contains_arabic(text: &str) -> bool {
    text.chars().any(unicode_blocks::is_arabic)
}

/// Check if text contains Hebrew
fn contains_hebrew(text: &str) -> bool {
    text.chars().any(unicode_blocks::is_hebrew)
}

/// Check if text contains Greek
fn contains_greek(text: &str) -> bool {
    text.chars().any(unicode_blocks::is_greek)
}

/// Check if text contains Cyrillic
fn contains_cyrillic(text: &str) -> bool {
    text.chars().any(unicode_blocks::is_cyrillic)
}

/// Check if text contains Latin
fn contains_latin(text: &str) -> bool {
    text.chars().any(|c| {
        if should_skip_for_detection(c) {
            return false;
        }
        unicode_blocks::is_latin(c)
    })
}

/// Check if character is Ukrainian-specific
fn is_ukrainian_specific(c: char) -> bool {
    matches!(
        c,
        '\u{0490}' | '\u{0491}' | '\u{0404}' | '\u{0454}' | '\u{0407}' | '\u{0457}'
    )
}

/// Check if character is Belarusian-specific
fn is_belarusian_specific(c: char) -> bool {
    matches!(c, '\u{040E}' | '\u{045E}')
}

/// Check if character is Bulgarian-specific
fn is_bulgarian_specific(c: char) -> bool {
    matches!(c, '\u{042A}' | '\u{044A}')
}

/// Check if character should be skipped for detection
fn should_skip_for_detection(c: char) -> bool {
    // Skip emojis
    if ('\u{1F300}'..='\u{1F9FF}').contains(&c) {
        return true;
    }
    // Skip punctuation
    if c.is_ascii_punctuation() {
        return true;
    }
    // Skip digits
    if c.is_ascii_digit() {
        return true;
    }
    // Skip symbols
    if c.is_ascii() && !c.is_alphabetic() {
        return true;
    }
    false
}

/// Check if text contains only symbols
fn is_only_symbols(text: &str) -> bool {
    text.chars().all(|c| {
        should_skip_for_detection(c)
            || (!unicode_blocks::is_cjk(c)
                && !unicode_blocks::is_hiragana(c)
                && !unicode_blocks::is_katakana(c)
                && !unicode_blocks::is_hangul(c)
                && !unicode_blocks::is_arabic(c)
                && !unicode_blocks::is_hebrew(c)
                && !unicode_blocks::is_greek(c)
                && !unicode_blocks::is_cyrillic(c)
                && !unicode_blocks::is_latin(c))
    })
}

/// Unicode blocks helper module
mod unicode_blocks {
    pub fn is_cjk(c: char) -> bool {
        ('\u{4E00}'..='\u{9FFF}').contains(&c)    // CJK Unified Ideographs
            || ('\u{3400}'..='\u{4DBF}').contains(&c) // CJK Extension A
            || ('\u{20000}'..='\u{2A6DF}').contains(&c) // CJK Extension B
    }

    pub fn is_hiragana(c: char) -> bool {
        ('\u{3040}'..='\u{309F}').contains(&c)
    }

    pub fn is_katakana(c: char) -> bool {
        ('\u{30A0}'..='\u{30FF}').contains(&c)
    }

    pub fn is_hangul(c: char) -> bool {
        ('\u{AC00}'..='\u{D7AF}').contains(&c)    // Hangul Syllables
            || ('\u{1100}'..='\u{11FF}').contains(&c) // Hangul Jamo
    }

    pub fn is_arabic(c: char) -> bool {
        ('\u{0600}'..='\u{06FF}').contains(&c)    // Arabic
            || ('\u{0750}'..='\u{077F}').contains(&c) // Arabic Supplement
    }

    pub fn is_hebrew(c: char) -> bool {
        ('\u{0590}'..='\u{05FF}').contains(&c)
    }

    pub fn is_greek(c: char) -> bool {
        ('\u{0370}'..='\u{03FF}').contains(&c)    // Greek and Coptic
            || ('\u{1F00}'..='\u{1FFF}').contains(&c) // Greek Extended
    }

    pub fn is_cyrillic(c: char) -> bool {
        ('\u{0400}'..='\u{04FF}').contains(&c)    // Cyrillic
            || ('\u{0500}'..='\u{052F}').contains(&c) // Cyrillic Supplement
    }

    pub fn is_latin(c: char) -> bool {
        c.is_ascii_uppercase() || c.is_ascii_lowercase() || ('\u{00C0}'..='\u{024F}').contains(&c)
    }
}

/// Create a default language detector
pub fn default_detector() -> LanguageDetector {
    LanguageDetector::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_display() {
        assert_eq!(Script::Latin.to_string(), "Latin");
        assert_eq!(Script::Cjk.to_string(), "CJK");
        assert_eq!(Script::Arabic.to_string(), "Arabic");
    }

    #[test]
    fn test_language_info() {
        let info = LanguageInfo::new(Script::Latin, vec!["EN".to_string()], true);
        assert_eq!(info.script, Script::Latin);
        assert_eq!(info.langs, vec!["EN"]);
        assert!(info.has_chars);
        assert_eq!(info.primary(), Some("EN"));
        assert!(info.contains_lang("EN"));
        assert!(!info.contains_lang("ZH"));
    }

    #[test]
    fn test_detect_english() {
        let detector = LanguageDetector::new();
        let info = detector.detect("Hello world");
        assert_eq!(info.script, Script::Latin);
        assert!(info.langs.contains(&"EN".to_string()));
        assert!(info.has_chars);
    }

    #[test]
    fn test_detect_chinese() {
        let detector = LanguageDetector::new();
        let info = detector.detect("你好世界");
        assert_eq!(info.script, Script::Cjk);
        assert!(info.langs.contains(&"ZH".to_string()));
        assert!(info.has_chars);
    }

    #[test]
    fn test_detect_japanese() {
        let detector = LanguageDetector::new();
        let info = detector.detect("こんにちは");
        assert_eq!(info.script, Script::Cjk);
        assert!(info.langs.contains(&"JA".to_string()));
        assert!(info.has_chars);
    }

    #[test]
    fn test_detect_korean() {
        let detector = LanguageDetector::new();
        let info = detector.detect("안녕하세요");
        assert_eq!(info.script, Script::Cjk);
        assert!(info.langs.contains(&"KO".to_string()));
        assert!(info.has_chars);
    }

    #[test]
    fn test_detect_mixed_cjk() {
        let detector = LanguageDetector::new();
        // Mixed Japanese (Hiragana + Kanji)
        let info = detector.detect("日本語です");
        assert_eq!(info.script, Script::Cjk);
        assert!(info.langs.contains(&"JA".to_string()));
    }

    #[test]
    fn test_detect_arabic() {
        let detector = LanguageDetector::new();
        let info = detector.detect("مرحبا");
        assert_eq!(info.script, Script::Arabic);
        assert!(info.langs.contains(&"AR".to_string()));
    }

    #[test]
    fn test_detect_cyrillic() {
        let detector = LanguageDetector::new();
        let info = detector.detect("Привет");
        assert_eq!(info.script, Script::Cyrillic);
        assert!(info.langs.contains(&"RU".to_string()));
    }

    #[test]
    fn test_only_symbols() {
        let detector = LanguageDetector::new();
        let info = detector.detect("!!! ??? ...");
        assert_eq!(info.script, Script::Unknown);
        assert!(!info.has_chars);
    }

    #[test]
    fn test_should_translate_basic() {
        let detector = LanguageDetector::new();

        // Empty target lang - should translate
        assert!(detector.should_translate("Hello", &[], ""));

        // AUTO target - should translate
        assert!(detector.should_translate("Hello", &[], "AUTO"));

        // Different script - should translate
        assert!(detector.should_translate("你好", &[], "EN"));

        // Same language - should not translate
        assert!(!detector.should_translate("Hello", &[], "EN"));
    }

    #[test]
    fn test_should_translate_with_source() {
        let detector = LanguageDetector::new();

        // Source matches detected language
        assert!(detector.should_translate("Hello", &["EN".to_string()], "ZH"));

        // Source doesn't match
        assert!(!detector.should_translate("Hello", &["ZH".to_string()], "JA"));

        // AUTO source with different script
        assert!(detector.should_translate("你好", &["AUTO".to_string()], "EN"));
    }

    #[test]
    fn test_allowed_langs() {
        let detector =
            LanguageDetector::with_allowed_langs(vec!["EN".to_string(), "ZH".to_string()]);

        let en_info = detector.detect("Hello");
        assert_eq!(en_info.script, Script::Latin);
        assert!(en_info.langs.contains(&"EN".to_string()));

        let zh_info = detector.detect("你好");
        assert_eq!(zh_info.script, Script::Cjk);
        assert!(zh_info.langs.contains(&"ZH".to_string()));
    }

    #[test]
    fn test_allowed_scripts() {
        let detector = LanguageDetector::with_allowed_langs(vec!["Latin".to_string()]);

        let info = detector.detect("Hello");
        assert_eq!(info.script, Script::Latin);

        // CJK should not be detected
        let cjk_info = detector.detect("你好");
        assert_eq!(cjk_info.script, Script::Unknown);
    }

    #[test]
    fn test_lang_to_script() {
        assert_eq!(lang_to_script("EN"), Some(Script::Latin));
        assert_eq!(lang_to_script("ZH"), Some(Script::Cjk));
        assert_eq!(lang_to_script("JA"), Some(Script::Cjk));
        assert_eq!(lang_to_script("AR"), Some(Script::Arabic));
        assert_eq!(lang_to_script("RU"), Some(Script::Cyrillic));
        assert_eq!(lang_to_script("UNKNOWN"), None);
    }
}
