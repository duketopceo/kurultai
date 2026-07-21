//! Shared content hashing helpers.

use sha2::{Digest, Sha256};

/// Hex-encoded SHA-256 of `s`.
pub fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex_encode(&hasher.finalize())
}

/// Atom id from source + source_id + content (hashes content once).
pub fn atom_id(source: &str, source_id: &str, content: &str) -> String {
    atom_id_from_hash(source, source_id, &sha256_hex(content))
}

/// Atom id when the content hash is already known (avoids double-hashing).
pub fn atom_id_from_hash(source: &str, source_id: &str, content_hash: &str) -> String {
    sha256_hex(&format!("{source}\0{source_id}\0{content_hash}"))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}
