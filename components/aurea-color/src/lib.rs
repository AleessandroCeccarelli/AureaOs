// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Safe, backend-independent ICC profile inspection and identity.

mod header;
mod identity;

pub use header::{IccHeader, IccHeaderError, IccProfileSummary, IccTag};
pub use identity::IccProfileFingerprint;

/// Maximum profile payload accepted by Aurea Color.
pub const MAX_PROFILE_SIZE: usize = 16 * 1024 * 1024;

/// Validates the ICC header and tag table, returning safe profile metadata.
pub fn inspect_profile(bytes: &[u8]) -> Result<IccProfileSummary, IccHeaderError> {
    IccHeader::parse(bytes).map(|header| header.summary())
}

/// Computes a content-addressed identity for a validated ICC profile.
pub fn fingerprint_profile(bytes: &[u8]) -> Result<IccProfileFingerprint, IccHeaderError> {
    IccHeader::parse(bytes)?;
    Ok(IccProfileFingerprint::from_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::{fingerprint_profile, inspect_profile, MAX_PROFILE_SIZE};

    fn minimal_profile() -> Vec<u8> {
        let mut bytes = vec![0_u8; 132];
        bytes[0..4].copy_from_slice(&(132_u32).to_be_bytes());
        bytes[8..12].copy_from_slice(b"\x04\x30\x00\x00");
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes
    }

    #[test]
    fn accepts_minimal_well_formed_profile() {
        let summary = inspect_profile(&minimal_profile()).unwrap();
        assert_eq!(&summary.device_class, b"mntr");
        assert_eq!(&summary.color_space, b"RGB ");
        assert_eq!(summary.tag_count, 0);
    }

    #[test]
    fn rejects_profile_over_size_limit() {
        let bytes = vec![0; MAX_PROFILE_SIZE + 1];
        assert!(inspect_profile(&bytes).is_err());
    }

    #[test]
    fn rejects_invalid_icc_signature() {
        let mut bytes = minimal_profile();
        bytes[36..40].copy_from_slice(b"nope");
        assert!(inspect_profile(&bytes).is_err());
    }

    #[test]
    fn fingerprint_is_stable_for_identical_content() {
        let bytes = minimal_profile();
        assert_eq!(
            fingerprint_profile(&bytes).unwrap(),
            fingerprint_profile(&bytes).unwrap()
        );
    }
}
