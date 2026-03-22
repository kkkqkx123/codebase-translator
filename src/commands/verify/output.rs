use super::collector::VerifyMatch;
use super::stats::VerifySummary;
use crate::core::error::{Result, TranslateError};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

/// Output formatter for verification results
pub struct OutputFormatter;

impl OutputFormatter {
    pub fn format(
        matches: &[VerifyMatch],
        summary: &VerifySummary,
        format: OutputFormat,
        detailed: bool,
        show_stats: bool,
    ) -> Result<String> {
        match format {
            OutputFormat::Table => Self::format_table(matches, summary, detailed, show_stats),
            OutputFormat::Json => Self::format_json(matches, summary),
            OutputFormat::Csv => Self::format_csv(matches, detailed),
        }
    }

    fn format_table(
        matches: &[VerifyMatch],
        summary: &VerifySummary,
        detailed: bool,
        show_stats: bool,
    ) -> Result<String> {
        use comfy_table::*;

        let mut table = Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);

        if detailed {
            table.set_header(vec![
                "Pattern",
                "Type",
                "Category",
                "File",
                "Line",
                "Extracted Text",
                "Raw Match",
            ]);
        } else {
            table.set_header(vec![
                "Pattern",
                "Type",
                "Category",
                "File",
                "Line",
                "Extracted Text",
            ]);
        }

        for m in matches {
            if detailed {
                table.add_row(vec![
                    m.pattern_name.clone(),
                    format!("{}", m.pattern_type),
                    m.category.clone(),
                    Self::format_filename(&m.file_path),
                    m.position.line.to_string(),
                    Self::truncate_text(&m.extracted_text, 40),
                    m.raw_match.clone().unwrap_or_else(|| "-".to_string()),
                ]);
            } else {
                table.add_row(vec![
                    m.pattern_name.clone(),
                    format!("{}", m.pattern_type),
                    m.category.clone(),
                    Self::format_filename(&m.file_path),
                    m.position.line.to_string(),
                    Self::truncate_text(&m.extracted_text, 60),
                ]);
            }
        }

        let mut output = table.to_string();

        if show_stats {
            output.push_str("\n\n");
            output.push_str("=== Summary ===\n");
            output.push_str(&format!("Total files: {}\n", summary.total_files));
            output.push_str(&format!("Total matches: {}\n", summary.total_matches));

            if !summary.patterns_used.is_empty() {
                output.push_str("\nPatterns used:\n");
                for (pattern, count) in &summary.patterns_used {
                    output.push_str(&format!("  - {}: {}\n", pattern, count));
                }
            }

            if !summary.by_category.is_empty() {
                output.push_str("\nBy category:\n");
                for (category, count) in &summary.by_category {
                    output.push_str(&format!("  - {}: {}\n", category, count));
                }
            }

            if !summary.by_file_type.is_empty() {
                output.push_str("\nBy file type:\n");
                for (file_type, count) in &summary.by_file_type {
                    output.push_str(&format!("  - {}: {}\n", file_type, count));
                }
            }

            if !summary.by_pattern_type.is_empty() {
                output.push_str("\nBy pattern type:\n");
                for (pattern_type, count) in &summary.by_pattern_type {
                    output.push_str(&format!("  - {}: {}\n", pattern_type, count));
                }
            }
        }

        Ok(output)
    }

    fn format_json(matches: &[VerifyMatch], summary: &VerifySummary) -> Result<String> {
        let output = serde_json::json!({
            "summary": summary,
            "matches": matches
        });
        serde_json::to_string_pretty(&output)
            .map_err(|e| TranslateError::Parse(format!("Failed to serialize JSON: {}", e)))
    }

    fn format_csv(matches: &[VerifyMatch], detailed: bool) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        if detailed {
            wtr.write_record([
                "pattern",
                "type",
                "category",
                "file",
                "line",
                "column",
                "extracted_text",
                "raw_match",
            ])
            .map_err(|e| TranslateError::Parse(format!("Failed to write CSV header: {}", e)))?;
        } else {
            wtr.write_record([
                "pattern",
                "type",
                "category",
                "file",
                "line",
                "extracted_text",
            ])
            .map_err(|e| TranslateError::Parse(format!("Failed to write CSV header: {}", e)))?;
        }

        for m in matches {
            if detailed {
                wtr.write_record([
                    &m.pattern_name,
                    &format!("{}", m.pattern_type),
                    &m.category,
                    &m.file_path.display().to_string(),
                    &m.position.line.to_string(),
                    &m.position.column.to_string(),
                    &m.extracted_text,
                    &m.raw_match.clone().unwrap_or_default(),
                ])
                .map_err(|e| TranslateError::Parse(format!("Failed to write CSV record: {}", e)))?;
            } else {
                wtr.write_record([
                    &m.pattern_name,
                    &format!("{}", m.pattern_type),
                    &m.category,
                    &m.file_path.display().to_string(),
                    &m.position.line.to_string(),
                    &m.extracted_text,
                ])
                .map_err(|e| TranslateError::Parse(format!("Failed to write CSV record: {}", e)))?;
            }
        }

        let data = wtr
            .into_inner()
            .map_err(|e| TranslateError::Parse(format!("Failed to finalize CSV: {}", e)))?;
        String::from_utf8(data)
            .map_err(|e| TranslateError::Parse(format!("Invalid UTF-8 in CSV: {}", e)))
    }

    fn format_filename(path: &std::path::Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.display().to_string())
    }

    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len.saturating_sub(3)])
        }
    }
}
