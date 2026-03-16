//! Environment variable loading and expansion
//!
//! This module provides functionality for loading environment variables from .env files
//! and expanding environment variable placeholders in configuration values.

use crate::core::error::{Result, TranslateError};
use regex::Regex;
use std::path::Path;

/// Environment variable loader
pub struct EnvLoader {
    /// Environment files to load
    env_files: Vec<String>,
}

impl EnvLoader {
    /// Create a new environment loader
    ///
    /// # Arguments
    ///
    /// * `env_files` - List of .env files to load (later files override earlier ones)
    pub fn new(env_files: Vec<String>) -> Self {
        Self { env_files }
    }

    /// Create a new environment loader with default .env file
    pub fn with_default() -> Self {
        Self::new(vec![".env".to_string()])
    }

    /// Load environment files
    ///
    /// This loads environment variables from the specified files.
    /// Existing environment variables are not overridden.
    pub fn load(&self) -> Result<()> {
        for file in &self.env_files {
            if Path::new(file).exists() {
                dotenvy::from_path(file).map_err(|e| {
                    TranslateError::Config(format!("Failed to load .env file {}: {}", file, e))
                })?;
            }
        }
        Ok(())
    }

    /// Load environment files with override
    ///
    /// This loads environment variables from the specified files,
    /// overriding existing environment variables.
    pub fn load_with_override(&self) -> Result<()> {
        for file in &self.env_files {
            if Path::new(file).exists() {
                dotenvy::from_path_override(file).map_err(|e| {
                    TranslateError::Config(format!(
                        "Failed to load .env file {} with override: {}",
                        file, e
                    ))
                })?;
            }
        }
        Ok(())
    }

    /// Find .env files in the specified directory
    ///
    /// This searches for .env files in the directory, including:
    /// - .env
    /// - .env.* (except .env.example)
    ///
    /// Returns file names (not full paths).
    pub fn find_env_files_in_directory(dir: &Path) -> Vec<String> {
        let mut files = Vec::new();

        // First check for .env
        let env_file = dir.join(".env");
        if env_file.exists() {
            files.push(".env".to_string());
        }

        // Check for .env.* files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with(".env.") && !name.ends_with(".example") {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }

        files
    }

    /// Load environment files from directory
    ///
    /// This automatically finds and loads all .env files in the specified directory.
    pub fn load_from_directory(dir: &Path) -> Result<()> {
        let env_files = Self::find_env_files_in_directory(dir);
        if !env_files.is_empty() {
            let loader = Self::new(env_files);
            loader.load()?;
        }
        Ok(())
    }
}

impl Default for EnvLoader {
    fn default() -> Self {
        Self::with_default()
    }
}

/// Expand environment variable placeholders in a string
///
/// This function replaces environment variable placeholders with their values.
/// Supported formats:
/// - `${VAR_NAME}`
/// - `$VAR_NAME`
///
/// # Arguments
///
/// * `input` - The string to expand
///
/// # Returns
///
/// The expanded string with placeholders replaced by actual environment variable values.
/// If an environment variable is not set, the placeholder is kept as-is.
///
/// # Examples
///
/// ```
/// use codebase_translate::config::expand_env_vars;
/// std::env::set_var("TEST_VAR", "hello");
/// let result = expand_env_vars("${TEST_VAR} world");
/// assert_eq!(result, "hello world");
/// ```
pub fn expand_env_vars(input: &str) -> String {
    let re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").expect("Invalid regex pattern");
    re.replace_all(input, |caps: &regex::Captures| {
        let var_name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
    })
    .to_string()
}

/// Check if a string contains environment variable placeholders
///
/// # Arguments
///
/// * `input` - The string to check
///
/// # Returns
///
/// `true` if the string contains placeholders, `false` otherwise.
pub fn has_env_vars(input: &str) -> bool {
    let re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").expect("Invalid regex pattern");
    re.is_match(input)
}

/// Replace environment variable placeholders in a map
///
/// # Arguments
///
/// * `map` - The map to replace placeholders in
pub fn replace_env_vars_in_map(map: &mut std::collections::HashMap<String, String>) {
    for value in map.values_mut() {
        *value = expand_env_vars(value);
    }
}

/// Replace environment variable placeholders in a nested map
///
/// This function recursively processes nested maps and strings.
///
/// # Arguments
///
/// * `map` - The nested map to replace placeholders in
pub fn replace_env_vars_in_nested_map(
    map: &mut std::collections::HashMap<String, serde_json::Value>,
) {
    for value in map.values_mut() {
        match value {
            serde_json::Value::String(s) => {
                *s = expand_env_vars(s);
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj.iter() {
                    let mut new_v = v.clone();
                    match &mut new_v {
                        serde_json::Value::String(s) => {
                            *s = expand_env_vars(s);
                        }
                        serde_json::Value::Object(inner_obj) => {
                            let mut inner_map: std::collections::HashMap<
                                String,
                                serde_json::Value,
                            > = inner_obj
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            replace_env_vars_in_nested_map(&mut inner_map);
                            *inner_obj = serde_json::Map::from_iter(
                                inner_map.into_iter().map(|(k, v)| (k, v)),
                            );
                        }
                        _ => {}
                    }
                    new_obj.insert(k.clone(), new_v);
                }
                *value = serde_json::Value::Object(new_obj);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars() {
        std::env::set_var("TEST_VAR", "hello");
        std::env::set_var("ANOTHER_VAR", "world");

        assert_eq!(expand_env_vars("${TEST_VAR}"), "hello");
        assert_eq!(expand_env_vars("$TEST_VAR"), "hello");
        assert_eq!(expand_env_vars("${TEST_VAR} ${ANOTHER_VAR}"), "hello world");
        assert_eq!(expand_env_vars("$TEST_VAR $ANOTHER_VAR"), "hello world");
        assert_eq!(expand_env_vars("${MISSING_VAR}"), "${MISSING_VAR}");
        assert_eq!(expand_env_vars("$MISSING_VAR"), "$MISSING_VAR");
    }

    #[test]
    fn test_has_env_vars() {
        assert!(has_env_vars("${VAR}"));
        assert!(has_env_vars("$VAR"));
        assert!(has_env_vars("text ${VAR} text"));
        assert!(!has_env_vars("no vars here"));
    }

    #[test]
    fn test_expand_nested_placeholders() {
        std::env::set_var("PATH_VAR", "/usr/bin");
        assert_eq!(expand_env_vars("${PATH_VAR}/node"), "/usr/bin/node");
    }

    #[test]
    fn test_replace_env_vars_in_map() {
        std::env::set_var("KEY1", "value1");
        std::env::set_var("KEY2", "value2");

        let mut map = std::collections::HashMap::new();
        map.insert("url".to_string(), "${KEY1}/api".to_string());
        map.insert("path".to_string(), "/${KEY2}/data".to_string());

        replace_env_vars_in_map(&mut map);

        assert_eq!(map.get("url").unwrap(), "value1/api");
        assert_eq!(map.get("path").unwrap(), "/value2/data");
    }

    #[test]
    fn test_replace_env_vars_in_nested_map() {
        std::env::set_var("API_URL", "https://api.example.com");
        std::env::set_var("API_KEY", "secret-key");

        let mut map = std::collections::HashMap::new();
        map.insert(
            "config".to_string(),
            serde_json::json!({
                "url": "${API_URL}",
                "auth": {
                    "key": "${API_KEY}"
                }
            }),
        );

        replace_env_vars_in_nested_map(&mut map);

        let config = map.get("config").unwrap();
        if let serde_json::Value::Object(obj) = config {
            assert_eq!(
                obj.get("url").unwrap(),
                &serde_json::Value::String("https://api.example.com".to_string())
            );
            if let serde_json::Value::Object(auth) = obj.get("auth").unwrap() {
                assert_eq!(
                    auth.get("key").unwrap(),
                    &serde_json::Value::String("secret-key".to_string())
                );
            }
        }
    }

    #[test]
    fn test_env_loader() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let env_file = temp_dir.path().join(".env");

        std::fs::write(&env_file, "TEST_VAR=hello\nANOTHER_VAR=world\n# Comment\n")
            .expect("Failed to write env file");

        let loader = EnvLoader::new(vec![env_file.to_string_lossy().to_string()]);
        loader.load().expect("Failed to load env file");

        assert_eq!(std::env::var("TEST_VAR").unwrap(), "hello");
        assert_eq!(std::env::var("ANOTHER_VAR").unwrap(), "world");
    }

    #[test]
    fn test_find_env_files() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let env_file = temp_dir.path().join(".env");
        let env_local = temp_dir.path().join(".env.local");
        let env_example = temp_dir.path().join(".env.example");

        std::fs::write(&env_file, "TEST=1").expect("Failed to write .env");
        std::fs::write(&env_local, "TEST=2").expect("Failed to write .env.local");
        std::fs::write(&env_example, "TEST=3").expect("Failed to write .env.example");

        let files = EnvLoader::find_env_files_in_directory(temp_dir.path());

        assert_eq!(files.len(), 2);
        assert!(files.contains(&".env".to_string()));
        assert!(files.contains(&".env.local".to_string()));
        assert!(!files.contains(&".env.example".to_string()));
    }
}
