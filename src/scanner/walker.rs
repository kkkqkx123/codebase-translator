//! Directory walker and file system scanner

use std::path::Path;
use tracing::{debug, info};

use crate::core::error::Result;
use crate::core::models::FileEntry;
use crate::scanner::gitignore::GitignoreMatcher;
use crate::scanner::r#trait::{ScanOptions, Scanner};

/// File system scanner implementation
pub struct FSScanner;

impl FSScanner {
    pub fn new() -> Self {
        Self
    }

    fn load_gitignore(&self, opts: &ScanOptions) -> Option<GitignoreMatcher> {
        if !opts.respect_gitignore {
            return None;
        }

        let gitignore_path = if let Some(path) = &opts.gitignore_path {
            path.clone()
        } else {
            Path::new(&opts.root_path).join(".gitignore")
        };

        let path_display = gitignore_path.display().to_string();

        if !gitignore_path.exists() {
            debug!(path = %path_display, ".gitignore not found");
            return None;
        }

        match GitignoreMatcher::from_path(&gitignore_path) {
            Ok(matcher) => {
                debug!(
                    path = %path_display,
                    patterns_count = matcher.patterns.len(),
                    "Loaded .gitignore"
                );
                Some(matcher)
            }
            Err(e) => {
                debug!(
                    path = %path_display,
                    error = %e,
                    "Failed to load .gitignore"
                );
                None
            }
        }
    }

    fn scan_directory(
        &self,
        dir: &Path,
        opts: &ScanOptions,
        entries: &mut Vec<FileEntry>,
        gitignore: &Option<GitignoreMatcher>,
    ) -> Result<()> {
        let abs_dir = dir.canonicalize().map_err(|e| {
            crate::core::error::TranslateError::Io(format!(
                "failed to get absolute path for {}: {}",
                dir.display(),
                e
            ))
        })?;

        info!("Starting directory scan: {}", abs_dir.display());

        let mut file_count = 0;
        self.walk_dir(&abs_dir, opts, entries, &mut file_count, gitignore)?;

        debug!(
            directory = %abs_dir.display(),
            files_found = file_count,
            "Scan completed"
        );

        Ok(())
    }

    fn walk_dir(
        &self,
        dir: &Path,
        opts: &ScanOptions,
        entries: &mut Vec<FileEntry>,
        file_count: &mut usize,
        gitignore: &Option<GitignoreMatcher>,
    ) -> Result<()> {
        let entries_iter = std::fs::read_dir(dir).map_err(|e| {
            crate::core::error::TranslateError::Io(format!(
                "failed to read directory {}: {}",
                dir.display(),
                e
            ))
        })?;

        for entry in entries_iter {
            let entry = entry.map_err(|e| {
                crate::core::error::TranslateError::Io(format!(
                    "error accessing path in {}: {}",
                    dir.display(),
                    e
                ))
            })?;

            let path = entry.path();

            let file_type = entry.file_type().map_err(|e| {
                crate::core::error::TranslateError::Io(format!(
                    "failed to get file type for {}: {}",
                    path.display(),
                    e
                ))
            })?;

            if file_type.is_dir() {
                if self.should_exclude_dir(&path, &opts.exclude_patterns, gitignore) {
                    debug!(dir = %path.display(), "Skipping excluded directory");
                    continue;
                }
                self.walk_dir(&path, opts, entries, file_count, gitignore)?;
            } else if file_type.is_file() {
                if !self.should_include_file(
                    &path,
                    &opts.include_patterns,
                    &opts.exclude_patterns,
                    gitignore,
                ) {
                    continue;
                }

                let metadata = entry.metadata().map_err(|e| {
                    crate::core::error::TranslateError::Io(format!(
                        "failed to get file metadata for {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                let relative_path = path
                    .strip_prefix(&opts.root_path)
                    .unwrap_or(&path)
                    .to_path_buf();

                let file_entry = FileEntry {
                    path: path.clone(),
                    relative_path,
                    size: metadata.len(),
                    modified: metadata.modified().map_err(|e| {
                        crate::core::error::TranslateError::Io(format!(
                            "failed to get modified time for {}: {}",
                            path.display(),
                            e
                        ))
                    })?,
                };

                debug!(
                    path = %path.display(),
                    size = file_entry.size,
                    "Found file"
                );

                entries.push(file_entry);
                *file_count += 1;
            } else if file_type.is_symlink() && !opts.follow_symlinks {
                debug!(path = %path.display(), "Skipping symlink");
            }
        }

        Ok(())
    }

    fn should_include_file(
        &self,
        path: &Path,
        include_patterns: &[String],
        exclude_patterns: &[String],
        gitignore: &Option<GitignoreMatcher>,
    ) -> bool {
        if !include_patterns.is_empty() {
            let matched = include_patterns
                .iter()
                .any(|pattern| self.match_pattern(path, pattern));

            if !matched {
                return false;
            }
        }

        if exclude_patterns
            .iter()
            .any(|pattern| self.match_pattern(path, pattern))
        {
            return false;
        }

        if let Some(matcher) = gitignore {
            if matcher.is_ignored(path) {
                return false;
            }
        }

        true
    }

    fn should_exclude_dir(
        &self,
        path: &Path,
        exclude_patterns: &[String],
        gitignore: &Option<GitignoreMatcher>,
    ) -> bool {
        if exclude_patterns
            .iter()
            .any(|pattern| self.match_pattern(path, pattern))
        {
            return true;
        }

        if let Some(matcher) = gitignore {
            if matcher.is_ignored(path) {
                return true;
            }
        }

        false
    }

    fn match_pattern(&self, path: &Path, pattern: &str) -> bool {
        let path_str = path.to_string_lossy();

        if pattern.contains("**") {
            return self.match_recursive_pattern(&path_str, pattern);
        }

        glob::Pattern::new(pattern)
            .map(|p| p.matches_path(path))
            .unwrap_or(false)
    }

    fn match_recursive_pattern(&self, path: &str, pattern: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split("**").collect();

        if pattern_parts.len() == 1 {
            return path == pattern;
        }

        let prefix = pattern_parts[0].trim_end_matches('/');
        let suffix = pattern_parts[1].trim_start_matches('/');

        if !prefix.is_empty() && !path.starts_with(prefix) {
            return false;
        }

        if !suffix.is_empty() {
            if suffix.contains('*') {
                let suffix_parts: Vec<&str> = suffix.split('*').collect();
                if suffix_parts.len() == 2 {
                    let suffix_prefix = suffix_parts[0];
                    let suffix_suffix = suffix_parts[1];
                    let filename = path.rsplit('/').next().unwrap_or(path);
                    return filename.starts_with(suffix_prefix)
                        && filename.ends_with(suffix_suffix);
                }
            }
            return path.ends_with(suffix);
        }

        true
    }
}

impl Default for FSScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner for FSScanner {
    fn scan(&self, opts: ScanOptions) -> Result<Vec<FileEntry>> {
        let root = Path::new(&opts.root_path);
        let abs_root = root.canonicalize().map_err(|e| {
            crate::core::error::TranslateError::Io(format!(
                "failed to get absolute path for {}: {}",
                opts.root_path, e
            ))
        })?;

        if !abs_root.exists() {
            return Err(crate::core::error::TranslateError::NotFound(format!(
                "directory does not exist: {}",
                abs_root.display()
            )));
        }

        if !abs_root.is_dir() {
            return Err(crate::core::error::TranslateError::InvalidArgument(
                format!("path is not a directory: {}", abs_root.display()),
            ));
        }

        let gitignore = self.load_gitignore(&opts);

        let mut entries = Vec::new();
        self.scan_directory(&abs_root, &opts, &mut entries, &gitignore)?;
        Ok(entries)
    }
}
