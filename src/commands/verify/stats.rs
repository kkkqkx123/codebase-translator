use std::collections::HashMap;

use super::collector::VerifyMatch;

/// Summary statistics for verification results
#[derive(Debug, Clone, serde::Serialize)]
pub struct VerifySummary {
    pub total_files: usize,
    pub total_matches: usize,
    pub patterns_used: HashMap<String, usize>,
    pub by_category: HashMap<String, usize>,
    pub by_file_type: HashMap<String, usize>,
    pub by_pattern_type: HashMap<String, usize>,
}

impl VerifySummary {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            total_matches: 0,
            patterns_used: HashMap::new(),
            by_category: HashMap::new(),
            by_file_type: HashMap::new(),
            by_pattern_type: HashMap::new(),
        }
    }
}

impl Default for VerifySummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics generator for verification results
pub struct StatisticsGenerator;

impl StatisticsGenerator {
    pub fn generate(matches: &[VerifyMatch], total_files: usize) -> VerifySummary {
        let mut summary = VerifySummary::new();
        summary.total_files = total_files;
        summary.total_matches = matches.len();

        for m in matches {
            *summary
                .patterns_used
                .entry(m.pattern_name.clone())
                .or_insert(0) += 1;
            *summary.by_category.entry(m.category.clone()).or_insert(0) += 1;

            let file_type = m
                .file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_lowercase();
            *summary.by_file_type.entry(file_type).or_insert(0) += 1;

            let pattern_type = format!("{}", m.pattern_type);
            *summary.by_pattern_type.entry(pattern_type).or_insert(0) += 1;
        }

        summary
    }
}
