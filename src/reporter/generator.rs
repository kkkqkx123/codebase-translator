use std::path::Path;
use crate::core::error::TranslateError;
use crate::reporter::r#trait::ReportFormat;
use crate::reporter::stats::TranslationStats;

pub trait ReportGenerator: Send + Sync {
    fn generate(&self, stats: &TranslationStats, format: ReportFormat) -> Result<String, TranslateError>;
    
    fn save(&self, path: &Path, stats: &TranslationStats, format: ReportFormat) -> Result<(), TranslateError> {
        let report = self.generate(stats, format)?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TranslateError::Io(e.to_string()))?;
        }
        
        std::fs::write(path, report).map_err(|e| TranslateError::Io(e.to_string()))?;
        Ok(())
    }
    
    fn save_with_template(
        &self,
        dir: &Path,
        template: &str,
        stats: &TranslationStats,
        format: ReportFormat,
    ) -> Result<std::path::PathBuf, TranslateError> {
        std::fs::create_dir_all(dir).map_err(|e| TranslateError::Io(e.to_string()))?;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let ext = match format {
            ReportFormat::Text => "txt",
            ReportFormat::Json => "json",
        };
        let filename = template
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{format}", ext);
        let path = dir.join(filename);
        
        self.save(&path, stats, format)?;
        Ok(path)
    }
}

#[derive(Debug, Clone)]
pub struct DefaultReportGenerator;

impl DefaultReportGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator for DefaultReportGenerator {
    fn generate(&self, stats: &TranslationStats, format: ReportFormat) -> Result<String, TranslateError> {
        match format {
            ReportFormat::Text => self.generate_text_report(stats),
            ReportFormat::Json => self.generate_json_report(stats),
        }
    }
}

impl DefaultReportGenerator {
    fn generate_text_report(&self, stats: &TranslationStats) -> Result<String, TranslateError> {
        let end_time = stats
            .end_time
            .ok_or_else(|| TranslateError::Parse("Stats should be finalized".to_string()))?;

        let duration = end_time.signed_duration_since(stats.start_time);

        let mut report = String::new();
        report.push_str(&format!("\n{}\n", "=".repeat(60)));
        report.push_str("Translation Report\n");
        report.push_str(&format!("{}\n\n", "=".repeat(60)));

        report.push_str("Time:\n");
        report.push_str(&format!(
            "  Start:      {}\n",
            stats.start_time.format("%Y-%m-%d %H:%M:%S")
        ));
        report.push_str(&format!(
            "  End:        {}\n",
            end_time.format("%Y-%m-%d %H:%M:%S")
        ));
        report.push_str(&format!(
            "  Duration:   {:.3}s\n",
            duration.num_milliseconds() as f64 / 1000.0
        ));
        report.push_str(&format!(
            "  Speed:      {:.1} files/s\n\n",
            stats.avg_speed_files_per_sec
        ));

        report.push_str("Files:\n");
        report.push_str(&format!("  Total:      {}\n", stats.total_files));
        report.push_str(&format!("  Processed:  {}\n", stats.processed_files));
        report.push_str(&format!("  Skipped:    {}\n", stats.skipped_files));
        report.push_str(&format!("  Failed:     {}\n\n", stats.failed_files));

        report.push_str("Translation Units:\n");
        report.push_str(&format!("  Total:      {}\n", stats.total_units));
        report.push_str(&format!("  Translated: {}\n", stats.translated_units));

        if stats.total_units > 0 {
            let percentage = self.calculate_translation_progress(stats);
            report.push_str(&format!("  Progress:   {:.1}%\n\n", percentage));
        }

        report.push_str("API Calls:\n");
        report.push_str(&format!("  Total:      {}\n\n", stats.api_call_count));

        report.push_str("Cache:\n");
        report.push_str(&format!("  Hits:       {}\n", stats.cache_hit_count));
        report.push_str(&format!("  Misses:     {}\n", stats.cache_miss_count));

        if stats.cache_hit_count + stats.cache_miss_count > 0 {
            let hit_rate = self.calculate_cache_hit_rate(stats);
            report.push_str(&format!("  Hit Rate:   {:.1}%\n\n", hit_rate));
        }

        if stats.error_count > 0 {
            report.push_str(&format!("Errors ({})\n", stats.error_count));
            for (i, err) in stats.errors.iter().enumerate() {
                report.push_str(&format!("  {}. {}: {}\n", i + 1, err.file_path, err.error));
            }
            report.push('\n');
        }

        if !stats.translator_stats.is_empty() {
            report.push_str("Translator Statistics:\n");
            for (name, stat) in &stats.translator_stats {
                report.push_str(&format!("  {}:\n", name));
                report.push_str(&format!(
                    "    Calls:      {} (success: {}, failed: {})\n",
                    stat.total_calls, stat.successful_calls, stat.failed_calls
                ));
                report.push_str(&format!("    Characters: {}\n", stat.total_chars));
                report.push_str(&format!(
                    "    Latency:    avg {:.1}ms",
                    stat.average_latency_ms
                ));
                if let Some(min) = stat.min_latency_ms {
                    report.push_str(&format!(", min {:.1}ms", min));
                }
                if let Some(max) = stat.max_latency_ms {
                    report.push_str(&format!(", max {:.1}ms", max));
                }
                report.push('\n');
            }
            report.push('\n');
        }

        if !stats.llm_provider_stats.is_empty() {
            report.push_str("LLM Provider Statistics:\n");
            for (id, stat) in &stats.llm_provider_stats {
                report.push_str(&format!(
                    "  {} ({} / {}):\n",
                    id, stat.provider_name, stat.model
                ));
                report.push_str(&format!(
                    "    Calls:      {} (success: {}, failed: {})\n",
                    stat.total_calls, stat.successful_calls, stat.failed_calls
                ));
                report.push_str(&format!("    Characters: {}\n", stat.total_chars));
                report.push_str(&format!(
                    "    Latency:    avg {:.1}ms",
                    stat.average_latency_ms
                ));
                if let Some(min) = stat.min_latency_ms {
                    report.push_str(&format!(", min {:.1}ms", min));
                }
                if let Some(max) = stat.max_latency_ms {
                    report.push_str(&format!(", max {:.1}ms", max));
                }
                report.push('\n');
            }
            report.push('\n');
        }

        report.push_str(&format!("{}\n", "=".repeat(60)));

        Ok(report)
    }

    fn generate_json_report(&self, stats: &TranslationStats) -> Result<String, TranslateError> {
        serde_json::to_string_pretty(stats)
            .map_err(|e| TranslateError::Parse(format!("Failed to serialize JSON report: {}", e)))
    }

    fn calculate_translation_progress(&self, stats: &TranslationStats) -> f64 {
        if stats.total_units == 0 {
            0.0
        } else {
            (stats.translated_units as f64 / stats.total_units as f64) * 100.0
        }
    }

    fn calculate_cache_hit_rate(&self, stats: &TranslationStats) -> f64 {
        let total = stats.cache_hit_count + stats.cache_miss_count;
        if total == 0 {
            0.0
        } else {
            (stats.cache_hit_count as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_report_generator_new() {
        DefaultReportGenerator::new();
    }

    #[test]
    fn test_generate_json_report() {
        let generator = DefaultReportGenerator::new();
        let mut stats = TranslationStats::new();
        stats.total_files = 10;
        stats.processed_files = 5;
        stats.finalize();
        
        let result = generator.generate(&stats, ReportFormat::Json);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("total_files"));
        assert!(json.contains("processed_files"));
    }

    #[test]
    fn test_calculate_translation_progress() {
        let generator = DefaultReportGenerator::new();
        let mut stats = TranslationStats::new();
        stats.total_units = 100;
        stats.translated_units = 50;
        
        let progress = generator.calculate_translation_progress(&stats);
        assert_eq!(progress, 50.0);
    }

    #[test]
    fn test_calculate_cache_hit_rate() {
        let generator = DefaultReportGenerator::new();
        let mut stats = TranslationStats::new();
        stats.cache_hit_count = 80;
        stats.cache_miss_count = 20;
        
        let rate = generator.calculate_cache_hit_rate(&stats);
        assert_eq!(rate, 80.0);
    }
}
