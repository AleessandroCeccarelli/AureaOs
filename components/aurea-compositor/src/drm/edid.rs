// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — DRM EDID inspection

//! Bounded validation and extraction of monitor identity fields from EDID.

/// Parsed, validated EDID metadata. Raw bytes are retained for later
/// fingerprinting by the color domain; this type is not a persistent ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdidInfo {
    raw: Vec<u8>,
    manufacturer: [u8; 3],
    product_code: u16,
    numeric_serial: u32,
    monitor_name: Option<String>,
    serial_number: Option<String>,
}

impl EdidInfo {
    /// Parses a base EDID block and all declared extension blocks.
    /// Invalid lengths, headers, extension counts, or checksums are rejected.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        const HEADER: [u8; 8] = [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];
        if bytes.len() < 128 || bytes.len() % 128 != 0 || bytes.len() > 32 * 128 {
            return None;
        }
        if bytes[..8] != HEADER {
            return None;
        }
        let expected_blocks = usize::from(bytes[126]).checked_add(1)?;
        if bytes.len() != expected_blocks * 128 {
            return None;
        }
        if bytes
            .chunks_exact(128)
            .any(|block| block.iter().fold(0_u8, |sum, byte| sum.wrapping_add(*byte)) != 0)
        {
            return None;
        }

        let encoded_vendor = u16::from_be_bytes([bytes[8], bytes[9]]);
        let vendor = [
            ((encoded_vendor >> 10) & 0x1f) as u8,
            ((encoded_vendor >> 5) & 0x1f) as u8,
            (encoded_vendor & 0x1f) as u8,
        ];
        if vendor.iter().any(|value| *value == 0 || *value > 26) {
            return None;
        }

        let mut info = Self {
            raw: bytes.to_vec(),
            manufacturer: [
                vendor[0] + b'A' - 1,
                vendor[1] + b'A' - 1,
                vendor[2] + b'A' - 1,
            ],
            product_code: u16::from_le_bytes([bytes[10], bytes[11]]),
            numeric_serial: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            monitor_name: None,
            serial_number: None,
        };
        for descriptor in bytes[54..126].chunks_exact(18) {
            if descriptor[..3] != [0, 0, 0] || descriptor[4] != 0 {
                continue;
            }
            let value = String::from_utf8_lossy(&descriptor[5..18])
                .trim_matches(|character: char| {
                    character == '\0' || character.is_ascii_whitespace()
                })
                .to_owned();
            if value.is_empty() {
                continue;
            }
            match descriptor[3] {
                0xfc => info.monitor_name = Some(value),
                0xff => info.serial_number = Some(value),
                _ => {}
            }
        }
        Some(info)
    }

    /// Returns the exact validated EDID bytes for domain-level fingerprinting.
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }
    pub fn manufacturer(&self) -> [u8; 3] {
        self.manufacturer
    }
    pub fn product_code(&self) -> u16 {
        self.product_code
    }
    pub fn numeric_serial(&self) -> u32 {
        self.numeric_serial
    }
    pub fn monitor_name(&self) -> Option<&str> {
        self.monitor_name.as_deref()
    }
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    /// A serial is useful evidence, but never sufficient by itself to identify
    /// a physical monitor or authorize automatic profile assignment.
    pub fn has_serial_evidence(&self) -> bool {
        self.numeric_serial != 0 || self.serial_number.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::EdidInfo;

    fn sample_edid() -> Vec<u8> {
        let mut bytes = vec![0_u8; 128];
        bytes[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
        // "DEL" encoded as 5-bit EISA manufacturer code.
        let vendor = (4_u16 << 10) | (5_u16 << 5) | 12_u16;
        bytes[8..10].copy_from_slice(&vendor.to_be_bytes());
        bytes[10..12].copy_from_slice(&0x1234_u16.to_le_bytes());
        bytes[12..16].copy_from_slice(&0x5678_u32.to_le_bytes());
        bytes[54..59].copy_from_slice(&[0, 0, 0, 0xfc, 0]);
        bytes[59..72].copy_from_slice(b"Aurea Monitor");
        let checksum = bytes[..127]
            .iter()
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
        bytes[127] = 0_u8.wrapping_sub(checksum);
        bytes
    }

    #[test]
    fn parses_manufacturer_product_and_serial() {
        let parsed = EdidInfo::parse(&sample_edid()).unwrap();
        assert_eq!(&parsed.manufacturer(), b"DEL");
        assert_eq!(parsed.product_code(), 0x1234);
        assert_eq!(parsed.numeric_serial(), 0x5678);
        assert!(parsed.has_serial_evidence());
        assert_eq!(parsed.monitor_name(), Some("Aurea Monitor"));
    }

    #[test]
    fn rejects_invalid_header_and_checksum() {
        let mut bytes = sample_edid();
        bytes[0] = 1;
        assert!(EdidInfo::parse(&bytes).is_none());
        let mut bytes = sample_edid();
        bytes[20] ^= 1;
        assert!(EdidInfo::parse(&bytes).is_none());
    }

    #[test]
    fn rejects_mismatched_extension_count() {
        let mut bytes = sample_edid();
        bytes[126] = 1;
        assert!(EdidInfo::parse(&bytes).is_none());
    }

    #[test]
    fn does_not_treat_missing_serial_as_unique_identity() {
        let mut bytes = sample_edid();
        bytes[12..16].fill(0);
        let checksum = bytes[..127]
            .iter()
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
        bytes[127] = 0_u8.wrapping_sub(checksum);
        assert!(!EdidInfo::parse(&bytes).unwrap().has_serial_evidence());
    }
}
