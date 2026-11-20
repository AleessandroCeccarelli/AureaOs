// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Safe, backend-independent ICC profile inspection and identity.

mod header;
mod identity;
mod storage;

pub use header::{IccHeader, IccHeaderError, IccProfileSummary, IccTag};
pub use identity::IccProfileFingerprint;
pub use storage::{IccProfileStore, ProfileStoreError, StoredProfile};

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
    use super::{MAX_PROFILE_SIZE, fingerprint_profile, inspect_profile};

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

    fn profile_with_one_tag(offset: u32, size: u32) -> Vec<u8> {
        let mut bytes = vec![0_u8; 144];
        bytes[0..4].copy_from_slice(&(144_u32).to_be_bytes());
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes[128..132].copy_from_slice(&1_u32.to_be_bytes());
        bytes[132..136].copy_from_slice(b"desc");
        bytes[136..140].copy_from_slice(&offset.to_be_bytes());
        bytes[140..144].copy_from_slice(&size.to_be_bytes());
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
    fn rejects_tag_table_that_exceeds_profile_bounds() {
        let mut bytes = minimal_profile();
        bytes[128..132].copy_from_slice(&1_u32.to_be_bytes());
        assert!(inspect_profile(&bytes).is_err());
    }

    #[test]
    fn rejects_tag_data_overlapping_the_tag_table() {
        assert!(inspect_profile(&profile_with_one_tag(132, 4)).is_err());
    }

    #[test]
    fn rejects_tag_data_outside_declared_profile() {
        assert!(inspect_profile(&profile_with_one_tag(140, 8)).is_err());
    }

    #[test]
    fn accepts_tag_data_inside_declared_profile() {
        let mut bytes = vec![0_u8; 148];
        bytes[0..4].copy_from_slice(&(148_u32).to_be_bytes());
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes[128..132].copy_from_slice(&1_u32.to_be_bytes());
        bytes[132..136].copy_from_slice(b"desc");
        bytes[136..140].copy_from_slice(&(144_u32).to_be_bytes());
        bytes[140..144].copy_from_slice(&(4_u32).to_be_bytes());
        assert_eq!(inspect_profile(&bytes).unwrap().tag_count, 1);
    }

    #[test]
    fn fingerprint_is_stable_for_identical_content() {
        let bytes = minimal_profile();
        assert_eq!(
            fingerprint_profile(&bytes).unwrap(),
            fingerprint_profile(&bytes).unwrap()
        );
    }
    #[test]
    fn rejects_unaligned_tag_data() {
        assert!(inspect_profile(&profile_with_one_tag(145, 4)).is_err());
    }

    #[test]
    fn rejects_duplicate_tag_signatures() {
        let mut bytes = vec![0_u8; 160];
        bytes[0..4].copy_from_slice(&(160_u32).to_be_bytes());
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes[128..132].copy_from_slice(&2_u32.to_be_bytes());
        for start in [132, 144] {
            bytes[start..start + 4].copy_from_slice(b"desc");
            bytes[start + 4..start + 8].copy_from_slice(&(156_u32).to_be_bytes());
            bytes[start + 8..start + 12].copy_from_slice(&(4_u32).to_be_bytes());
        }
        assert!(inspect_profile(&bytes).is_err());
    }

    #[test]
    fn rejects_partially_overlapping_tag_payloads() {
        let mut bytes = vec![0_u8; 168];
        bytes[0..4].copy_from_slice(&(168_u32).to_be_bytes());
        bytes[12..16].copy_from_slice(b"mntr");
        bytes[16..20].copy_from_slice(b"RGB ");
        bytes[20..24].copy_from_slice(b"XYZ ");
        bytes[36..40].copy_from_slice(b"acsp");
        bytes[128..132].copy_from_slice(&2_u32.to_be_bytes());
        bytes[132..136].copy_from_slice(b"desc");
        bytes[136..140].copy_from_slice(&(156_u32).to_be_bytes());
        bytes[140..144].copy_from_slice(&(8_u32).to_be_bytes());
        bytes[144..148].copy_from_slice(b"cprt");
        bytes[148..152].copy_from_slice(&(160_u32).to_be_bytes());
        bytes[152..156].copy_from_slice(&(8_u32).to_be_bytes());
        assert!(inspect_profile(&bytes).is_err());
    }

}
