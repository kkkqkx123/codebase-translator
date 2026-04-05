//! Translation replacer for applying translations to source content
//!
//! Provides functionality to replace translated content based on byte offsets.

use crate::parser::scanner::region::TranslatedRegion;

/// Translation replacer - applies translations based on byte offsets
pub struct TranslationReplacer;

impl TranslationReplacer {
    /// Apply translations to content
    ///
    /// Replaces all translated regions in the content using their byte offsets.
    /// Regions are sorted by position (descending) and replaced from end to start
    /// to avoid offset changes affecting subsequent replacements.
    pub fn apply(content: &str, regions: &[TranslatedRegion]) -> String {
        if regions.is_empty() {
            return content.to_string();
        }

        let mut sorted: Vec<_> = regions.iter().collect();
        sorted.sort_by(|a, b| b.content_start.cmp(&a.content_start));

        let mut result = content.to_string();

        for region in sorted {
            if region.content_start < region.content_end && region.content_end <= result.len() {
                result.replace_range(
                    region.content_start..region.content_end,
                    &region.translated_content,
                );
            }
        }

        result
    }

    /// Apply a single translation
    pub fn apply_single(content: &str, region: &TranslatedRegion) -> String {
        if region.content_start >= region.content_end || region.content_end > content.len() {
            return content.to_string();
        }

        let mut result = content.to_string();
        result.replace_range(
            region.content_start..region.content_end,
            &region.translated_content,
        );
        result
    }

    /// Validate that replacements don't overlap
    pub fn validate_regions(regions: &[TranslatedRegion]) -> Result<(), String> {
        let mut sorted: Vec<_> = regions.iter().collect();
        sorted.sort_by(|a, b| a.content_start.cmp(&b.content_start));

        for window in sorted.windows(2) {
            let prev = &window[0];
            let next = &window[1];

            if prev.content_end > next.content_start {
                return Err(format!(
                    "Overlapping regions: [{}, {}) overlaps with [{}, {})",
                    prev.content_start, prev.content_end, next.content_start, next.content_end
                ));
            }
        }

        Ok(())
    }

    /// Merge adjacent or overlapping regions
    pub fn merge_regions(regions: Vec<TranslatedRegion>) -> Vec<TranslatedRegion> {
        if regions.is_empty() {
            return regions;
        }

        let mut sorted = regions;
        sorted.sort_by(|a, b| a.content_start.cmp(&b.content_start));

        let mut merged = Vec::new();
        let mut current = sorted[0].clone();

        for region in sorted.into_iter().skip(1) {
            if region.content_start <= current.content_end {
                current.content_end = current.content_end.max(region.content_end);
                current.translated_content =
                    current.translated_content + &region.translated_content;
            } else {
                merged.push(current);
                current = region;
            }
        }
        merged.push(current);

        merged
    }

    /// Calculate the byte offset difference after replacement
    pub fn calculate_offset_change(original_len: usize, translated_len: usize) -> isize {
        translated_len as isize - original_len as isize
    }

    /// Adjust offsets after a replacement
    pub fn adjust_offsets(
        regions: &mut [TranslatedRegion],
        position: usize,
        delta: isize,
    ) {
        for region in regions.iter_mut() {
            if region.content_start > position {
                region.content_start = (region.content_start as isize + delta) as usize;
                region.content_end = (region.content_end as isize + delta) as usize;
            }
        }
    }
}

/// Content diff for tracking changes
#[derive(Debug, Clone)]
pub struct ContentDiff {
    /// Original content
    pub original: String,
    /// Modified content
    pub modified: String,
    /// List of changes
    pub changes: Vec<Change>,
}

/// A single change in the content
#[derive(Debug, Clone)]
pub struct Change {
    /// Start position in original content
    pub original_start: usize,
    /// End position in original content
    pub original_end: usize,
    /// Original text
    pub original_text: String,
    /// Replacement text
    pub replacement_text: String,
}

impl ContentDiff {
    /// Create a new content diff
    pub fn new(original: &str, modified: &str) -> Self {
        Self {
            original: original.to_string(),
            modified: modified.to_string(),
            changes: Vec::new(),
        }
    }

    /// Add a change
    pub fn add_change(
        &mut self,
        original_start: usize,
        original_end: usize,
        original_text: String,
        replacement_text: String,
    ) {
        self.changes.push(Change {
            original_start,
            original_end,
            original_text,
            replacement_text,
        });
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Get the number of changes
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    /// Generate a unified diff
    pub fn to_unified_diff(&self, filename: &str) -> String {
        let mut diff = String::new();
        diff.push_str(&format!("--- {}\n", filename));
        diff.push_str(&format!("+++ {}\n", filename));

        for change in &self.changes {
            diff.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                change.original_start,
                change.original_end - change.original_start,
                change.original_start,
                change.replacement_text.len()
            ));
            diff.push_str(&format!("-{}\n", change.original_text));
            diff.push_str(&format!("+{}\n", change.replacement_text));
        }

        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_single_translation() {
        let content = "// 这是注释\n";
        let region = TranslatedRegion::new(3, 7, "这是注释".to_string(), "This is comment".to_string());

        let result = TranslationReplacer::apply_single(content, &region);
        assert_eq!(result, "// This is comment\n");
    }

    #[test]
    fn test_apply_multiple_translations() {
        let content = "// 第一行\n// 第二行\n";
        let regions = vec![
            TranslatedRegion::new(3, 6, "第一行".to_string(), "Line 1".to_string()),
            TranslatedRegion::new(13, 16, "第二行".to_string(), "Line 2".to_string()),
        ];

        let result = TranslationReplacer::apply(content, &regions);
        assert_eq!(result, "// Line 1\n// Line 2\n");
    }

    #[test]
    fn test_validate_non_overlapping() {
        let regions = vec![
            TranslatedRegion::new(0, 5, "hello".to_string(), "HELLO".to_string()),
            TranslatedRegion::new(10, 15, "world".to_string(), "WORLD".to_string()),
        ];

        assert!(TranslationReplacer::validate_regions(&regions).is_ok());
    }

    #[test]
    fn test_validate_overlapping() {
        let regions = vec![
            TranslatedRegion::new(0, 10, "hello".to_string(), "HELLO".to_string()),
            TranslatedRegion::new(5, 15, "world".to_string(), "WORLD".to_string()),
        ];

        assert!(TranslationReplacer::validate_regions(&regions).is_err());
    }

    #[test]
    fn test_merge_adjacent_regions() {
        let regions = vec![
            TranslatedRegion::new(0, 5, "hello".to_string(), "HELLO".to_string()),
            TranslatedRegion::new(5, 10, "world".to_string(), "WORLD".to_string()),
        ];

        let merged = TranslationReplacer::merge_regions(regions);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].content_start, 0);
        assert_eq!(merged[0].content_end, 10);
    }

    #[test]
    fn test_offset_calculation() {
        let change = TranslationReplacer::calculate_offset_change(5, 10);
        assert_eq!(change, 5);

        let change = TranslationReplacer::calculate_offset_change(10, 5);
        assert_eq!(change, -5);
    }

    #[test]
    fn test_content_diff() {
        let mut diff = ContentDiff::new("hello world", "HELLO WORLD");
        diff.add_change(0, 5, "hello".to_string(), "HELLO".to_string());

        assert!(diff.has_changes());
        assert_eq!(diff.change_count(), 1);
    }
}
