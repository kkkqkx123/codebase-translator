//! Detection strategies for different performance requirements
//!
//! Provides three detection strategies:
//! - QuickDetector: O(32) - First 32 characters only
//! - SampledDetector: O(n/sample_rate) - Sample at intervals
//! - FullDetector: O(n) - Complete text analysis

use super::script::Script;

/// Detection strategy trait
pub trait DetectionStrategy {
    /// Detect the primary script of the text
    fn detect_script(&self, text: &str) -> Script;

    /// Check if text contains meaningful characters (not just symbols)
    fn has_meaningful_content(&self, text: &str) -> bool;
}

/// Quick detector - only checks first 32 characters
///
/// This is the fastest detection method, suitable for filtering decisions.
/// It checks only the first 32 non-whitespace characters.
pub struct QuickDetector {
    sample_size: usize,
}

impl QuickDetector {
    /// Create a new quick detector with default sample size (32)
    pub fn new() -> Self {
        Self { sample_size: 32 }
    }

    /// Create with custom sample size
    pub fn with_sample_size(size: usize) -> Self {
        Self { sample_size: size }
    }

    /// Quick check for CJK characters (Chinese, Japanese, Korean)
    pub fn has_cjk(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_cjk_char)
    }

    /// Quick check for Chinese characters specifically
    pub fn has_chinese(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_chinese_char)
    }

    /// Quick check for Japanese characters (Hiragana/Katakana)
    pub fn has_japanese(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(|c| is_hiragana(c) || is_katakana(c))
    }

    /// Quick check for Korean characters
    pub fn has_korean(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_hangul)
    }

    /// Quick check for Latin characters
    pub fn is_latin(&self, text: &str) -> bool {
        let sample: String = text
            .chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .collect();

        if sample.is_empty() {
            return false;
        }

        let latin_count = sample.chars().filter(|c| is_latin(*c)).count();
        let total_chars = sample.chars().filter(|c| !should_skip(*c)).count();

        if total_chars == 0 {
            return false;
        }

        // If more than 70% of characters are Latin, consider it Latin
        (latin_count as f64 / total_chars as f64) > 0.7
    }

    /// Quick check for Arabic characters
    pub fn has_arabic(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_arabic)
    }

    /// Quick check for Cyrillic characters
    pub fn has_cyrillic(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_cyrillic)
    }

    /// Quick check for Greek characters
    pub fn has_greek(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_greek)
    }

    /// Quick check for Hebrew characters
    pub fn has_hebrew(&self, text: &str) -> bool {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .any(is_hebrew)
    }

    /// Detect script using quick sampling
    pub fn detect_script(&self, text: &str) -> Script {
        let sample: Vec<char> = text
            .chars()
            .filter(|c| !c.is_whitespace())
            .take(self.sample_size)
            .collect();

        if sample.is_empty() || sample.iter().all(|c| should_skip(*c)) {
            return Script::Unknown;
        }

        // Check in order of specificity
        if sample
            .iter()
            .any(|c| is_hiragana(*c) || is_katakana(*c) || is_hangul(*c))
        {
            return Script::Cjk;
        }

        if sample.iter().any(|c| is_chinese_char(*c)) {
            return Script::Cjk;
        }

        if sample.iter().any(|c| is_arabic(*c)) {
            return Script::Arabic;
        }

        if sample.iter().any(|c| is_hebrew(*c)) {
            return Script::Hebrew;
        }

        if sample.iter().any(|c| is_greek(*c)) {
            return Script::Greek;
        }

        if sample.iter().any(|c| is_cyrillic(*c)) {
            return Script::Cyrillic;
        }

        if sample.iter().any(|c| is_latin(*c)) {
            return Script::Latin;
        }

        Script::Unknown
    }
}

impl Default for QuickDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Sampled detector - samples text at intervals
///
/// For long texts, this avoids scanning the entire content while still
/// providing good accuracy. Useful when you need better accuracy than
/// QuickDetector but can't afford full scanning.
pub struct SampledDetector {
    sample_rate: usize,
    max_samples: usize,
}

impl SampledDetector {
    /// Create with default settings (sample every 100 chars, max 10 samples)
    pub fn new() -> Self {
        Self {
            sample_rate: 100,
            max_samples: 10,
        }
    }

    /// Create with custom settings
    pub fn with_settings(sample_rate: usize, max_samples: usize) -> Self {
        Self {
            sample_rate,
            max_samples,
        }
    }

    /// Detect script by sampling
    pub fn detect_script(&self, text: &str) -> Script {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len == 0 {
            return Script::Unknown;
        }

        // For short texts, use quick detection
        if len <= 32 {
            return QuickDetector::new().detect_script(text);
        }

        // Sample at intervals
        let mut samples_checked = 0;
        let mut i = 0;

        while i < len && samples_checked < self.max_samples {
            let c = chars[i];

            if !c.is_whitespace() && !should_skip(c) {
                if is_hiragana(c) || is_katakana(c) || is_hangul(c) || is_chinese_char(c) {
                    return Script::Cjk;
                }
                if is_arabic(c) {
                    return Script::Arabic;
                }
                if is_hebrew(c) {
                    return Script::Hebrew;
                }
                if is_greek(c) {
                    return Script::Greek;
                }
                if is_cyrillic(c) {
                    return Script::Cyrillic;
                }
                if is_latin(c) {
                    return Script::Latin;
                }
            }

            samples_checked += 1;
            i += self.sample_rate;
        }

        Script::Unknown
    }

    /// Check if text has meaningful content by sampling
    pub fn has_meaningful_content(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len == 0 {
            return false;
        }

        let mut samples_checked = 0;
        let mut i = 0;

        while i < len && samples_checked < self.max_samples {
            let c = chars[i];
            if !should_skip(c) && !c.is_whitespace() {
                return true;
            }
            samples_checked += 1;
            i += self.sample_rate;
        }

        false
    }
}

impl Default for SampledDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Character classification functions

fn is_cjk_char(c: char) -> bool {
    is_chinese_char(c) || is_hiragana(c) || is_katakana(c) || is_hangul(c)
}

fn is_chinese_char(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)    // CJK Unified Ideographs
        || ('\u{3400}'..='\u{4DBF}').contains(&c) // CJK Extension A
        || ('\u{20000}'..='\u{2A6DF}').contains(&c) // CJK Extension B
        || ('\u{F900}'..='\u{FAFF}').contains(&c) // CJK Compatibility
}

fn is_hiragana(c: char) -> bool {
    ('\u{3040}'..='\u{309F}').contains(&c)
}

fn is_katakana(c: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&c)
}

fn is_hangul(c: char) -> bool {
    ('\u{AC00}'..='\u{D7AF}').contains(&c)    // Hangul Syllables
        || ('\u{1100}'..='\u{11FF}').contains(&c) // Hangul Jamo
}

fn is_arabic(c: char) -> bool {
    ('\u{0600}'..='\u{06FF}').contains(&c)    // Arabic
        || ('\u{0750}'..='\u{077F}').contains(&c) // Arabic Supplement
}

fn is_hebrew(c: char) -> bool {
    ('\u{0590}'..='\u{05FF}').contains(&c)
}

fn is_greek(c: char) -> bool {
    ('\u{0370}'..='\u{03FF}').contains(&c)    // Greek and Coptic
        || ('\u{1F00}'..='\u{1FFF}').contains(&c) // Greek Extended
}

fn is_cyrillic(c: char) -> bool {
    ('\u{0400}'..='\u{04FF}').contains(&c)    // Cyrillic
        || ('\u{0500}'..='\u{052F}').contains(&c) // Cyrillic Supplement
}

fn is_latin(c: char) -> bool {
    c.is_ascii_alphabetic() || ('\u{00C0}'..='\u{024F}').contains(&c)
}

fn should_skip(c: char) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_detector_cjk() {
        let detector = QuickDetector::new();

        assert!(detector.has_cjk("你好世界"));
        assert!(detector.has_cjk("こんにちは"));
        assert!(detector.has_cjk("안녕하세요"));
        assert!(!detector.has_cjk("Hello World"));
    }

    #[test]
    fn test_quick_detector_latin() {
        let detector = QuickDetector::new();

        assert!(detector.is_latin("Hello World"));
        assert!(detector.is_latin("Bonjour le monde"));
        assert!(!detector.is_latin("你好世界"));
    }

    #[test]
    fn test_quick_detector_script() {
        let detector = QuickDetector::new();

        assert_eq!(detector.detect_script("Hello"), Script::Latin);
        assert_eq!(detector.detect_script("你好"), Script::Cjk);
        assert_eq!(detector.detect_script("こんにちは"), Script::Cjk);
        assert_eq!(detector.detect_script("مرحبا"), Script::Arabic);
        assert_eq!(detector.detect_script("Привет"), Script::Cyrillic);
    }

    #[test]
    fn test_sampled_detector() {
        let detector = SampledDetector::new();

        // Short text - same as quick detection
        assert_eq!(detector.detect_script("Hello World"), Script::Latin);

        // Long text with Chinese at the beginning
        let long_chinese = "你好世界".to_string() + &"x".repeat(1000);
        assert_eq!(detector.detect_script(&long_chinese), Script::Cjk);

        // Long text with English only
        let long_english = "Hello ".to_string() + &"world ".repeat(200);
        assert_eq!(detector.detect_script(&long_english), Script::Latin);
    }

    #[test]
    fn test_quick_detector_sample_size() {
        let detector = QuickDetector::with_sample_size(5);

        // Only checks first 5 chars
        assert!(detector.is_latin("Hello World 你好"));
        assert!(!detector.has_cjk("Hello World 你好"));
    }
}
