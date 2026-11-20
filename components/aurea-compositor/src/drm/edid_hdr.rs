// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! CTA-861 HDR and wide-gamut sink capability parsing.
//!
//! This describes only what the monitor advertises in EDID. It deliberately
//! does not claim that the active GPU, connector, encoder, or link can carry
//! the same signal; those capabilities must be intersected separately.

/// HDR capabilities advertised by CTA-861 data blocks in a monitor EDID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EdidHdrCapabilities {
    bt2020_rgb: bool,
    bt2020_ycc: bool,
    pq: bool,
    hlg: bool,
    static_metadata_type_1: bool,
    max_luminance_millinit: Option<u32>,
    max_frame_average_luminance_millinit: Option<u32>,
    min_luminance_millinit: Option<u32>,
}

impl EdidHdrCapabilities {
    /// Parses CTA-861 extension blocks from a complete, checksum-validated EDID.
    ///
    /// Invalid or truncated CTA data blocks are ignored safely. Other valid
    /// extension blocks do not affect the result.
    pub fn parse(edid: &[u8]) -> Self {
        if edid.len() < 128 || edid.len() % 128 != 0 {
            return Self::default();
        }

        let mut capabilities = Self::default();
        for extension in edid.chunks_exact(128).skip(1) {
            if extension[0] != CTA_EXTENSION_TAG || extension[1] < CTA_REVISION_MIN {
                continue;
            }
            parse_cta_data_blocks(extension, &mut capabilities);
        }
        capabilities
    }

    /// Returns whether the sink advertises BT.2020 RGB.
    pub const fn supports_bt2020_rgb(self) -> bool {
        self.bt2020_rgb
    }

    /// Returns whether the sink advertises BT.2020 YCC.
    pub const fn supports_bt2020_ycc(self) -> bool {
        self.bt2020_ycc
    }

    /// Returns whether the sink advertises either BT.2020 representation.
    pub const fn supports_bt2020(self) -> bool {
        self.bt2020_rgb || self.bt2020_ycc
    }

    /// Returns whether the sink advertises SMPTE ST 2084 (PQ).
    pub const fn supports_pq(self) -> bool {
        self.pq
    }

    /// Returns whether the sink advertises Hybrid Log-Gamma.
    pub const fn supports_hlg(self) -> bool {
        self.hlg
    }

    /// Returns whether static metadata descriptor type 1 is advertised.
    pub const fn supports_static_metadata_type_1(self) -> bool {
        self.static_metadata_type_1
    }

    /// Returns the advertised desired content maximum luminance, if present.
    pub const fn max_luminance_millinit(self) -> Option<u32> {
        self.max_luminance_millinit
    }

    /// Returns the advertised desired content MaxFALL, if present.
    pub const fn max_frame_average_luminance_millinit(self) -> Option<u32> {
        self.max_frame_average_luminance_millinit
    }

    /// Returns the advertised desired content minimum luminance, if present.
    pub const fn min_luminance_millinit(self) -> Option<u32> {
        self.min_luminance_millinit
    }
}

const CTA_EXTENSION_TAG: u8 = 0x02;
const CTA_REVISION_MIN: u8 = 3;
const CTA_DATA_BLOCK_START: usize = 4;
const CTA_TAG_EXTENDED: u8 = 0x07;
const CTA_EXT_COLORIMETRY: u8 = 0x05;
const CTA_EXT_HDR_STATIC_METADATA: u8 = 0x06;

fn parse_cta_data_blocks(extension: &[u8], capabilities: &mut EdidHdrCapabilities) {
    let end = usize::from(extension[2]);
    if end == 0 || !(CTA_DATA_BLOCK_START..=127).contains(&end) {
        return;
    }

    let mut cursor = CTA_DATA_BLOCK_START;
    while cursor < end {
        let header = extension[cursor];
        let tag = header >> 5;
        let length = usize::from(header & 0x1f);
        let payload_start = cursor + 1;
        let payload_end = match payload_start.checked_add(length) {
            Some(value) if value <= end => value,
            _ => break,
        };
        let payload = &extension[payload_start..payload_end];

        if tag == CTA_TAG_EXTENDED {
            parse_extended_data_block(payload, capabilities);
        }
        cursor = payload_end;
    }
}

fn parse_extended_data_block(payload: &[u8], capabilities: &mut EdidHdrCapabilities) {
    let Some((&extended_tag, data)) = payload.split_first() else {
        return;
    };

    match extended_tag {
        CTA_EXT_COLORIMETRY if data.len() >= 2 => {
            // CTA-861 Colorimetry Data Block: byte 0 bit 7 = BT.2020 RGB,
            // bit 6 = BT.2020 YCC.
            capabilities.bt2020_rgb |= data[0] & (1 << 7) != 0;
            capabilities.bt2020_ycc |= data[0] & (1 << 6) != 0;
        }
        CTA_EXT_HDR_STATIC_METADATA if data.len() >= 2 => {
            parse_hdr_static_metadata(data, capabilities);
        }
        _ => {}
    }
}

fn parse_hdr_static_metadata(data: &[u8], capabilities: &mut EdidHdrCapabilities) {
    let eotf = data[0];
    let descriptors = data[1];

    // CTA-861 HDR Static Metadata Data Block: EOTF bits 2 and 3 are PQ
    // and HLG; descriptor bit 0 identifies static metadata type 1.
    capabilities.pq |= eotf & (1 << 2) != 0;
    capabilities.hlg |= eotf & (1 << 3) != 0;
    capabilities.static_metadata_type_1 |= descriptors & 1 != 0;

    if let Some(raw) = data.get(2).copied().filter(|value| *value != 0) {
        capabilities.max_luminance_millinit = Some(decode_max_luminance(raw));
    }
    if let Some(raw) = data.get(3).copied().filter(|value| *value != 0) {
        capabilities.max_frame_average_luminance_millinit =
            Some(decode_max_luminance(raw));
    }
    if let (Some(maximum), Some(raw)) = (
        capabilities.max_luminance_millinit,
        data.get(4).copied().filter(|value| *value != 0),
    ) {
        capabilities.min_luminance_millinit =
            Some(decode_min_luminance(maximum, raw));
    }
}

fn decode_max_luminance(raw: u8) -> u32 {
    let nits = 50.0_f64 * 2.0_f64.powf(f64::from(raw) / 32.0);
    (nits * 1_000.0).round().min(f64::from(u32::MAX)) as u32
}

fn decode_min_luminance(maximum_millinit: u32, raw: u8) -> u32 {
    let ratio = f64::from(raw) / 255.0;
    let nits = (f64::from(maximum_millinit) / 1_000.0) * ratio * ratio / 100.0;
    (nits * 1_000.0).round().min(f64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::EdidHdrCapabilities;

    const CTA_TAG: u8 = 0x02;

    fn edid_with_cta(blocks: &[(u8, &[u8])]) -> Vec<u8> {
        let mut edid = vec![0_u8; 256];
        edid[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
        // Valid "DEL" manufacturer code.
        let vendor = (4_u16 << 10) | (5_u16 << 5) | 12_u16;
        edid[8..10].copy_from_slice(&vendor.to_be_bytes());
        edid[126] = 1;

        let extension = &mut edid[128..256];
        extension[0] = CTA_TAG;
        extension[1] = 3;
        let mut cursor = 4;
        for (tag, payload) in blocks {
            extension[cursor] = (tag << 5) | payload.len() as u8;
            cursor += 1;
            extension[cursor..cursor + payload.len()].copy_from_slice(payload);
            cursor += payload.len();
        }
        extension[2] = cursor as u8;

        for block_start in [0, 128] {
            let checksum = edid[block_start..block_start + 127]
                .iter()
                .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
            edid[block_start + 127] = 0_u8.wrapping_sub(checksum);
        }
        edid
    }

    fn hdr_edid() -> Vec<u8> {
        edid_with_cta(&[
            (7, &[0x05, 0xc0, 0x00]),
            (7, &[0x06, 0x0c, 0x01, 0x64, 0x40, 0x20]),
        ])
    }

    #[test]
    fn detects_bt2020_pq_hlg_and_static_metadata() {
        let parsed = EdidHdrCapabilities::parse(&hdr_edid());
        assert!(parsed.supports_bt2020_rgb());
        assert!(parsed.supports_bt2020_ycc());
        assert!(parsed.supports_pq());
        assert!(parsed.supports_hlg());
        assert!(parsed.supports_static_metadata_type_1());
    }

    #[test]
    fn decodes_optional_luminance_values() {
        let parsed = EdidHdrCapabilities::parse(&hdr_edid());
        assert!(parsed.max_luminance_millinit().is_some());
        assert!(parsed.max_frame_average_luminance_millinit().is_some());
        assert!(parsed.min_luminance_millinit().is_some());
    }

    #[test]
    fn ignores_non_cta_extensions_and_unknown_extended_tags() {
        let mut edid = hdr_edid();
        edid[128] = 0x70;
        let parsed = EdidHdrCapabilities::parse(&edid);
        assert!(!parsed.supports_pq());
        assert!(!parsed.supports_bt2020());
    }

    #[test]
    fn malformed_data_block_does_not_read_past_collection() {
        let mut edid = hdr_edid();
        edid[132] = (7 << 5) | 31;
        let parsed = EdidHdrCapabilities::parse(&edid);
        assert!(!parsed.supports_pq());
        assert!(!parsed.supports_bt2020());
    }

    #[test]
    fn missing_hdr_metadata_does_not_imply_hdr_support() {
        let edid = edid_with_cta(&[(7, &[0x05, 0x80, 0x00])]);
        let parsed = EdidHdrCapabilities::parse(&edid);
        assert!(parsed.supports_bt2020());
        assert!(!parsed.supports_pq());
        assert!(!parsed.supports_hlg());
    }
}
