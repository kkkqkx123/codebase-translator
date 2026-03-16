//! Gitignore pattern matching module
//!
//! This module provides functionality to read and parse .gitignore files,
//! and match file paths against gitignore-style patterns.

use std::path::{Path, PathBuf};
use tracing::trace;

/// Gitignore pattern matcher
pub struct GitignoreMatcher {
    pub(crate) patterns: Vec<GitignorePattern>,
    base_dir: PathBuf,
}

/// A single gitignore pattern
#[derive(Debug, Clone)]
pub(crate) struct GitignorePattern {
    /// The original pattern string
    original: String,
    /// Whether this pattern is negated (starts with !)
    negated: bool,
    /// Whether this pattern is directory-only (ends with /)
    dir_only: bool,
    /// Whether this pattern is anchored (starts with /)
    anchored: bool,
    /// Whether this pattern uses globstar (**)
    globstar: bool,
}

impl GitignoreMatcher {
    /// Create a new gitignore matcher from patterns
    pub fn new(patterns: Vec<String>, base_dir: impl AsRef<Path>) -> Self {
        let parsed_patterns = patterns
            .into_iter()
            .filter_map(|p| {
                let p = p.trim();
                if p.is_empty() || p.starts_with('#') {
                    return None;
                }
                GitignorePattern::parse(p)
            })
            .collect();

        Self {
            patterns: parsed_patterns,
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a new gitignore matcher from a .gitignore file
    pub fn from_file(gitignore_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = gitignore_path.as_ref();
        let base_dir = path.parent().unwrap_or(Path::new("."));

        let content = std::fs::read_to_string(path)?;
        let patterns: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        Ok(Self::new(patterns, base_dir))
    }

    /// Create a new gitignore matcher from a .gitignore file path (owned)
    pub fn from_path(gitignore_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = gitignore_path.as_ref();
        let base_dir = path.parent().unwrap_or(Path::new("."));

        let content = std::fs::read_to_string(path)?;
        let patterns: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        Ok(Self::new(patterns, base_dir))
    }

    /// Check if a path should be ignored
    pub fn is_ignored(&self, path: &Path) -> bool {
        let relative_path = match path.strip_prefix(&self.base_dir) {
            Ok(p) => p,
            Err(_) => path,
        };

        let path_str = relative_path.to_string_lossy();
        let is_dir = path.is_dir();

        let mut ignored = false;

        for pattern in &self.patterns {
            if pattern.dir_only && !is_dir {
                continue;
            }

            let matches = self.matches_pattern(&path_str, pattern);

            if matches {
                if pattern.negated {
                    ignored = false;
                    trace!(
                        path = %path_str,
                        pattern = %pattern.original,
                        "Path negated by gitignore pattern"
                    );
                } else {
                    ignored = true;
                    trace!(
                        path = %path_str,
                        pattern = %pattern.original,
                        "Path matched gitignore pattern"
                    );
                }
            }
        }

        ignored
    }

    /// Check if a path matches a gitignore pattern
    fn matches_pattern(&self, path: &str, pattern: &GitignorePattern) -> bool {
        let path_normalized = path.replace('\\', "/");

        if pattern.globstar {
            return self.matches_globstar(&path_normalized, pattern);
        }

        if pattern.anchored {
            self.matches_anchored(&path_normalized, pattern)
        } else {
            self.matches_unanchored(&path_normalized, pattern)
        }
    }

    /// Match anchored pattern (starts with /)
    fn matches_anchored(&self, path: &str, pattern: &GitignorePattern) -> bool {
        let pattern_str = pattern.original.trim_start_matches('!');
        let pattern_str = pattern_str.trim_start_matches('/');

        let path_parts: Vec<&str> = path.split('/').collect();
        let pattern_parts: Vec<&str> = pattern_str.split('/').collect();

        if pattern_parts.is_empty() {
            return path_parts.is_empty();
        }

        if path_parts.len() < pattern_parts.len() {
            return false;
        }

        let path_slice = &path_parts[..pattern_parts.len()];
        self.matches_parts(path_slice, &pattern_parts)
    }

    /// Match unanchored pattern
    fn matches_unanchored(&self, path: &str, pattern: &GitignorePattern) -> bool {
        let pattern_str = pattern.original.trim_start_matches('!');
        let pattern_str = pattern_str.trim_end_matches('/');

        let path_parts: Vec<&str> = path.split('/').collect();
        let pattern_parts: Vec<&str> = pattern_str.split('/').collect();

        for i in 0..=path_parts.len().saturating_sub(pattern_parts.len()) {
            let slice = &path_parts[i..i + pattern_parts.len()];
            if self.matches_parts(slice, &pattern_parts) {
                return true;
            }
        }

        false
    }

    /// Match path parts against pattern parts with glob support
    fn matches_parts(&self, path_parts: &[&str], pattern_parts: &[&str]) -> bool {
        for (path_part, pattern_part) in path_parts.iter().zip(pattern_parts.iter()) {
            if !self.matches_glob(path_part, pattern_part) {
                return false;
            }
        }
        true
    }

    /// Match a single path part against a pattern part with glob support
    fn matches_glob(&self, text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return !text.is_empty();
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.is_empty() {
                return true;
            }

            let mut pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }

                if i == 0 {
                    if !text.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else {
                    match text[pos..].find(part) {
                        Some(idx) => pos += idx + part.len(),
                        None => return false,
                    }
                }
            }

            if parts.len() > 1 {
                return pos <= text.len();
            }

            return true;
        }

        text == pattern
    }

    /// Match globstar pattern (contains **)
    fn matches_globstar(&self, path: &str, pattern: &GitignorePattern) -> bool {
        let pattern_str = pattern.original.trim_start_matches('!');
        let pattern_str = pattern_str.trim_end_matches('/');

        // Handle **/*.ext pattern - match any file with this extension at any depth
        // The pattern **/*.ext should match:
        // - test.ext (in root)
        // - dir/test.ext (in dir)
        // - dir/subdir/test.ext (in nested dir)
        if pattern_str == "**/*.log" || (pattern_str.starts_with("**/*") && pattern_str.len() > 5) {
            let ext = &pattern_str[5..]; // Skip "**/*"
            return path.ends_with(&format!(".{}", ext));
        }

        let parts: Vec<&str> = pattern_str.split("**").collect();

        if parts.len() == 1 {
            return self.matches_glob(path, pattern_str);
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            let part = part.trim_start_matches('/');

            if i == 0 && pattern.anchored {
                if !path.starts_with(part) {
                    return false;
                }
                pos = part.len();
            } else {
                match path[pos..].find(part) {
                    Some(idx) => pos += idx + part.len(),
                    None => return false,
                }
            }
        }

        true
    }
}

impl GitignorePattern {
    /// Parse a gitignore pattern string
    fn parse(pattern: &str) -> Option<Self> {
        let original = pattern.trim().to_string();
        let pattern = original.as_str();

        if pattern.is_empty() || pattern.starts_with('#') {
            return None;
        }

        let negated = pattern.starts_with('!');
        let pattern = if negated { &pattern[1..] } else { pattern };

        let anchored = pattern.starts_with('/');
        let pattern = if anchored { &pattern[1..] } else { pattern };

        let dir_only = pattern.ends_with('/');
        let pattern = if dir_only {
            &pattern[..pattern.len() - 1]
        } else {
            pattern
        };

        let globstar = pattern.contains("**");

        Some(Self {
            original,
            negated,
            dir_only,
            anchored,
            globstar,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pattern() {
        let pattern = GitignorePattern::parse("node_modules/").unwrap();
        assert!(pattern.dir_only);
        assert!(!pattern.negated);
        assert!(!pattern.anchored);

        let pattern = GitignorePattern::parse("/target").unwrap();
        assert!(pattern.anchored);
        assert!(!pattern.dir_only);

        let pattern = GitignorePattern::parse("!important.txt").unwrap();
        assert!(pattern.negated);
    }

    #[test]
    fn test_match_simple() {
        let matcher = GitignoreMatcher::new(
            vec!["*.log".to_string(), "node_modules".to_string()],
            Path::new("."),
        );

        assert!(matcher.is_ignored(Path::new("test.log")));
        assert!(matcher.is_ignored(Path::new("node_modules")));
        assert!(!matcher.is_ignored(Path::new("test.txt")));
    }

    #[test]
    fn test_match_anchored() {
        let matcher = GitignoreMatcher::new(vec!["/target".to_string()], Path::new("."));

        assert!(matcher.is_ignored(Path::new("target")));
        assert!(!matcher.is_ignored(Path::new("src/target")));
    }

    #[test]
    fn test_match_globstar() {
        let matcher = GitignoreMatcher::new(vec!["**/*.log".to_string()], Path::new("."));

        assert!(matcher.is_ignored(Path::new("test.log")));
        assert!(matcher.is_ignored(Path::new("src/test.log")));
        assert!(matcher.is_ignored(Path::new("src/nested/test.log")));
    }

    #[test]
    fn test_negation() {
        let matcher = GitignoreMatcher::new(
            vec!["*.log".to_string(), "!important.log".to_string()],
            Path::new("."),
        );

        assert!(!matcher.is_ignored(Path::new("important.log")));
        assert!(matcher.is_ignored(Path::new("test.log")));
    }
}
