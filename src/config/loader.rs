use std::path::{Path, PathBuf};

use crate::core::error::{Result, TranslateError};

use super::{env::EnvLoader, global::GlobalConfig, project::ProjectConfig};

/// Configuration loader
pub struct ConfigLoader {
    /// Global config file path
    global_config_path: Option<PathBuf>,
    /// Project config file path
    project_config_path: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new config loader
    pub fn new() -> Self {
        Self {
            global_config_path: None,
            project_config_path: None,
        }
    }

    /// Set global config path
    pub fn with_global_config<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.global_config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set project config path
    pub fn with_project_config<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.project_config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Load global configuration
    ///
    /// Configuration priority (from high to low):
    /// 1. System environment variables
    /// 2. Environment variables from .env files
    /// 3. Values from global config file (supports ${VAR} placeholders)
    /// 4. Default values
    ///
    /// Config file search paths (by priority):
    /// 1. Directory specified by TRANSLATOR_CONFIG_HOME environment variable
    /// 2. Executable directory
    /// 3. Current working directory
    /// 4. User config directory
    pub fn load_global(&self) -> Result<GlobalConfig> {
        let path = self
            .global_config_path
            .clone()
            .or_else(Self::find_global_config_path)
            .ok_or_else(|| {
                TranslateError::Config("Could not determine global config path".to_string())
            })?;

        let mut config = GlobalConfig::default();

        // Step 1: Load .env files first to ensure environment variables are set before expanding config
        let env_files = Self::find_env_files();
        if !env_files.is_empty() {
            for env_file in &env_files {
                let env_loader = EnvLoader::new(vec![env_file.clone()]);
                if let Err(e) = env_loader.load() {
                    tracing::warn!("Failed to load .env file {}: {}", env_file, e);
                }
            }
        }

        // Step 2: Load config file if it exists
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let expanded_content = super::env::expand_env_vars(&content);
            let file_config: GlobalConfig = toml::from_str(&expanded_content).map_err(|e| {
                TranslateError::Config(format!("Failed to parse config file: {}", e))
            })?;
            config.merge(file_config);
        }

        // Step 3: Expand environment variable placeholders in config (e.g., ${VAR})
        // This must be done before ApplyEnvVars, as config file uses placeholder syntax
        config.expand_env_vars();

        // Step 4: Apply environment variable overrides (supported env vars directly override config values)
        config.apply_env_vars();

        // Step 5: Validate configuration
        if let Err(e) = config.validate() {
            return Err(TranslateError::Config(format!(
                "Configuration validation failed: {}",
                e
            )));
        }

        Ok(config)
    }

    /// Load project configuration
    pub fn load_project(&self) -> Result<ProjectConfig> {
        let path = self
            .project_config_path
            .clone()
            .or_else(|| Self::find_project_config(std::env::current_dir().ok()?.as_path()))
            .ok_or_else(|| {
                TranslateError::Config("Could not find project config file".to_string())
            })?;

        if !path.exists() {
            return Ok(ProjectConfig::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let expanded_content = super::env::expand_env_vars(&content);
        let mut config: ProjectConfig = toml::from_str(&expanded_content)?;
        config.normalize_patterns();
        if let Err(e) = config.validate() {
            return Err(TranslateError::Config(format!(
                "Project config validation failed: {}",
                e
            )));
        }
        Ok(config)
    }

    /// Load both global and project configs
    ///
    /// Configuration priority (from high to low):
    /// 1. Environment variables
    /// 2. ProjectConfig.logging
    /// 3. GlobalConfig.logging
    /// 4. Default values
    pub fn load(&self) -> Result<(GlobalConfig, ProjectConfig)> {
        let mut global = self.load_global()?;
        let project = self.load_project()?;

        // Merge logging configuration: project config overrides global config
        if let Some(ref project_logging) = project.logging {
            global.logging = project_logging.clone();
        }

        // Apply environment variable overrides (highest priority)
        global.apply_env_vars();

        Ok((global, project))
    }

    /// Find project config by searching up the directory tree
    fn find_project_config(start_dir: &Path) -> Option<PathBuf> {
        let config_names = [".translator.toml"];

        let mut current = Some(start_dir);
        while let Some(dir) = current {
            for name in &config_names {
                let path = dir.join(name);
                if path.exists() {
                    return Some(path);
                }
            }
            current = dir.parent();
        }

        None
    }

    /// Get global config search paths (by priority)
    fn get_global_config_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Directory specified by TRANSLATOR_CONFIG_HOME environment variable
        if let Ok(config_home) = std::env::var("TRANSLATOR_CONFIG_HOME") {
            paths.push(PathBuf::from(config_home.clone()).join("config.toml"));
            paths.push(PathBuf::from(config_home).join("translator.toml"));
        }

        // 2. Executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths.push(exe_dir.join("config.toml"));
                paths.push(exe_dir.join("translator.toml"));
            }
        }

        // 3. Current working directory
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("config.toml"));
            paths.push(cwd.join("translator.toml"));
        }

        // 4. User config directory
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("codebase-translator").join("config.toml"));
            paths.push(
                config_dir
                    .join("codebase-translator")
                    .join("translator.toml"),
            );
        }

        paths
    }

    /// Find global config path by searching in priority order
    /// Find global config path by searching in priority order
    pub fn find_global_config_path() -> Option<PathBuf> {
        for path in Self::get_global_config_search_paths() {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Get .env file search paths (by priority)
    fn get_env_file_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths.push(exe_dir.to_path_buf());
            }
        }

        // 2. Current working directory
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd);
        }

        // 3. Global config file directory
        if let Some(global_path) = Self::find_global_config_path() {
            if let Some(global_dir) = global_path.parent() {
                paths.push(global_dir.to_path_buf());
            }
        }

        paths
    }

    /// Find all .env files in search paths
    fn find_env_files() -> Vec<String> {
        let mut files = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for dir in Self::get_env_file_search_paths() {
            if seen.contains(&dir) {
                continue;
            }
            seen.insert(dir.clone());

            let env_file = dir.join(".env");
            if env_file.exists() {
                files.push(env_file.to_string_lossy().to_string());
            }
        }

        files
    }

    /// Save global configuration to a specific path
    ///
    /// NOTE: This method does NOT check if the file already exists.
    /// The caller is responsible for checking and confirming overwrite.
    pub fn save_global(&self, config: &GlobalConfig, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save project configuration
    pub fn save_project(&self, config: &ProjectConfig, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_project_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join(".translator.toml");

        let config_content = r#"
[translate]
target_lang = "zh"
source_langs = ["en"]

[cache]
enabled = true
directory = ".translator"
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config file");

        let loader = ConfigLoader::new().with_project_config(&config_path);
        let config = loader
            .load_project()
            .expect("Failed to load project config");

        assert_eq!(config.translate.target_lang, "zh");
        assert_eq!(config.translate.source_langs, vec!["en"]);
        assert!(config.cache.enabled);
        assert_eq!(config.cache.directory, ".translator");
    }

    #[test]
    fn test_default_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");

        // Create an empty .translator.toml file to avoid searching up the directory tree
        let config_path = temp_dir.path().join(".translator.toml");
        std::fs::write(&config_path, "").expect("Failed to create empty config file");

        let loader = ConfigLoader::new();
        let config = loader
            .load_project()
            .expect("Failed to load project config");

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore current dir");

        assert_eq!(config.translate.target_lang, "en");
        assert!(!config.writer.dry_run);
        assert!(config.writer.backup);
    }

    #[test]
    fn test_find_project_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join(".translator.toml");

        std::fs::write(&config_path, "").expect("Failed to write config file");

        let found_path = ConfigLoader::find_project_config(temp_dir.path());
        assert_eq!(found_path, Some(config_path));
    }

    #[test]
    fn test_find_project_config_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let found_path = ConfigLoader::find_project_config(temp_dir.path());
        assert_eq!(found_path, None);
    }

    #[test]
    fn test_project_config_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join(".translator.toml");

        let config_content = r#"
[translate]
target_lang = "AUTO"
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config file");

        let loader = ConfigLoader::new().with_project_config(&config_path);
        let result = loader.load_project();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("target language cannot be AUTO"));
    }

    #[test]
    fn test_normalize_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join(".translator.toml");

        let config_content = r#"
[include]
patterns = ["  **/*.rs  ", "  **/*.go  "]

[exclude]
patterns = ["  vendor/**  "]
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write config file");

        let loader = ConfigLoader::new().with_project_config(&config_path);
        let config = loader
            .load_project()
            .expect("Failed to load project config");

        assert_eq!(config.include.patterns, vec!["**/*.rs", "**/*.go"]);
        assert_eq!(config.exclude.patterns, vec!["vendor/**"]);
    }
}
