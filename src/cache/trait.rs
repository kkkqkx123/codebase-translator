//! Cache trait definition
//!
//! This module defines the Cache trait for storing translation results.
//!
//! Implementations of this trait should provide appropriate logging for:
//! - Cache initialization and configuration
//! - Cache hits and misses
//! - Cache entry operations (get, set, invalidate)
//! - Cache maintenance operations (clear, cleanup_orphaned)
//! - Cache statistics and performance metrics

use std::collections::HashMap;

use crate::core::error::Result;
use crate::core::models::{CacheEntry, CacheEntryInfo, CacheStats};

/// Cache trait for storing translation results
pub trait Cache: Send + Sync {
    /// Get cached entry for a file hash
    fn get(&self, file_hash: &str) -> Result<Option<CacheEntry>>;

    /// Store a cache entry
    fn set(&self, entry: &CacheEntry) -> Result<()>;

    /// Invalidate cache entry for a file hash
    fn invalidate(&self, file_hash: &str) -> Result<()>;

    /// Clear all cache
    fn clear(&self) -> Result<()>;

    /// Close cache and release resources
    fn close(&self) -> Result<()>;

    /// List all cache entries
    fn list_entries(&self) -> Result<Vec<CacheEntryInfo>>;

    /// Cleanup orphaned cache entries (files that no longer exist)
    fn cleanup_orphaned(&self, existing_hashes: HashMap<String, bool>) -> Result<usize>;

    /// Get cache statistics
    fn stats(&self) -> Result<CacheStats>;
}
