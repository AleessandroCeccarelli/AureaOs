// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Converts validated DRM observations into desktop physical-identity evidence.

use aurea_desktop_api::display::DisplayIdentityEvidence;

use super::{
    connector_identity::DrmConnectorIdentity, edid::EdidInfo, output::DrmOutputDescriptor,
};

/// Builds conservative physical-monitor evidence from a discovered DRM output.
///
/// Both validated EDID and a resolved sysfs connector path are required.
/// Runtime output IDs are deliberately excluded: they are not persistent
/// physical identities. Outputs without a usable unit serial remain valid
/// evidence, but the desktop resolver will require manual review.
pub fn identity_evidence_for_output(
    output: &DrmOutputDescriptor,
) -> Option<DisplayIdentityEvidence> {
    evidence_from_observation(output.edid()?, output.physical_identity()?)
}

fn evidence_from_observation(
    edid: &EdidInfo,
    connector: &DrmConnectorIdentity,
) -> Option<DisplayIdentityEvidence> {
    let serial = edid
        .serial_number()
        .map(str::to_owned)
        .or_else(|| (edid.numeric_serial() != 0).then(|| edid.numeric_serial().to_string()));

    let connector_path = format!(
        "{}#{}",
        connector.gpu_sysfs_path().display(),
        connector.connector_name()
    );

    DisplayIdentityEvidence::new(
        connector_path,
        edid.manufacturer(),
        edid.product_code(),
        serial,
    )
}

#[cfg(test)]
mod tests {
    use super::evidence_from_observation;
    use crate::drm::{connector_identity::DrmConnectorIdentity, edid::EdidInfo};
    use std::path::PathBuf;

    fn edid_with_serial(serial: u32) -> EdidInfo {
        let mut bytes = vec![0_u8; 128];
        bytes[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
        let vendor = (4_u16 << 10) | (5_u16 << 5) | 12_u16;
        bytes[8..10].copy_from_slice(&vendor.to_be_bytes());
        bytes[10..12].copy_from_slice(&0x1234_u16.to_le_bytes());
        bytes[12..16].copy_from_slice(&serial.to_le_bytes());
        let checksum = bytes[..127]
            .iter()
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
        bytes[127] = 0_u8.wrapping_sub(checksum);
        EdidInfo::parse(&bytes).unwrap()
    }

    fn connector() -> DrmConnectorIdentity {
        DrmConnectorIdentity::resolve(
            std::path::Path::new("/dev/dri/card0"),
            "DP-1",
            &PathBuf::from("/nonexistent-sysfs-root"),
        )
        .unwrap_or_else(|| DrmConnectorIdentity {
            gpu_sysfs_path: PathBuf::from("/sys/devices/pci0000:00/0000:00:02.0"),
            connector_name: "DP-1".to_owned(),
        })
    }

    #[test]
    fn combines_validated_edid_and_connector_path() {
        let evidence = evidence_from_observation(&edid_with_serial(42), &connector()).unwrap();
        assert_eq!(evidence.manufacturer(), *b"DEL");
        assert_eq!(evidence.product_code(), 0x1234);
        assert_eq!(evidence.serial(), Some("42"));
        assert!(evidence.connector_path().ends_with("#DP-1"));
    }

    #[test]
    fn missing_unit_serial_produces_reviewable_evidence() {
        let evidence = evidence_from_observation(&edid_with_serial(0), &connector()).unwrap();
        assert!(!evidence.has_unit_serial());
    }
}
