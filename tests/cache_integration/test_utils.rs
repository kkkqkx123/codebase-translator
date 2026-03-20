use sha2::{Digest, Sha256};

pub mod hash_utils {
    use super::*;

    pub fn generate_test_hash(seed: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}
