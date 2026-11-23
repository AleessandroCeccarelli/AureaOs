// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

use sha2::{Digest, Sha256};

/// Stable SHA-256 content fingerprint for an ICC profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IccProfileFingerprint([u8; 32]);

impl IccProfileFingerprint {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut value = [0_u8; 32];
        value.copy_from_slice(&digest);
        Self(value)
    }

    /// Returns the raw SHA-256 digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns a lowercase hexadecimal representation.
    pub fn to_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(64);
        for byte in self.0 {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }
}
