//! Core data structures for text region scanning
//!
//! Defines the types used to represent text regions extracted from source files.

use serde::{Deserialize, Serialize};

/// Text region type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextRegionType {
    /// Single line comment // ...
    LineComment,
    /// Block comment /* ... */
    BlockComment,
    /// Doc comment /** ... */ or /// ...
    DocComment,
    /// Single quoted string '...'
    SingleQuotedString,
    /// Double quoted string "..."
    DoubleQuotedString,
    /// Template string `...`
    TemplateString,
    /// Raw string r#"..."#, r"..."
    RawString,
    /// Multi-line string """...""", '''...'''
    MultiLineString,
}

impl TextRegionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LineComment => "line_comment",
            Self::BlockComment => "block_comment",
            Self::DocComment => "doc_comment",
            Self::SingleQuotedString => "single_quoted_string",
            Self::DoubleQuotedString => "double_quoted_string",
            Self::TemplateString => "template_string",
            Self::RawString => "raw_string",
            Self::MultiLineString => "multiline_string",
        }
    }

    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            Self::LineComment | Self::BlockComment | Self::DocComment
        )
    }

    pub fn is_string(&self) -> bool {
        matches!(
            self,
            Self::SingleQuotedString
                | Self::DoubleQuotedString
                | Self::TemplateString
                | Self::RawString
                | Self::MultiLineString
        )
    }

    pub fn is_doc(&self) -> bool {
        matches!(self, Self::DocComment)
    }

    pub fn has_placeholders(&self) -> bool {
        matches!(self, Self::TemplateString)
    }
}

impl std::fmt::Display for TextRegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Template placeholder span
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceholderSpan {
    /// Placeholder start position (relative to content)
    pub start: usize,
    /// Placeholder end position (relative to content)
    pub end: usize,
    /// Original placeholder text
    pub original: String,
}

impl PlaceholderSpan {
    pub fn new(start: usize, end: usize, original: String) -> Self {
        Self {
            start,
            end,
            original,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Scanned text region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    /// Region type
    pub region_type: TextRegionType,
    /// Content start byte offset (excluding prefix)
    pub content_start: usize,
    /// Content end byte offset (excluding suffix)
    pub content_end: usize,
    /// Full region start byte offset (including prefix)
    pub full_start: usize,
    /// Full region end byte offset (including suffix)
    pub full_end: usize,
    /// Prefix (e.g. "// ", "/* ", "\"")
    pub prefix: String,
    /// Suffix (e.g. " */", "\"")
    pub suffix: String,
    /// Template placeholder positions (only for template strings)
    pub placeholders: Vec<PlaceholderSpan>,
}

impl TextRegion {
    pub fn new(region_type: TextRegionType, full_start: usize, full_end: usize) -> Self {
        Self {
            region_type,
            content_start: full_start,
            content_end: full_end,
            full_start,
            full_end,
            prefix: String::new(),
            suffix: String::new(),
            placeholders: Vec::new(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub fn with_content_range(mut self, start: usize, end: usize) -> Self {
        self.content_start = start;
        self.content_end = end;
        self
    }

    pub fn with_placeholders(mut self, placeholders: Vec<PlaceholderSpan>) -> Self {
        self.placeholders = placeholders;
        self
    }

    pub fn content_length(&self) -> usize {
        self.content_end.saturating_sub(self.content_start)
    }

    pub fn full_length(&self) -> usize {
        self.full_end.saturating_sub(self.full_start)
    }

    pub fn is_empty(&self) -> bool {
        self.content_start >= self.content_end
    }

    pub fn extract_content<'a>(&self, source: &'a str) -> Option<&'a str> {
        if self.content_start >= self.content_end || self.content_end > source.len() {
            return None;
        }
        source.get(self.content_start..self.content_end)
    }

    pub fn extract_full<'a>(&self, source: &'a str) -> Option<&'a str> {
        if self.full_start >= self.full_end || self.full_end > source.len() {
            return None;
        }
        source.get(self.full_start..self.full_end)
    }
}

/// Translated region for replacement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedRegion {
    /// Content start byte offset
    pub content_start: usize,
    /// Content end byte offset
    pub content_end: usize,
    /// Original content
    pub original_content: String,
    /// Translated content
    pub translated_content: String,
}

impl TranslatedRegion {
    pub fn new(
        content_start: usize,
        content_end: usize,
        original_content: String,
        translated_content: String,
    ) -> Self {
        Self {
            content_start,
            content_end,
            original_content,
            translated_content,
        }
    }

    pub fn from_region(region: &TextRegion, translated: String, source: &str) -> Option<Self> {
        let original = region.extract_content(source)?.to_string();
        Some(Self::new(
            region.content_start,
            region.content_end,
            original,
            translated,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_region_type() {
        assert!(TextRegionType::LineComment.is_comment());
        assert!(TextRegionType::BlockComment.is_comment());
        assert!(TextRegionType::DocComment.is_comment());
        assert!(TextRegionType::DocComment.is_doc());

        assert!(TextRegionType::DoubleQuotedString.is_string());
        assert!(TextRegionType::TemplateString.is_string());
        assert!(TextRegionType::TemplateString.has_placeholders());
        assert!(!TextRegionType::DoubleQuotedString.has_placeholders());
    }

    #[test]
    fn test_text_region() {
        let region = TextRegion::new(TextRegionType::LineComment, 0, 15)
            .with_prefix("// ")
            .with_content_range(3, 15);

        assert_eq!(region.content_length(), 12);
        assert_eq!(region.full_length(), 15);
        assert!(!region.is_empty());
    }

    #[test]
    fn test_extract_content() {
        let source = "// 这是注释";
        let region = TextRegion::new(TextRegionType::LineComment, 0, 15)
            .with_prefix("// ")
            .with_content_range(3, 15);

        let content = region.extract_content(source).unwrap();
        assert_eq!(content, "这是注释");

        let full = region.extract_full(source).unwrap();
        assert_eq!(full, "// 这是注释");
    }

    #[test]
    fn test_placeholder_span() {
        let ph = PlaceholderSpan::new(5, 15, "${name}".to_string());
        assert_eq!(ph.len(), 10);
        assert!(!ph.is_empty());
    }
}
