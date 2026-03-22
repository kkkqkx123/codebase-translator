//! Full language detector with detailed analysis
//!
//! This detector provides complete text analysis for accurate language
//! identification. It's slower than QuickDetector but can distinguish
//! between languages in the same script (e.g., Japanese vs Chinese).

use super::script::Script;
use std::collections::HashSet;

/// Language information from detection
#[derive(Debug, Clone, Default)]
pub struct LanguageInfo {
    /// Detected script
    pub script: Script,
    /// Possible languages (sorted by confidence)
    pub langs: Vec<String>,
    /// Whether text has actual characters (not just symbols)
    pub has_chars: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl LanguageInfo {
    /// Create new language info
    pub fn new(script: Script, langs: Vec<String>, has_chars: bool) -> Self {
        Self {
            script,
            langs,
            has_chars,
            confidence: 0.0,
        }
    }

    /// Create with confidence
    pub fn with_confidence(
        script: Script,
        langs: Vec<String>,
        has_chars: bool,
        confidence: f64,
    ) -> Self {
        Self {
            script,
            langs,
            has_chars,
            confidence,
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

/// Full language detector
///
/// This detector scans the entire text to provide accurate language
/// identification. It can distinguish between:
/// - Chinese, Japanese, Korean (all CJK script)
/// - Russian, Ukrainian, Bulgarian (all Cyrillic)
/// - And more
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
                    if let Some(script) = Script::from_lang_code(lang) {
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

    /// Create with allowed scripts
    pub fn with_allowed_scripts(allowed: Vec<Script>) -> Self {
        let mut langs = HashSet::new();

        for script in &allowed {
            match script {
                Script::Latin => {
                    langs.extend(
                        vec!["EN", "DE", "FR", "ES", "IT", "PT", "NL"]
                            .into_iter()
                            .map(String::from),
                    );
                }
                Script::Cjk => {
                    langs.extend(vec!["ZH", "JA", "KO"].into_iter().map(String::from));
                }
                Script::Arabic => {
                    langs.insert("AR".to_string());
                }
                Script::Hebrew => {
                    langs.insert("HE".to_string());
                }
                Script::Greek => {
                    langs.insert("EL".to_string());
                }
                Script::Cyrillic => {
                    langs.extend(vec!["RU", "UK", "BG"].into_iter().map(String::from));
                }
                Script::Unknown => {}
            }
        }

        Self {
            allowed_langs: Some(langs),
            allowed_scripts: Some(allowed.into_iter().collect()),
        }
    }

    /// Detect language of text (full analysis)
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
                let confidence = calculate_confidence(text, &langs);
                return LanguageInfo::with_confidence(Script::Cjk, langs, true, confidence);
            }
        }

        // Check Arabic
        if self.should_check_script(Script::Arabic) && contains_arabic(text) {
            return LanguageInfo::with_confidence(
                Script::Arabic,
                vec!["AR".to_string()],
                true,
                1.0,
            );
        }

        // Check Hebrew
        if self.should_check_script(Script::Hebrew) && contains_hebrew(text) {
            return LanguageInfo::with_confidence(
                Script::Hebrew,
                vec!["HE".to_string()],
                true,
                1.0,
            );
        }

        // Check Greek
        if self.should_check_script(Script::Greek) && contains_greek(text) {
            return LanguageInfo::with_confidence(
                Script::Greek,
                vec!["EL".to_string()],
                true,
                1.0,
            );
        }

        // Check Cyrillic
        if self.should_check_script(Script::Cyrillic) && contains_cyrillic(text) {
            let langs = detect_cyrillic_langs(text, self.allowed_langs.as_ref());
            let confidence = calculate_confidence(text, &langs);
            return LanguageInfo::with_confidence(Script::Cyrillic, langs, true, confidence);
        }

        // Check Latin (default)
        if self.should_check_script(Script::Latin) && contains_latin(text) {
            let langs = detect_latin_langs(self.allowed_langs.as_ref());
            let confidence = calculate_confidence(text, &langs);
            return LanguageInfo::with_confidence(Script::Latin, langs, true, confidence);
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

        let target_script = Script::from_lang_code(target_lang).unwrap_or(Script::Unknown);

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
            if let Some(script) = Script::from_lang_code(src_lang) {
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

// Detection helper functions

fn detect_cjk(text: &str, allowed_langs: Option<&HashSet<String>>) -> Option<Vec<String>> {
    let mut has_han = false;
    let mut has_hiragana = false;
    let mut has_katakana = false;
    let mut has_hangul = false;

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

fn detect_latin_langs(allowed_langs: Option<&HashSet<String>>) -> Vec<String> {
    let all_latin = vec![
        "EN", "DE", "FR", "ES", "IT", "PT", "NL", "SV", "PL", "TR", "CS", "RO", "HU", "DA",
        "NO",
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

fn calculate_confidence(text: &str, detected_langs: &[String]) -> f64 {
    if detected_langs.is_empty() {
        return 0.0;
    }

    // Simple confidence based on text length and language specificity
    let char_count = text.chars().filter(|c| !c.is_whitespace()).count();

    if char_count == 0 {
        return 0.0;
    }

    // More characters = higher confidence (up to a point)
    let length_confidence = (char_count as f64 / 100.0).min(1.0);

    // Fewer possible languages = higher confidence
    let specificity = 1.0 / detected_langs.len() as f64;

    (length_confidence * 0.5 + specificity * 0.5).min(1.0)
}

// Character classification

fn is_cjk_unified(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
        || ('\u{3400}'..='\u{4DBF}').contains(&c)
        || ('\u{20000}'..='\u{2A6DF}').contains(&c)
}

fn is_hiragana(c: char) -> bool {
    ('\u{3040}'..='\u{309F}').contains(&c)
}

fn is_katakana(c: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&c)
}

fn is_hangul(c: char) -> bool {
    ('\u{AC00}'..='\u{D7AF}').contains(&c) || ('\u{1100}'..='\u{11FF}').contains(&c)
}

fn contains_arabic(text: &str) -> bool {
    text.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
}

fn contains_hebrew(text: &str) -> bool {
    text.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c))
}

fn contains_greek(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{0370}'..='\u{03FF}').contains(&c) || ('\u{1F00}'..='\u{1FFF}').contains(&c)
    })
}

fn contains_cyrillic(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{0400}'..='\u{04FF}').contains(&c) || ('\u{0500}'..='\u{052F}').contains(&c)
    })
}

fn contains_latin(text: &str) -> bool {
    text.chars().any(|c| {
        c.is_ascii_alphabetic() || ('\u{00C0}'..='\u{024F}').contains(&c)
    })
}

fn is_ukrainian_specific(c: char) -> bool {
    matches!(
        c,
        '\u{0490}' | '\u{0491}' | '\u{0404}' | '\u{0454}' | '\u{0407}' | '\u{0457}'
    )
}

fn is_belarusian_specific(c: char) -> bool {
    matches!(c, '\u{040E}' | '\u{045E}')
}

fn is_bulgarian_specific(c: char) -> bool {
    matches!(c, '\u{042A}' | '\u{044A}')
}

fn should_skip(c: char) -> bool {
    if ('\u{1F300}'..='\u{1F9FF}').contains(&c) {
        return true;
    }
    if c.is_ascii_punctuation() {
        return true;
    }
    if c.is_ascii_digit() {
        return true;
    }
    if c.is_ascii() && !c.is_alphabetic() {
        return true;
    }
    false
}

fn is_only_symbols(text: &str) -> bool {
    text.chars().all(|c| {
        should_skip(c)
            || !('\u{4E00}'..='\u{9FFF}').contains(&c)
                && !('\u{3040}'..='\u{309F}').contains(&c)
                && !('\u{30A0}'..='\u{30FF}').contains(&c)
                && !('\u{AC00}'..='\u{D7AF}').contains(&c)
                && !('\u{0600}'..='\u{06FF}').contains(&c)
                && !('\u{0590}'..='\u{05FF}').contains(&c)
                && !('\u{0370}'..='\u{03FF}').contains(&c)
                && !('\u{0400}'..='\u{04FF}').contains(&c)
                && !c.is_ascii_alphabetic()
                && !('\u{00C0}'..='\u{024F}').contains(&c)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_detect_korean() {
        let detector = LanguageDetector::new();
        let info = detector.detect("안녕하세요");
        assert_eq!(info.script, Script::Cjk);
        assert!(info.langs.contains(&"KO".to_string()));
    }

    #[test]
    fn test_detect_arabic() {
        let detector = LanguageDetector::new();
        let info = detector.detect("مرحبا بالعالم");
        assert_eq!(info.script, Script::Arabic);
        assert!(info.langs.contains(&"AR".to_string()));
    }

    #[test]
    fn test_detect_cyrillic() {
        let detector = LanguageDetector::new();
        let info = detector.detect("Привет мир");
        assert_eq!(info.script, Script::Cyrillic);
        assert!(info.langs.contains(&"RU".to_string()));
    }

    #[test]
    fn test_with_allowed_langs() {
        let detector = LanguageDetector::with_allowed_langs(vec!["EN".to_string(), "ZH".to_string()]);

        let en_info = detector.detect("Hello");
        assert_eq!(en_info.script, Script::Latin);

        let zh_info = detector.detect("你好");
        assert_eq!(zh_info.script, Script::Cjk);
    }

    #[test]
    fn test_should_translate() {
        let detector = LanguageDetector::new();

        // Should translate Chinese to English
        assert!(detector.should_translate("你好", &["ZH".to_string()], "EN"));

        // Should not translate English when target is English
        assert!(!detector.should_translate("Hello", &["ZH".to_string()], "EN"));

        // AUTO mode
        assert!(detector.should_translate("你好", &["AUTO".to_string()], "EN"));
        assert!(!detector.should_translate("Hello", &["AUTO".to_string()], "EN"));
    }

    #[test]
    fn test_confidence() {
        let detector = LanguageDetector::new();

        let short = detector.detect("Hi");
        let long = detector.detect("Hello world, this is a longer text with more content");

        assert!(long.confidence > short.confidence);
    }
}
