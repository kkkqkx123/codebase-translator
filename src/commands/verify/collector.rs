use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::models::{NodeType, PatternType, Position, TranslationUnit};

/// A verified match from extraction rules
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifyMatch {
    pub file_path: PathBuf,
    pub pattern_name: String,
    pub pattern_type: PatternType,
    pub category: String,
    pub extracted_text: String,
    pub position: Position,
    pub raw_match: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl VerifyMatch {
    pub fn new(
        file_path: PathBuf,
        pattern_name: String,
        pattern_type: PatternType,
        category: String,
        extracted_text: String,
        position: Position,
        raw_match: Option<String>,
    ) -> Self {
        Self {
            file_path,
            pattern_name,
            pattern_type,
            category,
            extracted_text,
            position,
            raw_match,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Collector for extracting verification matches from translation units
pub struct MatchCollector;

impl MatchCollector {
    pub fn collect_from_units(
        file_path: PathBuf,
        units: Vec<TranslationUnit>,
        content: &str,
    ) -> Vec<VerifyMatch> {
        let mut matches = Vec::new();

        for unit in units {
            let pattern_name = Self::extract_pattern_name(&unit);
            let pattern_type = Self::determine_pattern_type(&unit);
            let category = Self::extract_category(&unit);
            let raw_match = Self::extract_raw_match(&unit, content);

            let verify_match = VerifyMatch::new(
                file_path.clone(),
                pattern_name,
                pattern_type,
                category,
                unit.content.clone(),
                unit.start_pos,
                raw_match,
            );

            matches.push(verify_match);
        }

        matches
    }

    fn extract_pattern_name(unit: &TranslationUnit) -> String {
        if let Some(name) = &unit.pattern_name {
            name.clone()
        } else {
            match unit.node_type {
                NodeType::Comment => "comment".to_string(),
                NodeType::DocString => "docstring".to_string(),
                NodeType::ErrorMessage => "error_message".to_string(),
                NodeType::FormatString => "format_string".to_string(),
                NodeType::LogMessage => "log_message".to_string(),
                NodeType::StringLiteral => "string_literal".to_string(),
            }
        }
    }

    fn determine_pattern_type(unit: &TranslationUnit) -> PatternType {
        unit.pattern_type.unwrap_or(PatternType::Builtin)
    }

    fn extract_category(unit: &TranslationUnit) -> String {
        match unit.node_type {
            NodeType::ErrorMessage => "error_handling".to_string(),
            NodeType::LogMessage => "output".to_string(),
            NodeType::FormatString => "output".to_string(),
            NodeType::StringLiteral => "variables".to_string(),
            _ => "other".to_string(),
        }
    }

    fn extract_raw_match(unit: &TranslationUnit, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();

        if unit.start_pos.line == 0 || unit.start_pos.line > lines.len() {
            return None;
        }

        let line_index = unit.start_pos.line - 1;
        let line = lines[line_index];

        let start_col = unit.start_pos.column.saturating_sub(1);
        let end_col = unit.end_pos.column.saturating_sub(1);

        if start_col <= end_col && end_col <= line.len() {
            Some(line[start_col..end_col].to_string())
        } else {
            Some(line.to_string())
        }
    }
}
