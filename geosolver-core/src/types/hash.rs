use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

pub fn hash_bytes(bytes: &[u8]) -> Hash {
    let digest = Sha256::digest(bytes);
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    Hash(out)
}

pub fn hash_tagged(tag: &str, bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update((tag.len() as u64).to_be_bytes());
    hasher.update(tag.as_bytes());
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    Hash(out)
}

pub fn hash_sequence(tag: &str, chunks: &[Vec<u8>]) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice((chunks.len() as u64).to_be_bytes().as_slice());
    for chunk in chunks {
        bytes.extend_from_slice((chunk.len() as u64).to_be_bytes().as_slice());
        bytes.extend_from_slice(chunk);
    }
    hash_tagged(tag, &bytes)
}
