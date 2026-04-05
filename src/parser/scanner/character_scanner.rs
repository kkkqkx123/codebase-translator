//! Core text scanner for extracting translatable text regions
//!
//! Uses character-by-character scanning to identify comments, strings,
//! and other text regions without relying on AST parsing.

use crate::parser::filtering::checks::language::QuickDetector;
use crate::parser::scanner::language::ScannerLanguageConfig;
use crate::parser::scanner::region::{PlaceholderSpan, TextRegion, TextRegionType};

/// Scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Extract comments
    pub extract_comments: bool,
    /// Extract doc strings
    pub extract_doc_strings: bool,
    /// Extract strings
    pub extract_strings: bool,
    /// Extract template strings
    pub extract_templates: bool,
    /// Target languages for detection
    pub target_languages: Vec<String>,
    /// Minimum content length
    pub min_length: usize,
    /// Maximum content length
    pub max_length: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            extract_comments: true,
            extract_doc_strings: true,
            extract_strings: false,
            extract_templates: false,
            target_languages: Vec::new(),
            min_length: 1,
            max_length: 10000,
        }
    }
}

impl ScannerConfig {
    pub fn new(target_languages: Vec<String>) -> Self {
        Self {
            target_languages,
            ..Default::default()
        }
    }

    pub fn with_comments(mut self, extract: bool) -> Self {
        self.extract_comments = extract;
        self
    }

    pub fn with_doc_strings(mut self, extract: bool) -> Self {
        self.extract_doc_strings = extract;
        self
    }

    pub fn with_strings(mut self, extract: bool) -> Self {
        self.extract_strings = extract;
        self
    }

    pub fn with_templates(mut self, extract: bool) -> Self {
        self.extract_templates = extract;
        self
    }

    pub fn with_min_length(mut self, len: usize) -> Self {
        self.min_length = len;
        self
    }

    pub fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    pub fn should_extract_type(&self, region_type: TextRegionType) -> bool {
        match region_type {
            TextRegionType::LineComment | TextRegionType::BlockComment => self.extract_comments,
            TextRegionType::DocComment => self.extract_doc_strings,
            TextRegionType::SingleQuotedString | TextRegionType::DoubleQuotedString => {
                self.extract_strings
            }
            TextRegionType::TemplateString => self.extract_templates,
            TextRegionType::RawString | TextRegionType::MultiLineString => self.extract_strings,
        }
    }
}

/// Language-aware text scanner
pub struct TextScanner {
    /// Language configuration
    language: ScannerLanguageConfig,
    /// Target language detector
    detector: QuickDetector,
    /// Scanner configuration
    config: ScannerConfig,
}

impl TextScanner {
    /// Create a new scanner with language config
    pub fn new(language: ScannerLanguageConfig, config: ScannerConfig) -> Self {
        Self {
            language,
            detector: QuickDetector::new(),
            config,
        }
    }

    /// Create a scanner from file extension
    pub fn from_extension(ext: &str, config: ScannerConfig) -> Option<Self> {
        ScannerLanguageConfig::from_extension(ext).map(|lang| Self::new(lang, config))
    }

    /// Scan content and extract all text regions containing target languages
    pub fn scan(&self, content: &str) -> Vec<TextRegion> {
        let mut regions = Vec::new();
        let bytes = content.as_bytes();
        let mut pos = 0;

        while pos < bytes.len() {
            if let Some(region) = self.try_scan_region(bytes, pos, content) {
                if self.should_extract(&region, content) {
                    regions.push(region.clone());
                }
                pos = region.full_end;
                continue;
            }

            pos += 1;
        }

        regions
    }

    /// Try to scan any type of text region starting at position
    fn try_scan_region(&self, bytes: &[u8], pos: usize, _content: &str) -> Option<TextRegion> {
        if pos >= bytes.len() {
            return None;
        }

        // Try doc comments first (longest prefix)
        for prefix in &self.language.doc_comment_prefixes {
            if bytes[pos..].starts_with(prefix.as_bytes()) {
                if prefix.starts_with("/*") || prefix.starts_with("/**") {
                    return self.scan_block_doc_comment(bytes, pos, prefix);
                } else if *prefix == "\"\"\"" || *prefix == "'''" {
                    return self.scan_multiline_string_with_type(
                        bytes,
                        pos,
                        prefix,
                        TextRegionType::DocComment,
                    );
                } else {
                    return self.scan_line_doc_comment(bytes, pos, prefix);
                }
            }
        }

        // Try block comments
        for (start, end) in &self.language.block_comment_delimiters {
            if bytes[pos..].starts_with(start.as_bytes()) {
                if self.is_doc_comment_start(bytes, pos, start) {
                    continue;
                }
                return self.scan_block_comment(bytes, pos, start, end);
            }
        }

        // Try line comments
        for prefix in &self.language.line_comment_prefixes {
            if bytes[pos..].starts_with(prefix.as_bytes()) {
                if self.is_doc_comment_or_block_start(bytes, pos, prefix) {
                    continue;
                }
                return self.scan_line_comment(bytes, pos, prefix);
            }
        }

        // Try raw strings
        for prefix in &self.language.raw_string_prefixes {
            if bytes[pos..].starts_with(prefix.as_bytes()) {
                return self.scan_raw_string(bytes, pos, prefix);
            }
        }

        // Try multi-line strings
        for delim in &self.language.multiline_delimiters {
            if bytes[pos..].starts_with(delim.as_bytes()) {
                return self.scan_multiline_string(bytes, pos, delim);
            }
        }

        // Try template strings
        if let Some(quote) = self.language.template_quote {
            if bytes[pos] == quote as u8 {
                return self.scan_template_string(bytes, pos);
            }
        }

        // Try regular strings
        for quote in &self.language.string_quotes {
            if bytes[pos] == *quote as u8 {
                return self.scan_quoted_string(bytes, pos, *quote);
            }
        }

        None
    }

    /// Check if position is a doc comment start (to skip in block comment matching)
    fn is_doc_comment_start(&self, bytes: &[u8], pos: usize, start: &str) -> bool {
        for doc_prefix in &self.language.doc_comment_prefixes {
            if doc_prefix.starts_with(start) && bytes[pos..].starts_with(doc_prefix.as_bytes()) {
                return true;
            }
        }
        false
    }

    /// Check if position is a doc comment or block comment start (to skip in line comment matching)
    fn is_doc_comment_or_block_start(&self, bytes: &[u8], pos: usize, prefix: &str) -> bool {
        for doc_prefix in &self.language.doc_comment_prefixes {
            if doc_prefix.starts_with(prefix) && bytes[pos..].starts_with(doc_prefix.as_bytes()) {
                return true;
            }
        }
        for (start, _) in &self.language.block_comment_delimiters {
            if start.starts_with(prefix) && bytes[pos..].starts_with(start.as_bytes()) {
                return true;
            }
        }
        false
    }

    /// Scan a line comment
    fn scan_line_comment(&self, bytes: &[u8], pos: usize, prefix: &str) -> Option<TextRegion> {
        let prefix_len = prefix.len();
        let mut end = pos + prefix_len;

        while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
            end += 1;
        }

        let content_start = pos + prefix_len;
        let content_start = if content_start < bytes.len() && bytes[content_start] == b' ' {
            content_start + 1
        } else {
            content_start
        };

        Some(
            TextRegion::new(TextRegionType::LineComment, pos, end)
                .with_prefix(prefix)
                .with_content_range(content_start, end),
        )
    }

    /// Scan a line doc comment
    fn scan_line_doc_comment(&self, bytes: &[u8], pos: usize, prefix: &str) -> Option<TextRegion> {
        let prefix_len = prefix.len();
        let mut end = pos + prefix_len;

        while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
            end += 1;
        }

        let content_start = pos + prefix_len;
        let content_start = if content_start < bytes.len() && bytes[content_start] == b' ' {
            content_start + 1
        } else {
            content_start
        };

        Some(
            TextRegion::new(TextRegionType::DocComment, pos, end)
                .with_prefix(prefix)
                .with_content_range(content_start, end),
        )
    }

    /// Scan a block comment
    fn scan_block_comment(
        &self,
        bytes: &[u8],
        pos: usize,
        start: &str,
        end: &str,
    ) -> Option<TextRegion> {
        let start_len = start.len();
        let mut current = pos + start_len;

        while current + end.len() <= bytes.len() {
            if bytes[current..].starts_with(end.as_bytes()) {
                let content_start = pos + start_len;
                let content_end = current;

                let content_start = if content_start < bytes.len() && bytes[content_start] == b' '
                {
                    content_start + 1
                } else {
                    content_start
                };

                let content_end = if content_end > 0 && bytes[content_end - 1] == b' ' {
                    content_end - 1
                } else {
                    content_end
                };

                return Some(
                    TextRegion::new(TextRegionType::BlockComment, pos, current + end.len())
                        .with_prefix(start)
                        .with_suffix(end)
                        .with_content_range(content_start, content_end),
                );
            }
            current += 1;
        }

        None
    }

    /// Scan a block doc comment
    fn scan_block_doc_comment(&self, bytes: &[u8], pos: usize, prefix: &str) -> Option<TextRegion> {
        let end_marker = "*/";
        let start_len = prefix.len();
        let mut current = pos + start_len;

        while current + end_marker.len() <= bytes.len() {
            if bytes[current..].starts_with(end_marker.as_bytes()) {
                let content_start = pos + start_len;
                let content_end = current;

                let content_start = if content_start < bytes.len() && bytes[content_start] == b' '
                {
                    content_start + 1
                } else {
                    content_start
                };

                let content_end = if content_end > 0 && bytes[content_end - 1] == b' ' {
                    content_end - 1
                } else {
                    content_end
                };

                return Some(
                    TextRegion::new(TextRegionType::DocComment, pos, current + end_marker.len())
                        .with_prefix(prefix)
                        .with_suffix(end_marker)
                        .with_content_range(content_start, content_end),
                );
            }
            current += 1;
        }

        None
    }

    /// Scan a template string (handles nested expressions)
    fn scan_template_string(&self, bytes: &[u8], pos: usize) -> Option<TextRegion> {
        if bytes[pos] != b'`' {
            return None;
        }

        let mut end = pos + 1;
        let mut placeholders = Vec::new();

        while end < bytes.len() {
            match bytes[end] {
                b'\\' => {
                    end += 2;
                    continue;
                }
                b'`' => {
                    end += 1;
                    break;
                }
                b'$' if end + 1 < bytes.len() && bytes[end + 1] == b'{' => {
                    let placeholder_start = end - pos - 1;
                    end += 2;

                    let mut brace_depth = 1;
                    while end < bytes.len() && brace_depth > 0 {
                        match bytes[end] {
                            b'{' => brace_depth += 1,
                            b'}' => brace_depth -= 1,
                            b'`' => {
                                if let Some(nested) = self.scan_template_string(bytes, end) {
                                    end = nested.full_end;
                                    continue;
                                }
                            }
                            b'\\' => end += 1,
                            _ => {}
                        }
                        end += 1;
                    }

                    let placeholder_end = end - pos - 1;
                    if placeholder_end > placeholder_start {
                        let original = String::from_utf8_lossy(
                            &bytes[pos + 1 + placeholder_start..end - 1],
                        )
                        .to_string();

                        placeholders.push(PlaceholderSpan::new(
                            placeholder_start,
                            placeholder_end,
                            original,
                        ));
                    }
                }
                _ => {
                    end += 1;
                }
            }
        }

        Some(
            TextRegion::new(TextRegionType::TemplateString, pos, end)
                .with_prefix("`")
                .with_suffix("`")
                .with_content_range(pos + 1, end - 1)
                .with_placeholders(placeholders),
        )
    }

    /// Scan a quoted string
    fn scan_quoted_string(&self, bytes: &[u8], pos: usize, quote: char) -> Option<TextRegion> {
        let quote_byte = quote as u8;
        if bytes[pos] != quote_byte {
            return None;
        }

        let mut end = pos + 1;

        while end < bytes.len() {
            match bytes[end] {
                b'\\' => {
                    end += 2;
                    continue;
                }
                b'\n' | b'\r' => {
                    return None;
                }
                c if c == quote_byte => {
                    end += 1;
                    break;
                }
                _ => {
                    end += 1;
                }
            }
        }

        let region_type = if quote == '"' {
            TextRegionType::DoubleQuotedString
        } else {
            TextRegionType::SingleQuotedString
        };

        Some(
            TextRegion::new(region_type, pos, end)
                .with_prefix(quote.to_string())
                .with_suffix(quote.to_string())
                .with_content_range(pos + 1, end - 1),
        )
    }

    /// Scan a raw string
    fn scan_raw_string(&self, bytes: &[u8], pos: usize, prefix: &str) -> Option<TextRegion> {
        let prefix_len = prefix.len();
        let mut end = pos + prefix_len;

        // Determine the end delimiter based on prefix
        let end_delim = if prefix.contains('#') {
            let hash_count = prefix.chars().filter(|&c| c == '#').count();
            "\"".to_string() + &"#".repeat(hash_count)
        } else {
            "\"".to_string()
        };

        // Find the content start (after opening quote)
        if end < bytes.len() && bytes[end] == b'"' {
            end += 1;
        } else {
            return None;
        }

        let content_start = end;

        // Find the end delimiter
        while end + end_delim.len() <= bytes.len() {
            if bytes[end..].starts_with(end_delim.as_bytes()) {
                let content_end = end;
                return Some(
                    TextRegion::new(TextRegionType::RawString, pos, end + end_delim.len())
                        .with_prefix(prefix)
                        .with_suffix(&end_delim)
                        .with_content_range(content_start, content_end),
                );
            }
            end += 1;
        }

        None
    }

    /// Scan a multi-line string
    fn scan_multiline_string(&self, bytes: &[u8], pos: usize, delim: &str) -> Option<TextRegion> {
        self.scan_multiline_string_with_type(bytes, pos, delim, TextRegionType::MultiLineString)
    }

    /// Scan a multi-line string with specified region type
    fn scan_multiline_string_with_type(
        &self,
        bytes: &[u8],
        pos: usize,
        delim: &str,
        region_type: TextRegionType,
    ) -> Option<TextRegion> {
        let delim_len = delim.len();
        let mut end = pos + delim_len;

        // Skip the opening delimiter
        let content_start = end;

        // Find the closing delimiter
        while end + delim_len <= bytes.len() {
            if bytes[end..].starts_with(delim.as_bytes()) {
                let content_end = end;
                return Some(
                    TextRegion::new(region_type, pos, end + delim_len)
                        .with_prefix(delim)
                        .with_suffix(delim)
                        .with_content_range(content_start, content_end),
                );
            }
            end += 1;
        }

        None
    }

    /// Determine if a region should be extracted
    fn should_extract(&self, region: &TextRegion, content: &str) -> bool {
        if region.is_empty() {
            return false;
        }

        if !self.config.should_extract_type(region.region_type) {
            return false;
        }

        let content_len = region.content_length();
        if content_len < self.config.min_length || content_len > self.config.max_length {
            return false;
        }

        let text = match region.extract_content(content) {
            Some(t) => t,
            None => return false,
        };

        self.contains_target_language(text)
    }

    /// Check if text contains target language characters
    fn contains_target_language(&self, text: &str) -> bool {
        if self.config.target_languages.is_empty() {
            return true;
        }

        for lang in &self.config.target_languages {
            let lang_upper = lang.to_uppercase();
            match lang_upper.as_str() {
                "ZH" | "ZH-CN" | "ZH-TW" | "HANS" | "HANT" | "CHINESE" => {
                    if self.detector.has_chinese(text) {
                        return true;
                    }
                }
                "JA" | "JAPANESE" => {
                    if self.detector.has_japanese(text) {
                        return true;
                    }
                }
                "KO" | "KOREAN" => {
                    if self.detector.has_korean(text) {
                        return true;
                    }
                }
                "AR" | "ARABIC" => {
                    if self.detector.has_arabic(text) {
                        return true;
                    }
                }
                "RU" | "RUSSIAN" => {
                    if self.detector.has_cyrillic(text) {
                        return true;
                    }
                }
                "EL" | "GREEK" => {
                    if self.detector.has_greek(text) {
                        return true;
                    }
                }
                "HE" | "HEBREW" => {
                    if self.detector.has_hebrew(text) {
                        return true;
                    }
                }
                _ => {
                    return true;
                }
            }
        }
        false
    }

    /// Get the language configuration
    pub fn language(&self) -> &ScannerLanguageConfig {
        &self.language
    }

    /// Get the scanner configuration
    pub fn config(&self) -> &ScannerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_scanner(lang: &str) -> TextScanner {
        let config = ScannerConfig::new(vec!["zh".to_string()])
            .with_comments(true)
            .with_doc_strings(true);
        TextScanner::from_extension(lang, config).expect("Failed to create scanner")
    }

    #[test]
    fn test_scan_line_comment() {
        let scanner = create_test_scanner("js");
        let content = "// 这是注释\n";
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::LineComment);
        assert_eq!(regions[0].extract_content(content).unwrap(), "这是注释");
    }

    #[test]
    fn test_scan_block_comment() {
        let scanner = create_test_scanner("js");
        let content = "/* 这是块注释 */";
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::BlockComment);
        assert_eq!(regions[0].extract_content(content).unwrap(), "这是块注释");
    }

    #[test]
    fn test_scan_doc_comment() {
        let scanner = create_test_scanner("js");
        let content = "/** 这是文档注释 */";
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::DocComment);
    }

    #[test]
    fn test_scan_template_string() {
        let config = ScannerConfig::new(vec!["zh".to_string()])
            .with_comments(true)
            .with_doc_strings(true)
            .with_templates(true);
        let scanner = TextScanner::from_extension("js", config).expect("Failed to create scanner");
        let content = r#"const msg = `你好 ${name}，欢迎！`;"#;
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::TemplateString);
        assert!(!regions[0].placeholders.is_empty());
    }

    #[test]
    fn test_scan_quoted_string() {
        let config = ScannerConfig::new(vec!["zh".to_string()])
            .with_strings(true);
        let scanner = TextScanner::from_extension("js", config).expect("Failed to create scanner");
        let content = r#"const msg = "你好世界";"#;
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::DoubleQuotedString);
    }

    #[test]
    fn test_scan_python_multiline() {
        let config = ScannerConfig::new(vec!["zh".to_string()])
            .with_comments(true)
            .with_doc_strings(true)
            .with_strings(true);
        let scanner = TextScanner::from_extension("py", config).expect("Failed to create scanner");
        let content = "\"\"\"\n这是多行字符串\n第二行\n\"\"\"";
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::DocComment);
    }

    #[test]
    fn test_skip_english_only() {
        let scanner = create_test_scanner("js");
        let content = r#"// This is English comment
const msg = "Hello World";"#;
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 0);
    }

    #[test]
    fn test_nested_template_string() {
        let config = ScannerConfig::new(vec!["zh".to_string()])
            .with_comments(true)
            .with_doc_strings(true)
            .with_templates(true);
        let scanner = TextScanner::from_extension("js", config).expect("Failed to create scanner");
        let content = r#"const msg = `外层 ${`内层 ${value}`} 结束`;"#;
        let regions = scanner.scan(content);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, TextRegionType::TemplateString);
    }
}
