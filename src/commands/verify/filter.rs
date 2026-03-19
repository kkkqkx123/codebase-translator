use super::collector::VerifyMatch;

/// Filter options for verification results
#[derive(Debug, Clone, Default)]
pub struct FilterOptions {
    pub pattern_name: Option<String>,
    pub extension: Option<String>,
    pub category: Option<String>,
    pub search_text: Option<String>,
}

impl FilterOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pattern_name(mut self, pattern: String) -> Self {
        self.pattern_name = Some(pattern);
        self
    }

    pub fn with_extension(mut self, ext: String) -> Self {
        self.extension = Some(ext);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_search_text(mut self, text: String) -> Self {
        self.search_text = Some(text);
        self
    }
}

/// Filter for verification matches
pub struct MatchFilter;

impl MatchFilter {
    pub fn filter(matches: Vec<VerifyMatch>, options: &FilterOptions) -> Vec<VerifyMatch> {
        matches
            .into_iter()
            .filter(|m| Self::matches_filter(m, options))
            .collect()
    }

    fn matches_filter(match_item: &VerifyMatch, options: &FilterOptions) -> bool {
        if let Some(pattern) = &options.pattern_name {
            if !match_item.pattern_name.contains(pattern) {
                return false;
            }
        }

        if let Some(ext) = &options.extension {
            let file_ext = match_item
                .file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !file_ext.eq_ignore_ascii_case(ext) {
                return false;
            }
        }

        if let Some(category) = &options.category {
            if match_item.category != *category {
                return false;
            }
        }

        if let Some(search) = &options.search_text {
            if !match_item.extracted_text.contains(search) {
                return false;
            }
        }

        true
    }
}
