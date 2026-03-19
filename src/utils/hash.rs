//! Hash utility functions

use tracing::trace;

use sha2::{Digest, Sha256};

/// Calculate SHA-256 hash for content
///
/// Returns a 64-character hexadecimal string representing the SHA-256 hash.
///
/// # Examples
///
/// ```
/// use codebase_translate::utils::hash::calculate_hash;
///
/// let content = b"hello world";
/// let hash = calculate_hash(content);
/// assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
/// ```
pub fn calculate_hash(content: &[u8]) -> String {
    trace!(
        content_len = content.len(),
        "Calculating hash"
    );
    
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash_length() {
        let content = b"test content";
        let hash = calculate_hash(content);
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex characters");
    }

    #[test]
    fn test_calculate_hash_consistency() {
        let content = b"consistent content";
        let hash1 = calculate_hash(content);
        let hash2 = calculate_hash(content);
        assert_eq!(hash1, hash2, "Same content should produce same hash");
    }

    #[test]
    fn test_calculate_hash_different_content() {
        let hash1 = calculate_hash(b"content1");
        let hash2 = calculate_hash(b"content2");
        assert_ne!(
            hash1, hash2,
            "Different content should produce different hashes"
        );
    }

    #[test]
    fn test_calculate_hash_empty() {
        let hash = calculate_hash(b"");
        assert_eq!(
            hash.len(),
            64,
            "Empty content should still produce 64-char hash"
        );
        // SHA-256 of empty string is known
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_calculate_hash_large_content() {
        let content = vec![0u8; 10000];
        let hash = calculate_hash(&content);
        assert_eq!(
            hash.len(),
            64,
            "Large content should still produce 64-char hash"
        );
    }
}
