//! Cache utility functions

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Generate project directory fingerprint
///
/// Combines directory path and stable metadata to identify directory
pub fn generate_project_fingerprint(
    project_dir: &Path,
) -> Result<String, crate::core::error::TranslateError> {
    let abs_path = project_dir.canonicalize().map_err(|e| {
        crate::core::error::TranslateError::Cache(format!("Failed to canonicalize path: {}", e))
    })?;

    let mut features = Vec::new();
    features.push(abs_path.to_string_lossy().to_string());

    // Use directory metadata instead of file modification time for stability
    if let Ok(metadata) = fs::metadata(project_dir) {
        if let Ok(created) = metadata.created() {
            features.push(
                created
                    .duration_since(std::time::UNIX_EPOCH)
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

    // Use hash to generate a stable project ID
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let hash = hasher.finalize();

    // Use first 16 characters of hash as project ID
    hex::encode(hash)[..16].to_string()
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

    let cache_dir = PathBuf::from(base_dir).join("translator").join("cache");

    // Ensure the cache directory exists
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        tracing::warn!(
            "Failed to create global cache directory {}: {}",
            cache_dir.display(),
            e
        );
    }

    cache_dir
}

/// Resolve cache directory path based on cache mode
pub fn resolve_cache_dir(
    cache_mode: &crate::core::models::CacheMode,
    project_dir: &Path,
) -> PathBuf {
    match cache_mode {
        crate::core::models::CacheMode::Global => {
            let global_dir = get_global_cache_dir();
            let project_id = generate_project_id(project_dir);
            global_dir.join(project_id).join("translator")
        }
        crate::core::models::CacheMode::Local => project_dir.join(".translator").join("cache"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_project_id() {
        let path = Path::new("/home/user/project");
        let id = generate_project_id(path);
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_alphanumeric()));

        // Windows path
        let path = Path::new("C:\\Users\\user\\project");
        let id = generate_project_id(path);
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_alphanumeric()));

        // Same path should generate same ID
        let path1 = Path::new("/home/user/project");
        let path2 = Path::new("/home/user/project");
        let id1 = generate_project_id(path1);
        let id2 = generate_project_id(path2);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_global_cache_dir() {
        let dir = get_global_cache_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("translator") || dir_str.contains("translator\\cache"));
    }
}
