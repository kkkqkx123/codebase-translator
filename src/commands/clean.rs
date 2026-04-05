use clap::Parser;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::{
    cache::CacheFactory,
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
};

use super::Command;

#[derive(Parser, Debug)]
pub struct CleanArgs {
    #[arg(long, help = "Clean cache files")]
    pub cache: bool,

    #[arg(long, help = "Clean backup files")]
    pub backup: bool,

    #[arg(long, help = "Clean both cache and backup files")]
    pub all: bool,

    #[arg(long, help = "Only clean files older than N days")]
    pub older_than: Option<u32>,

    #[arg(long, help = "Dry run - show what would be deleted")]
    pub dry_run: bool,

    #[arg(long, help = "Backup directory path")]
    pub backup_dir: Option<String>,

    #[arg(long, help = "Cache directory path")]
    pub cache_dir: Option<String>,
}

impl Command for CleanArgs {
    fn execute(&self, _global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()> {
        execute_clean_command(project_config, self)
    }
}

fn execute_clean_command(project_config: &ProjectConfig, args: &CleanArgs) -> Result<()> {
    let clean_cache = args.cache || args.all;
    let clean_backup = args.backup || args.all;

    if !clean_cache && !clean_backup {
        warn!("No clean targets specified. Use --cache, --backup, or --all");
        return Ok(());
    }

    info!(
        cache = clean_cache,
        backup = clean_backup,
        dry_run = args.dry_run,
        older_than_days = args.older_than,
        "Starting clean operation"
    );

    let current_dir = std::env::current_dir()?;

    if clean_cache {
        clean_cache_files(project_config, args, &current_dir)?;
    }

    if clean_backup {
        clean_backup_files(project_config, args, &current_dir)?;
    }

    info!("Clean operation completed");
    Ok(())
}

fn clean_cache_files(
    project_config: &ProjectConfig,
    args: &CleanArgs,
    current_dir: &Path,
) -> Result<()> {
    info!("Cleaning cache files...");

    let cache_dir = if let Some(dir) = &args.cache_dir {
        PathBuf::from(dir)
    } else {
        current_dir.join(&project_config.cache.directory)
    };

    debug!(cache_dir = %cache_dir.display(), "Cache directory");

    if !cache_dir.exists() {
        info!("Cache directory does not exist: {}", cache_dir.display());
        return Ok(());
    }

    debug!("Creating cache instance");
    let cache = CacheFactory::create(
        &project_config.cache,
        current_dir.to_string_lossy().as_ref(),
    )?;

    if args.dry_run {
        debug!("Retrieving cache entries for dry run");
        let entries = cache.list_entries()?;
        let filtered = filter_by_age(&entries, args.older_than, &cache_dir)?;
        info!("Dry run: would delete {} cache entries", filtered.len());
        for entry in filtered {
            info!("  - {}", entry.file_path);
        }
    } else {
        debug!("Clearing cache");
        cache.clear()?;
        info!("Cache cleared successfully");
    }

    Ok(())
}

fn clean_backup_files(
    project_config: &ProjectConfig,
    args: &CleanArgs,
    current_dir: &Path,
) -> Result<()> {
    info!("Cleaning backup files...");

    let backup_dir = if let Some(dir) = &args.backup_dir {
        PathBuf::from(dir)
    } else if let Some(config_dir) = &project_config.writer.backup_dir {
        PathBuf::from(config_dir)
    } else {
        current_dir.join(".translator").join("backups")
    };

    debug!(backup_dir = %backup_dir.display(), "Backup directory");

    if !backup_dir.exists() {
        info!("Backup directory does not exist: {}", backup_dir.display());
        return Ok(());
    }

    debug!("Finding backup files");
    let backup_files = find_backup_files(&backup_dir)?;
    debug!(found_files = backup_files.len(), "Backup files found");
    let filtered = filter_backup_files_by_age(&backup_files, args.older_than)?;

    if args.dry_run {
        info!("Dry run: would delete {} backup files", filtered.len());
        for file in &filtered {
            info!("  - {}", file.display());
        }
    } else {
        let mut deleted_count = 0;
        for file in filtered {
            if let Err(e) = std::fs::remove_file(&file) {
                warn!("Failed to delete {}: {}", file.display(), e);
            } else {
                deleted_count += 1;
            }
        }
        info!("Deleted {} backup files", deleted_count);
    }

    Ok(())
}

fn find_backup_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut backup_files = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "bak" {
                        backup_files.push(path);
                    }
                }
            } else if path.is_dir() {
                backup_files.extend(find_backup_files(&path)?);
            }
        }
    }

    Ok(backup_files)
}

fn filter_by_age(
    entries: &[crate::core::models::CacheEntryInfo],
    older_than_days: Option<u32>,
    cache_dir: &Path,
) -> Result<Vec<crate::core::models::CacheEntryInfo>> {
    if older_than_days.is_none() {
        return Ok(entries.to_vec());
    }

    let threshold = chrono::Utc::now() - chrono::Duration::days(older_than_days.unwrap() as i64);
    let mut filtered = Vec::new();

    for entry in entries {
        let file_path = cache_dir.join(&entry.file_path);
        if let Ok(metadata) = std::fs::metadata(&file_path) {
            if let Ok(modified) = metadata.modified() {
                let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                if modified_time < threshold {
                    filtered.push(entry.clone());
                }
            }
        }
    }

    Ok(filtered)
}

fn filter_backup_files_by_age(
    files: &[PathBuf],
    older_than_days: Option<u32>,
) -> Result<Vec<PathBuf>> {
    if older_than_days.is_none() {
        return Ok(files.to_vec());
    }

    let threshold = chrono::Utc::now() - chrono::Duration::days(older_than_days.unwrap() as i64);
    let mut filtered = Vec::new();

    for file in files {
        if let Ok(metadata) = std::fs::metadata(file) {
            if let Ok(modified) = metadata.modified() {
                let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                if modified_time < threshold {
                    filtered.push(file.clone());
                }
            }
        }
    }

    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_project_config() -> ProjectConfig {
        ProjectConfig::default()
    }

    fn create_test_global_config() -> GlobalConfig {
        GlobalConfig::default()
    }

    fn create_temp_backup_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let temp_dir = std::env::temp_dir().join(format!(
            "translator_test_backups_{}_{}_{}",
            pid, timestamp, counter
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create temp backup dir");
        temp_dir
    }

    fn create_test_backup_file(dir: &Path, name: &str, _days_old: i64) -> PathBuf {
        let file_path = dir.join(format!("{}.bak", name));
        fs::write(&file_path, "test content").expect("Failed to write test file");

        file_path
    }

    fn cleanup_temp_dir(dir: &Path) {
        if dir.exists() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn test_clean_args_default() {
        let args = CleanArgs {
            cache: false,
            backup: false,
            all: false,
            older_than: None,
            dry_run: false,
            backup_dir: None,
            cache_dir: None,
        };

        assert!(!args.cache);
        assert!(!args.backup);
        assert!(!args.all);
        assert!(args.older_than.is_none());
        assert!(!args.dry_run);
    }

    #[test]
    fn test_clean_args_with_options() {
        let args = CleanArgs {
            cache: true,
            backup: false,
            all: false,
            older_than: Some(7),
            dry_run: true,
            backup_dir: Some("/tmp/backups".to_string()),
            cache_dir: Some("/tmp/cache".to_string()),
        };

        assert!(args.cache);
        assert!(!args.backup);
        assert!(!args.all);
        assert_eq!(args.older_than, Some(7));
        assert!(args.dry_run);
        assert_eq!(args.backup_dir, Some("/tmp/backups".to_string()));
        assert_eq!(args.cache_dir, Some("/tmp/cache".to_string()));
    }

    #[test]
    fn test_find_backup_files_empty_dir() {
        let temp_dir = std::env::temp_dir().join("test_empty_backups");
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let result = find_backup_files(&temp_dir).expect("Failed to find backup files");
        assert!(result.is_empty());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_find_backup_files_with_files() {
        let temp_dir = create_temp_backup_dir();

        create_test_backup_file(&temp_dir, "file1", 0);
        create_test_backup_file(&temp_dir, "file2", 0);
        fs::write(temp_dir.join("not_backup.txt"), "test content")
            .expect("Failed to write test file");

        let result = find_backup_files(&temp_dir).expect("Failed to find backup files");
        assert_eq!(result.len(), 2);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_find_backup_files_recursive() {
        let temp_dir = create_temp_backup_dir();
        let sub_dir = temp_dir.join("subdir");
        fs::create_dir_all(&sub_dir).expect("Failed to create subdir");

        create_test_backup_file(&temp_dir, "file1", 0);
        create_test_backup_file(&sub_dir, "file2", 0);

        let result = find_backup_files(&temp_dir).expect("Failed to find backup files");
        assert_eq!(result.len(), 2);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_filter_backup_files_by_age_no_filter() {
        let temp_dir = create_temp_backup_dir();

        let file1 = create_test_backup_file(&temp_dir, "file1", 0);
        let file2 = create_test_backup_file(&temp_dir, "file2", 0);

        let files = vec![file1.clone(), file2.clone()];
        let result =
            filter_backup_files_by_age(&files, None).expect("Failed to filter files by age");

        assert_eq!(result.len(), 2);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_filter_by_age_no_filter() {
        let entries = vec![
            crate::core::models::CacheEntryInfo {
                file_hash: "hash1".to_string(),
                file_path: "file1".to_string(),
            },
            crate::core::models::CacheEntryInfo {
                file_hash: "hash2".to_string(),
                file_path: "file2".to_string(),
            },
        ];

        let temp_dir = std::env::temp_dir();
        let result = filter_by_age(&entries, None, &temp_dir).expect("Failed to filter by age");

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_clean_command_no_targets() {
        let project_config = create_test_project_config();
        let args = CleanArgs {
            cache: false,
            backup: false,
            all: false,
            older_than: None,
            dry_run: false,
            backup_dir: None,
            cache_dir: None,
        };

        let result = execute_clean_command(&project_config, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_command_with_cache_target() {
        let project_config = create_test_project_config();
        let args = CleanArgs {
            cache: true,
            backup: false,
            all: false,
            older_than: None,
            dry_run: false,
            backup_dir: None,
            cache_dir: None,
        };

        let result = execute_clean_command(&project_config, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_command_with_all_target() {
        let project_config = create_test_project_config();
        let args = CleanArgs {
            cache: false,
            backup: false,
            all: true,
            older_than: None,
            dry_run: false,
            backup_dir: None,
            cache_dir: None,
        };

        let result = execute_clean_command(&project_config, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_command_with_custom_backup_dir() {
        let temp_dir = create_temp_backup_dir();
        let project_config = create_test_project_config();

        let args = CleanArgs {
            cache: false,
            backup: true,
            all: false,
            older_than: None,
            dry_run: false,
            backup_dir: Some(temp_dir.to_string_lossy().to_string()),
            cache_dir: None,
        };

        let result = execute_clean_command(&project_config, &args);
        assert!(result.is_ok());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_clean_command_dry_run() {
        let temp_dir = create_temp_backup_dir();
        create_test_backup_file(&temp_dir, "file1", 0);

        let project_config = create_test_project_config();
        let args = CleanArgs {
            cache: false,
            backup: true,
            all: false,
            older_than: None,
            dry_run: true,
            backup_dir: Some(temp_dir.to_string_lossy().to_string()),
            cache_dir: None,
        };

        let result = execute_clean_command(&project_config, &args);
        assert!(result.is_ok());

        let backup_files = find_backup_files(&temp_dir).expect("Failed to find backup files");
        assert_eq!(backup_files.len(), 1);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_command_impl() {
        let project_config = create_test_project_config();
        let global_config = create_test_global_config();
        let args = CleanArgs {
            cache: true,
            backup: false,
            all: false,
            older_than: None,
            dry_run: false,
            backup_dir: None,
            cache_dir: None,
        };

        let result = args.execute(&global_config, &project_config);
        assert!(result.is_ok());
    }
}
