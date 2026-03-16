//! Cache utility functions

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Generate project directory fingerprint
///
/// Combines directory path, creation time and other info to identify directory reconstruction
pub fn generate_project_fingerprint(
    project_dir: &Path,
) -> Result<String, crate::core::error::TranslateError> {
    let abs_path = project_dir.canonicalize().map_err(|e| {
        crate::core::error::TranslateError::Cache(format!("Failed to canonicalize path: {}", e))
    })?;

    let mut features = Vec::new();
    features.push(abs_path.to_string_lossy().to_string());

    // Get oldest file modification time
    if let Ok(entries) = fs::read_dir(project_dir) {
        let mut oldest_time: Option<std::time::SystemTime> = None;

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if oldest_time.is_none() || modified < oldest_time.unwrap() {
                        oldest_time = Some(modified);
                    }
                }
            }
        }

        if let Some(time) = oldest_time {
            features.push(
                time.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_default(),
            );
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(features.join("|"));
    let hash = hasher.finalize();
    Ok(hex::encode(hash)[..16].to_string())
}

/// Generate project ID from project path
///
/// Used for project isolation in global cache mode
pub fn generate_project_id(project_dir: &Path) -> String {
    let abs_path = project_dir
        .canonicalize()
        .unwrap_or_else(|_| project_dir.to_path_buf());

    let path_str = abs_path.to_string_lossy().to_string();
    let path_str = path_str.replace('\\', "/");

    let mut result = String::from("-");

    // Handle Windows drive letters
    if path_str.contains(':') {
        if let Some(idx) = path_str.find(':') {
            let drive = path_str[..idx].to_lowercase();
            result.push_str(&drive);
            result.push('-');
            let rest = &path_str[idx + 1..];
            let rest = rest.trim_start_matches('/');
            result.push_str(rest);
        }
    } else {
        let rest = path_str.trim_start_matches('/');
        result.push_str(rest);
    }

    // Replace special characters
    result = result.replace('/', "-").replace(' ', "-").to_lowercase();
    result
}

/// Get global cache directory
///
/// Returns platform-specific cache directory for storing global cache
pub fn get_global_cache_dir() -> PathBuf {
    let base_dir = if cfg!(windows) {
        env::var("LOCALAPPDATA")
            .or_else(|_| env::var("APPDATA"))
            .unwrap_or_else(|_| {
                env::var("USERPROFILE")
                    .map(|p| {
                        PathBuf::from(p)
                            .join("AppData")
                            .join("Local")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_else(|_| ".".to_string())
            })
    } else if cfg!(target_os = "macos") {
        env::var("HOME")
            .map(|p| {
                PathBuf::from(p)
                    .join("Library")
                    .join("Caches")
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|_| "/tmp".to_string())
    } else {
        // Linux/Unix
        env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
            env::var("HOME")
                .map(|p| {
                    PathBuf::from(p)
                        .join(".cache")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_else(|_| "/tmp".to_string())
        })
    };

    PathBuf::from(base_dir).join("translator").join("cache")
}

/// Resolve cache directory path based on cache mode
pub fn resolve_cache_dir(
    cache_mode: &crate::core::models::CacheMode,
    cache_directory: &str,
    project_dir: &Path,
) -> PathBuf {
    match cache_mode {
        crate::core::models::CacheMode::Global => {
            let global_dir = get_global_cache_dir();
            let project_id = generate_project_id(project_dir);
            global_dir.join(project_id).join(cache_directory)
        }
        crate::core::models::CacheMode::Local => project_dir.join(cache_directory),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_project_id() {
        let path = Path::new("/home/user/project");
        let id = generate_project_id(path);
        assert!(id.starts_with('-'));
        assert!(!id.contains('/'));

        // Windows path
        let path = Path::new("C:\\Users\\user\\project");
        let id = generate_project_id(path);
        assert!(id.starts_with('-'));
        assert!(id.contains("c-"));
        assert!(!id.contains('\\'));
    }

    #[test]
    fn test_get_global_cache_dir() {
        let dir = get_global_cache_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("translator") || dir_str.contains("translator\\cache"));
    }
}
