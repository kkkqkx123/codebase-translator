use clap::Parser;
use tracing::info;

use crate::{
    commands::Command,
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::{Result, TranslateError},
    core::models::FileEntry,
    scanner::{FSScanner, ScanOptions, Scanner},
};

/// Detect language script content in files and generate a report
#[derive(Parser, Debug)]
pub struct DetectArgs {
    /// Path to file or directory to detect
    #[arg(value_name = "PATH")]
    path: String,

    /// Language family/script to detect (cjk, cyrillic, latin, arabic, hebrew, greek)
    #[arg(short, long)]
    language: Option<String>,

    /// Output file for the report (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Verbose mode: show line-by-line details
    #[arg(long)]
    verbose: bool,

    /// Include patterns (glob)
    #[arg(long)]
    include: Vec<String>,

    /// Exclude patterns (glob)
    #[arg(long)]
    exclude: Vec<String>,

    /// Respect .gitignore
    #[arg(long, default_value = "true")]
    respect_gitignore: bool,
}

impl Command for DetectArgs {
    fn execute(
        &self,
        _global_config: &GlobalConfig,
        _project_config: &ProjectConfig,
    ) -> Result<()> {
        let scanner = FSScanner::new();

        let target_script = self.parse_target_script()?;

        info!(
            path = %self.path,
            language = ?self.language,
            verbose = self.verbose,
            "Starting language script detection"
        );

        // Scan files
        let opts = ScanOptions {
            root_path: self.path.clone(),
            include_patterns: self.include.clone(),
            exclude_patterns: self.exclude.clone(),
            respect_gitignore: self.respect_gitignore,
            ..Default::default()
        };

        let files = scanner.scan(opts)?;

        if files.is_empty() {
            info!("No files found to scan");
            return Ok(());
        }

        let mut report = LanguageDetectionReport {
            detection_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            target_path: self.path.clone(),
            target_script: target_script.clone(),
            ..Default::default()
        };

        for file in &files {
            self.detect_file(file, &target_script, &mut report)?;
        }

        // Generate report
        let report_text = self.generate_report(&report)?;

        match &self.output {
            Some(path) => {
                std::fs::write(path, report_text)?;
                info!("Report saved to: {}", path);
            }
            None => {
                println!("{}", report_text);
            }
        }

        info!(
            files_scanned = report.total_files,
            lines_scanned = report.total_lines,
            matching_lines = report.matching_lines,
            "Language script detection completed"
        );

        Ok(())
    }
}

impl DetectArgs {
    fn parse_target_script(&self) -> Result<String> {
        match self.language.as_deref() {
            Some(lang) => {
                let lang_upper = lang.to_uppercase();
                match lang_upper.as_str() {
                    "CJK" | "CYRILLIC" | "LATIN" | "ARABIC" | "HEBREW" | "GREEK" => Ok(lang_upper),
                    _ => Err(TranslateError::InvalidArgument(format!(
                        "Unsupported language family: '{}'. Supported families: cjk, cyrillic, latin, arabic, hebrew, greek",
                        lang
                    ))),
                }
            }
            None => {
                // Default to CJK if no language specified
                Ok("CJK".to_string())
            }
        }
    }

    fn detect_file(
        &self,
        file: &FileEntry,
        target_script: &str,
        report: &mut LanguageDetectionReport,
    ) -> Result<()> {
        let content = std::fs::read_to_string(&file.path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut matching_lines: Vec<(usize, String)> = Vec::new();
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            if self.matches_script(line, target_script) {
                matching_lines.push((line_num, line.to_string()));
            }
        }

        // Merge consecutive matching lines into segments
        let segments = self.merge_consecutive_lines(&matching_lines);

        // Add segments to report
        if !segments.is_empty() {
            let segment_count = segments.len();
            report.file_results.push(FileDetectionResult {
                file_path: file.relative_path.display().to_string(),
                script_family: target_script.to_string(),
                segments,
                total_matching_lines: matching_lines.len(),
            });
            report.matching_lines += matching_lines.len();
            report.matching_segments += segment_count;
        }

        report.total_files += 1;
        report.total_lines += lines.len();

        Ok(())
    }

    fn matches_script(&self, text: &str, target_script: &str) -> bool {
        match target_script {
            "CJK" => contains_cjk(text),
            "CYRILLIC" => contains_cyrillic(text),
            "LATIN" => contains_latin(text),
            "ARABIC" => contains_arabic(text),
            "HEBREW" => contains_hebrew(text),
            "GREEK" => contains_greek(text),
            _ => false,
        }
    }

    fn merge_consecutive_lines(&self, matching_lines: &[(usize, String)]) -> Vec<Segment> {
        if matching_lines.is_empty() {
            return Vec::new();
        }

        let mut segments: Vec<Segment> = Vec::new();
        let mut current_start = matching_lines[0].0;
        let mut current_end = matching_lines[0].0;
        let mut current_lines: Vec<(usize, String)> = vec![matching_lines[0].clone()];

        for (line_num, line) in matching_lines.iter().skip(1) {
            // Check if this line is consecutive to the previous one
            if *line_num == current_end + 1 {
                current_end = *line_num;
                current_lines.push((*line_num, line.clone()));
            } else {
                // Add the current segment to results
                segments.push(Segment {
                    start_line: current_start,
                    end_line: current_end,
                    lines: current_lines.clone(),
                });

                // Start a new segment
                current_start = *line_num;
                current_end = *line_num;
                current_lines = vec![(*line_num, line.clone())];
            }
        }

        // Add the last segment
        if !current_lines.is_empty() {
            segments.push(Segment {
                start_line: current_start,
                end_line: current_end,
                lines: current_lines,
            });
        }

        segments
    }

    fn generate_report(&self, report: &LanguageDetectionReport) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!("{}\n", "=".repeat(80)));
        output.push_str("Language Detection Report\n");
        output.push_str(&format!("{}\n\n", "=".repeat(80)));

        // Summary
        output.push_str("Summary:\n");
        output.push_str(&format!("  Detection Time:    {}\n", report.detection_time));
        output.push_str(&format!("  Target Path:       {}\n", report.target_path));
        output.push_str(&format!("  Total Files:       {}\n", report.total_files));
        output.push_str(&format!("  Total Lines:       {}\n", report.total_lines));
        output.push_str(&format!("  Matching Lines:    {}\n", report.matching_lines));
        output.push_str(&format!(
            "  Matching Segments: {}\n",
            report.matching_segments
        ));
        output.push_str(&format!("  Target Script:     {}\n", report.target_script));
        output.push_str(&format!("{}\n\n", "-".repeat(80)));

        // Detection notice
        output.push_str("NOTE: Detection is based on Unicode script/language family.\n");
        output.push_str("      Specific language identification is not guaranteed.\n");
        output.push_str(&format!("{}\n\n", "-".repeat(80)));

        // Results
        output.push_str("Detection Results:\n");
        output.push_str(&format!("{}\n\n", "-".repeat(80)));

        if report.file_results.is_empty() {
            output.push_str("No matching content found.\n\n");
        } else {
            for file_result in &report.file_results {
                output.push_str(&format!("File: {}\n", file_result.file_path));

                for (segment_idx, segment) in file_result.segments.iter().enumerate() {
                    output.push_str(&format!(
                        "  Segment {} (Lines {}-{}):\n",
                        segment_idx + 1,
                        segment.start_line,
                        segment.end_line
                    ));

                    // Show preview (first few lines of segment)
                    if self.verbose {
                        // In verbose mode, show all lines
                        for (line_num, line) in &segment.lines {
                            output.push_str(&format!("    {:>4}: {}\n", line_num, line));
                        }
                    } else {
                        // In normal mode, show preview (first 3 lines or less)
                        let preview_lines: Vec<_> = segment.lines.iter().take(3).collect();
                        for (_line_num, line) in preview_lines {
                            output.push_str(&format!("    {}\n", truncate_string(line, 80)));
                        }
                        if segment.lines.len() > 3 {
                            output.push_str(&format!(
                                "    ... ({} more lines)\n",
                                segment.lines.len() - 3
                            ));
                        }
                    }

                    output.push('\n');
                }

                output.push_str(&format!(
                    "  Total: {} matching line(s)\n",
                    file_result.total_matching_lines
                ));
                output.push_str(&format!("{}\n\n", "-".repeat(80)));
            }
        }

        output.push_str(&format!("{}\n", "=".repeat(80)));

        Ok(output)
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

// Report structures

#[derive(Debug, Clone, Default)]
pub struct LanguageDetectionReport {
    pub detection_time: String,
    pub target_path: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub matching_lines: usize,
    pub matching_segments: usize,
    pub target_script: String,
    pub file_results: Vec<FileDetectionResult>,
}

#[derive(Debug, Clone)]
pub struct FileDetectionResult {
    pub file_path: String,
    pub script_family: String,
    pub segments: Vec<Segment>,
    pub total_matching_lines: usize,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub start_line: usize,
    pub end_line: usize,
    pub lines: Vec<(usize, String)>,
}

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x3400..=0x4DBF).contains(&cp)
            || (0x4E00..=0x9FFF).contains(&cp)
            || (0xF900..=0xFAFF).contains(&cp)
            || (0x3040..=0x309F).contains(&cp)
            || (0x30A0..=0x30FF).contains(&cp)
            || (0xAC00..=0xD7AF).contains(&cp)
            || (0x1100..=0x11FF).contains(&cp)
            || (0x2E80..=0x2EFF).contains(&cp)
            || (0x2F00..=0x2FDF).contains(&cp)
            || (0x31C0..=0x31EF).contains(&cp)
            || (0x20000..=0x2EBEF).contains(&cp)
            || (0x2F800..=0x2FA1F).contains(&cp)
    })
}

fn contains_cyrillic(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x0400..=0x04FF).contains(&cp) || (0x0500..=0x052F).contains(&cp)
    })
}

fn contains_latin(text: &str) -> bool {
    text.chars()
        .any(|c| c.is_ascii_alphabetic() || ((0x00C0..=0x024F).contains(&(c as u32))))
}

fn contains_arabic(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x0600..=0x06FF).contains(&cp) || (0x0750..=0x077F).contains(&cp)
    })
}

fn contains_hebrew(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x0590..=0x05FF).contains(&cp)
    })
}

fn contains_greek(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x0370..=0x03FF).contains(&cp) || (0x1F00..=0x1FFF).contains(&cp)
    })
}
