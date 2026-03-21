//! Binary cache implementation using MessagePack
//!
//! Single-file binary cache that stores all cache entries in one file with an index.
//! Uses MessagePack for serialization and includes a file header with magic number,
//! version, and checksum.

use crate::cache::util;
use crate::core::error::{Result, TranslateError};
use crate::core::models::{CacheConfig, CacheEntry, CacheEntryInfo, CacheStats};

use crc32fast::Hasher as Crc32Hasher;
use rand::Rng;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

const CACHE_MAGIC: &[u8; 8] = b"CBCACHE\x00";
const CACHE_VERSION: u32 = 1;
const HEADER_SIZE: usize = 32;
const INDEX_ENTRY_SIZE: usize = 72;
const HASH_SIZE: usize = 64;

#[derive(Debug, Clone)]
struct FileHeader {
    magic: [u8; 8],
    version: u32,
    index_offset: u64,
    index_size: u64,
    checksum: u32,
}

impl FileHeader {
    fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..8].copy_from_slice(&self.magic);
        bytes[8..12].copy_from_slice(&self.version.to_le_bytes());
        bytes[12..20].copy_from_slice(&self.index_offset.to_le_bytes());
        bytes[20..28].copy_from_slice(&self.index_size.to_le_bytes());
        bytes[28..32].copy_from_slice(&self.checksum.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(TranslateError::Cache("Header too small".to_string()));
        }

        let magic: [u8; 8] = bytes[0..8]
            .try_into()
            .map_err(|_| TranslateError::Cache("Invalid magic bytes length".to_string()))?;
        let version = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| TranslateError::Cache("Invalid version bytes length".to_string()))?,
        );
        let index_offset =
            u64::from_le_bytes(bytes[12..20].try_into().map_err(|_| {
                TranslateError::Cache("Invalid index_offset bytes length".to_string())
            })?);
        let index_size =
            u64::from_le_bytes(bytes[20..28].try_into().map_err(|_| {
                TranslateError::Cache("Invalid index_size bytes length".to_string())
            })?);
        let checksum = u32::from_le_bytes(
            bytes[28..32]
                .try_into()
                .map_err(|_| TranslateError::Cache("Invalid checksum bytes length".to_string()))?,
        );

        Ok(Self {
            magic,
            version,
            index_offset,
            index_size,
            checksum,
        })
    }
}

#[derive(Debug, Clone)]
enum EntryState {
    Pending(Vec<u8>),
    Committed { offset: u32, size: u32 },
}

/// Binary cache implementation
pub struct BinaryCache {
    config: CacheConfig,
    project_fingerprint: String,
    cache_file_path: PathBuf,
    entries: Arc<RwLock<HashMap<String, EntryState>>>,
    dirty: Arc<RwLock<bool>>,
}

impl BinaryCache {
    pub fn new(config: CacheConfig, project_dir: impl AsRef<Path>) -> Result<Self> {
        let project_dir = project_dir.as_ref();
        let project_fingerprint = util::generate_project_fingerprint(project_dir)?;

        let cache_dir = util::resolve_cache_dir(&config.mode, project_dir);
        let cache_file_path = cache_dir.join("translator-cache.bin");

        info!(
            cache_file = %cache_file_path.display(),
            mode = ?config.mode,
            enabled = config.enabled,
            "Creating binary cache"
        );

        let cache = Self {
            config,
            project_fingerprint,
            cache_file_path,
            entries: Arc::new(RwLock::new(HashMap::new())),
            dirty: Arc::new(RwLock::new(false)),
        };

        // Load existing cache index
        if let Err(e) = cache.load_index() {
            // Loading failed is not a fatal error, might be first use
            debug!(
                error = %e,
                "Failed to load cache index"
            );
        }

        debug!("Binary cache initialized successfully");
        Ok(cache)
    }

    /// Get the project fingerprint for this cache
    pub fn project_fingerprint(&self) -> &str {
        &self.project_fingerprint
    }

    fn ensure_cache_dir(&self) -> Result<()> {
        let cache_dir = self
            .cache_file_path
            .parent()
            .ok_or_else(|| TranslateError::Cache("Invalid cache file path".to_string()))?;

        std::fs::create_dir_all(cache_dir)
            .map_err(|e| TranslateError::Cache(format!("Failed to create cache dir: {}", e)))?;

        Ok(())
    }

    /// Load existing cache index
    fn load_index(&self) -> Result<()> {
        debug!(
            cache_file = %self.cache_file_path.display(),
            "Loading cache index"
        );
        let data_result = std::fs::read(&self.cache_file_path);
        let data = match data_result {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!("Cache file not found, starting with empty cache");
                return Ok(());
            }
            Err(e) => {
                return Err(TranslateError::Cache(format!(
                    "Failed to read cache file: {}",
                    e
                )))
            }
        };

        if data.len() < HEADER_SIZE {
            return Err(TranslateError::Cache("Cache file too small".to_string()));
        }

        let header = FileHeader::from_bytes(&data[..HEADER_SIZE])?;

        if header.magic != *CACHE_MAGIC {
            return Err(TranslateError::Cache(
                "Invalid cache file magic".to_string(),
            ));
        }

        if header.version != CACHE_VERSION {
            return Err(TranslateError::Cache(format!(
                "Unsupported cache version: {}",
                header.version
            )));
        }

        let data_checksum = calculate_crc32(&data[HEADER_SIZE..]);
        if data_checksum != header.checksum {
            return Err(TranslateError::Cache(
                "Cache file checksum mismatch".to_string(),
            ));
        }

        let index_start = header.index_offset as usize;
        let index_end = index_start + header.index_size as usize;

        if index_end > data.len() {
            return Err(TranslateError::Cache("Invalid index offset".to_string()));
        }

        let index_data = &data[index_start..index_end];
        let mut new_entries = HashMap::new();
        let mut offset = 0;

        while offset + INDEX_ENTRY_SIZE <= index_data.len() {
            let hash_bytes = &index_data[offset..offset + 64];
            let entry_offset = u32::from_le_bytes([
                index_data[offset + 64],
                index_data[offset + 65],
                index_data[offset + 66],
                index_data[offset + 67],
            ]);
            let size = u32::from_le_bytes([
                index_data[offset + 68],
                index_data[offset + 69],
                index_data[offset + 70],
                index_data[offset + 71],
            ]);

            let hash = String::from_utf8(hash_bytes.to_vec())
                .map_err(|e| TranslateError::Cache(format!("Invalid hash: {}", e)))?;

            new_entries.insert(
                hash,
                EntryState::Committed {
                    offset: entry_offset,
                    size,
                },
            );
            offset += INDEX_ENTRY_SIZE;
        }

        let entry_count = new_entries.len();

        let mut entries_lock = self.entries.write().map_err(|_| {
            TranslateError::Lock(
                "Failed to acquire write lock on entries in load_index".to_string(),
            )
        })?;
        *entries_lock = new_entries;

        debug!(entries = entry_count, "Cache index loaded successfully");
        Ok(())
    }

    fn read_data(&self, offset: u32, size: u32) -> Result<Vec<u8>> {
        let data = std::fs::read(&self.cache_file_path)
            .map_err(|e| TranslateError::Cache(format!("Failed to read cache file: {}", e)))?;

        if data.len() < HEADER_SIZE {
            return Err(TranslateError::Cache("Cache file too small".to_string()));
        }

        let start = HEADER_SIZE + offset as usize;
        let end = start + size as usize;

        if end > data.len() {
            return Err(TranslateError::Cache(format!(
                "Cache file read out of bounds: {}..{}",
                start, end
            )));
        }

        Ok(data[start..end].to_vec())
    }

    fn save(&self) -> Result<()> {
        self.ensure_cache_dir()?;

        // Check if dirty and reset flag
        let should_save = {
            let mut dirty_lock = self.dirty.write().map_err(|_| {
                TranslateError::Lock("Failed to acquire write lock on dirty in save".to_string())
            })?;
            if *dirty_lock {
                *dirty_lock = false;
                true
            } else {
                false
            }
        };

        if !should_save {
            debug!("Cache not dirty or already being saved, skipping save");
            return Ok(());
        }

        debug!("Saving cache to disk");

        // Snapshot entries and collect pending data
        let (entries_snapshot, pending_data) = {
            let entries_lock = self.entries.write().map_err(|_| {
                TranslateError::Lock("Failed to acquire write lock on entries in save".to_string())
            })?;

            let mut pending_data = HashMap::new();
            let mut entries_snapshot = HashMap::new();

            for (hash, entry_state) in entries_lock.iter() {
                if let EntryState::Pending(ref data) = entry_state {
                    pending_data.insert(hash.clone(), data.clone());
                }
                entries_snapshot.insert(hash.clone(), entry_state.clone());
            }

            (entries_snapshot, pending_data)
        };

        // Read existing file data once if needed
        let existing_data = if entries_snapshot
            .values()
            .any(|s| matches!(s, EntryState::Committed { .. }))
        {
            match std::fs::read(&self.cache_file_path) {
                Ok(data) => Some(data),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => {
                    return Err(TranslateError::Cache(format!(
                        "Failed to read cache file: {}",
                        e
                    )))
                }
            }
        } else {
            None
        };

        let mut data_buf = Vec::new();
        let mut new_entries = HashMap::new();

        // Build new data buffer and entries
        for (hash, entry_state) in entries_snapshot.iter() {
            match entry_state {
                EntryState::Pending(data) => {
                    let offset = data_buf.len() as u32;
                    let size = data.len() as u32;
                    data_buf.extend_from_slice(data);
                    new_entries.insert(hash.clone(), EntryState::Committed { offset, size });
                }
                EntryState::Committed { offset, size } => {
                    if let Some(pending) = pending_data.get(hash) {
                        // Use pending data
                        let new_offset = data_buf.len() as u32;
                        let new_size = pending.len() as u32;
                        data_buf.extend_from_slice(pending);
                        new_entries.insert(
                            hash.clone(),
                            EntryState::Committed {
                                offset: new_offset,
                                size: new_size,
                            },
                        );
                    } else if let Some(ref file_data) = existing_data {
                        // Read from existing data in memory
                        let start = HEADER_SIZE + *offset as usize;
                        let end = start + *size as usize;
                        if end <= file_data.len() {
                            let entry_data = &file_data[start..end];
                            let new_offset = data_buf.len() as u32;
                            let new_size = entry_data.len() as u32;
                            data_buf.extend_from_slice(entry_data);
                            new_entries.insert(
                                hash.clone(),
                                EntryState::Committed {
                                    offset: new_offset,
                                    size: new_size,
                                },
                            );
                        }
                    }
                }
            }
        }

        debug!(
            entries = new_entries.len(),
            data_size = data_buf.len(),
            "Cache data prepared"
        );

        if new_entries.is_empty() {
            return Ok(());
        }

        // Build index
        let mut index_buf = Vec::new();
        for (hash, entry_state) in new_entries.iter() {
            let hash_bytes = hash.as_bytes();
            if hash_bytes.len() != HASH_SIZE {
                return Err(TranslateError::Cache(format!(
                    "Hash must be exactly {} bytes, got {}",
                    HASH_SIZE,
                    hash_bytes.len()
                )));
            }
            if let EntryState::Committed { offset, size } = entry_state {
                index_buf.extend_from_slice(hash_bytes);
                index_buf.extend_from_slice(&offset.to_le_bytes());
                index_buf.extend_from_slice(&size.to_le_bytes());
            }
        }

        let header = FileHeader {
            magic: *CACHE_MAGIC,
            version: CACHE_VERSION,
            index_offset: (HEADER_SIZE + data_buf.len()) as u64,
            index_size: index_buf.len() as u64,
            checksum: 0,
        };

        let mut hasher = Crc32Hasher::new();
        hasher.update(&data_buf);
        hasher.update(&index_buf);
        let checksum = hasher.finalize();

        let header = FileHeader { checksum, ..header };

        let mut file_buf = Vec::new();
        file_buf.extend_from_slice(&header.to_bytes());
        file_buf.extend_from_slice(&data_buf);
        file_buf.extend_from_slice(&index_buf);

        let file_size = file_buf.len();
        let temp_file = format!(
            "{}.tmp.{}",
            self.cache_file_path.display(),
            rand::thread_rng().gen::<u64>()
        );
        std::fs::write(&temp_file, &file_buf)
            .map_err(|e| TranslateError::Cache(format!("Failed to write temp file: {}", e)))?;
        std::fs::rename(&temp_file, &self.cache_file_path)
            .map_err(|e| TranslateError::Cache(format!("Failed to rename cache file: {}", e)))?;

        {
            let mut entries_lock = self.entries.write().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire write lock on entries in save (update)".to_string(),
                )
            })?;
            *entries_lock = new_entries;
        }

        debug!(
            cache_file = %self.cache_file_path.display(),
            file_size = file_size,
            "Cache saved successfully"
        );
        Ok(())
    }

    fn add_entry(&self, entry: &CacheEntry) -> Result<()> {
        debug!(
            file_hash = %entry.file_hash,
            file_path = %entry.file_path,
            "Adding cache entry"
        );
        let serialized = rmp_serde::to_vec(entry)
            .map_err(|e| TranslateError::Cache(format!("Failed to serialize entry: {}", e)))?;

        let mut entries_lock = self.entries.write().map_err(|_| {
            TranslateError::Lock("Failed to acquire write lock on entries in add_entry".to_string())
        })?;
        let mut dirty_lock = self.dirty.write().map_err(|_| {
            TranslateError::Lock("Failed to acquire write lock on dirty in add_entry".to_string())
        })?;

        entries_lock.insert(entry.file_hash.clone(), EntryState::Pending(serialized));
        *dirty_lock = true;

        debug!("Cache entry added successfully");
        Ok(())
    }

    /// Get cached entry for a file hash
    pub fn get(&self, file_hash: &str) -> Result<Option<CacheEntry>> {
        if !self.config.enabled {
            debug!("Cache disabled, returning None");
            return Ok(None);
        }

        debug!(
            file_hash = %file_hash,
            "Getting cache entry"
        );

        let entry_state = {
            let entries_lock = self.entries.read().map_err(|_| {
                TranslateError::Lock("Failed to acquire read lock on entries in get".to_string())
            })?;
            entries_lock.get(file_hash).cloned()
        };

        if let Some(entry_state) = entry_state {
            let data = match entry_state {
                EntryState::Pending(data) => data,
                EntryState::Committed { offset, size } => self.read_data(offset, size)?,
            };

            let cache_entry: CacheEntry = rmp_serde::from_slice(&data).map_err(|e| {
                TranslateError::Cache(format!("Failed to deserialize entry: {}", e))
            })?;

            if cache_entry.project_fingerprint != self.project_fingerprint {
                debug!(
                    file_hash = %file_hash,
                    "Cache entry fingerprint mismatch, returning None"
                );
                return Ok(None);
            }

            debug!(
                file_hash = %file_hash,
                file_path = %cache_entry.file_path,
                "Cache entry found"
            );
            Ok(Some(cache_entry))
        } else {
            debug!(
                file_hash = %file_hash,
                "Cache entry not found"
            );
            Ok(None)
        }
    }

    /// Store a cache entry
    pub fn set(&self, entry: &CacheEntry) -> Result<()> {
        if !self.config.enabled {
            debug!("Cache disabled, skipping set");
            return Ok(());
        }

        debug!(
            file_hash = %entry.file_hash,
            file_path = %entry.file_path,
            "Setting cache entry"
        );
        self.add_entry(entry)?;
        self.save()?;

        debug!("Cache entry set successfully");
        Ok(())
    }

    /// Invalidate cache entry for a file hash
    pub fn invalidate(&self, file_hash: &str) -> Result<()> {
        if !self.config.enabled {
            debug!("Cache disabled, skipping invalidate");
            return Ok(());
        }

        debug!(
            file_hash = %file_hash,
            "Invalidating cache entry"
        );

        {
            let mut entries_lock = self.entries.write().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire write lock on entries in invalidate".to_string(),
                )
            })?;
            let mut dirty_lock = self.dirty.write().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire write lock on dirty in invalidate".to_string(),
                )
            })?;

            entries_lock.remove(file_hash);
            *dirty_lock = true;
        }

        self.save()?;

        debug!("Cache entry invalidated successfully");
        Ok(())
    }

    /// Clear all cache
    pub fn clear(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("Cache disabled, skipping clear");
            return Ok(());
        }

        debug!("Clearing cache");

        match std::fs::remove_file(&self.cache_file_path) {
            Ok(_) => {
                let mut entries_lock = self.entries.write().map_err(|_| {
                    TranslateError::Lock(
                        "Failed to acquire write lock on entries in clear".to_string(),
                    )
                })?;
                entries_lock.clear();
                debug!("Cache cleared successfully");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut entries_lock = self.entries.write().map_err(|_| {
                    TranslateError::Lock(
                        "Failed to acquire write lock on entries in clear".to_string(),
                    )
                })?;
                entries_lock.clear();
                debug!("Cache file not found, entries cleared");
                Ok(())
            }
            Err(e) => Err(TranslateError::Cache(format!(
                "Failed to remove cache file: {}",
                e
            ))),
        }
    }

    /// Close cache and release resources
    pub fn close(&self) -> Result<()> {
        debug!("Closing cache");
        self.save()?;
        debug!("Cache closed successfully");
        Ok(())
    }

    /// List all cache entries
    pub fn list_entries(&self) -> Result<Vec<CacheEntryInfo>> {
        if !self.config.enabled {
            debug!("Cache disabled, returning empty list");
            return Ok(Vec::new());
        }

        debug!("Listing cache entries");

        let hashes: Vec<String> = {
            let entries_lock = self.entries.read().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire read lock on entries in list_entries".to_string(),
                )
            })?;
            entries_lock.keys().cloned().collect()
        };

        let mut result = Vec::new();
        for hash in hashes {
            if let Some(entry) = self.get(&hash)? {
                result.push(CacheEntryInfo {
                    file_hash: hash,
                    file_path: entry.file_path,
                });
            }
        }

        debug!(entries = result.len(), "Cache entries listed successfully");
        Ok(result)
    }

    /// Cleanup orphaned cache entries (files that no longer exist)
    pub fn cleanup_orphaned(&self, existing_hashes: HashMap<String, bool>) -> Result<usize> {
        if !self.config.enabled {
            debug!("Cache disabled, skipping cleanup");
            return Ok(0);
        }

        info!(
            total_entries = existing_hashes.len(),
            "Cleaning up orphaned cache entries"
        );

        let to_remove: Vec<String> = {
            let entries_lock = self.entries.read().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire read lock on entries in cleanup_orphaned".to_string(),
                )
            })?;
            entries_lock
                .keys()
                .filter(|hash| !existing_hashes.contains_key(*hash))
                .cloned()
                .collect()
        };

        if to_remove.is_empty() {
            debug!("No orphaned entries found");
            return Ok(0);
        }

        debug!(
            orphaned_count = to_remove.len(),
            "Removing orphaned entries"
        );

        {
            let mut entries_lock = self.entries.write().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire write lock on entries in cleanup_orphaned".to_string(),
                )
            })?;
            let mut dirty_lock = self.dirty.write().map_err(|_| {
                TranslateError::Lock(
                    "Failed to acquire write lock on dirty in cleanup_orphaned".to_string(),
                )
            })?;

            for hash in &to_remove {
                entries_lock.remove(hash);
            }
            *dirty_lock = true;
        }

        self.save()?;

        info!(
            removed_count = to_remove.len(),
            "Orphaned entries cleaned up successfully"
        );
        Ok(to_remove.len())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        if !self.config.enabled {
            debug!("Cache disabled, returning empty stats");
            return Ok(CacheStats::default());
        }

        let entries_lock = self.entries.read().map_err(|_| {
            TranslateError::Lock("Failed to acquire read lock on entries in stats".to_string())
        })?;
        let entry_count = entries_lock.len();

        let total_size = match std::fs::metadata(&self.cache_file_path) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => {
                return Err(TranslateError::Cache(format!(
                    "Failed to get metadata: {}",
                    e
                )))
            }
        };

        let stats = CacheStats {
            entry_count,
            total_size,
        };

        debug!(
            entry_count = stats.entry_count,
            total_size = stats.total_size,
            "Cache stats retrieved successfully"
        );
        Ok(stats)
    }
}

fn calculate_crc32(data: &[u8]) -> u32 {
    let mut hasher = Crc32Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    fn generate_test_hash(seed: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    #[test]
    fn test_binary_cache_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: crate::core::models::CacheMode::Local,
            directory: ".cache".to_string(),
            format: "binary".to_string(),
        };

        let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
        let fingerprint = cache.project_fingerprint();

        // Test set and get
        let hash1 = generate_test_hash("test_file");
        let mut entry = CacheEntry::new(&hash1, "/path/to/file.txt", 123456, "local", fingerprint);
        entry.mark_as_translated();

        cache.set(&entry).unwrap();

        let retrieved = cache.get(&hash1).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.file_hash, hash1);
        assert_eq!(retrieved.file_path, "/path/to/file.txt");
        assert!(retrieved.is_translated);
        assert!(retrieved.translation_timestamp > 0);

        // Test stats
        let stats = cache.stats().unwrap();
        assert_eq!(stats.entry_count, 1);

        // Test invalidate
        cache.invalidate(&hash1).unwrap();

        let retrieved = cache.get(&hash1).unwrap();
        assert!(retrieved.is_none());

        // Test close
        cache.close().unwrap();
    }

    #[test]
    fn test_binary_cache_list_entries() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: crate::core::models::CacheMode::Local,
            directory: ".cache".to_string(),
            format: "binary".to_string(),
        };

        let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
        let fingerprint = cache.project_fingerprint();

        let hash1 = generate_test_hash("file1");
        let hash2 = generate_test_hash("file2");
        let entry1 = CacheEntry::new(&hash1, "/path/to/file1.txt", 123456, "local", fingerprint);
        let entry2 = CacheEntry::new(&hash2, "/path/to/file2.txt", 123456, "local", fingerprint);

        cache.set(&entry1).unwrap();
        cache.set(&entry2).unwrap();

        let entries = cache.list_entries().unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_binary_cache_cleanup_orphaned() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            enabled: true,
            mode: crate::core::models::CacheMode::Local,
            directory: ".cache".to_string(),
            format: "binary".to_string(),
        };

        let cache = BinaryCache::new(config, temp_dir.path()).unwrap();
        let fingerprint = cache.project_fingerprint();

        let hash1 = generate_test_hash("file1");
        let hash2 = generate_test_hash("file2");
        let entry1 = CacheEntry::new(&hash1, "/path/to/file1.txt", 123456, "local", fingerprint);
        let entry2 = CacheEntry::new(&hash2, "/path/to/file2.txt", 123456, "local", fingerprint);

        cache.set(&entry1).unwrap();
        cache.set(&entry2).unwrap();

        let mut existing_hashes = HashMap::new();
        existing_hashes.insert(hash1.clone(), true);

        let cleaned = cache.cleanup_orphaned(existing_hashes).unwrap();
        assert_eq!(cleaned, 1);

        let entries = cache.list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_hash, hash1);
    }
}
