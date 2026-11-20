// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Intersection of monitor HDR declarations and verified DRM link support.

use aurea_desktop_api::display::DisplayHdrCapabilities;

use super::edid_hdr::EdidHdrCapabilities;

/// HDR signal features verified for the active connector-to-display path.
///
/// These flags must come from DRM/KMS capability discovery and successful
/// validation of the active output configuration. They must never be copied
/// from EDID: EDID describes the sink, not the GPU or transport path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrmHdrLinkCapabilities {
    bt2020: bool,
    pq: bool,
    hlg: bool,
    static_metadata_type_1: bool,
}

impl DrmHdrLinkCapabilities {
    /// Creates link evidence from capabilities independently verified by KMS.
    pub const fn verified(
        bt2020: bool,
        pq: bool,
        hlg: bool,
        static_metadata_type_1: bool,
    ) -> Self {
        Self {
            bt2020,
            pq,
            hlg,
            static_metadata_type_1,
        }
    }

    /// Returns the conservative intersection for the connected sink and link.
    ///
    /// A feature is enabled only when both the monitor and the active link
    /// advertise it. Unknown or unverified link support remains disabled.
    pub fn intersect(
        self,
        sink: EdidHdrCapabilities,
    ) -> DisplayHdrCapabilities {
        let supports_bt2020 = self.bt2020 && sink.supports_bt2020();
        let supports_pq = self.pq && sink.supports_pq();
        let supports_hlg = self.hlg && sink.supports_hlg();
        let supports_static_metadata =
            self.static_metadata_type_1 && sink.supports_static_metadata_type_1();

        DisplayHdrCapabilities {
            supports_bt2020,
            supports_pq,
            supports_hlg,
            // scRGB is deliberately not inferred from EDID or link presence.
            // It will be enabled only after the complete linear-output path
            // and sink acceptance are independently represented here.
            supports_scrgb: false,
            supports_static_metadata,
            min_luminance_millinit: sink.min_luminance_millinit(),
            max_luminance_millinit: sink.max_luminance_millinit(),
            max_frame_average_luminance_millinit:
                sink.max_frame_average_luminance_millinit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DrmHdrLinkCapabilities;
    use crate::drm::edid_hdr::EdidHdrCapabilities;

    fn hdr_sink() -> EdidHdrCapabilities {
        let mut edid = vec![0_u8; 256];
        edid[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
        let vendor = (4_u16 << 10) | (5_u16 << 5) | 12_u16;
        edid[8..10].copy_from_slice(&vendor.to_be_bytes());
        edid[126] = 1;

        let extension = &mut edid[128..];
        extension[0] = 0x02;
        extension[1] = 3;
        extension[2] = 14;
        extension[4] = (7 << 5) | 3;
        extension[5..8].copy_from_slice(&[0x05, 0xc0, 0x00]);
        extension[8] = (7 << 5) | 6;
        extension[9..15].copy_from_slice(&[0x06, 0x0c, 0x01, 0x64, 0x40, 0x20]);

        for start in [0, 128] {
            let sum = edid[start..start + 127]
                .iter()
                .fold(0_u8, |sum, byte| sum.wrapping_add(*byte));
            edid[start + 127] = 0_u8.wrapping_sub(sum);
        }
        EdidHdrCapabilities::parse(&edid)
    }

    #[test]
    fn unverified_link_never_exposes_hdr() {
        let result = DrmHdrLinkCapabilities::default().intersect(hdr_sink());
        assert!(!result.supports_hdr10());
        assert!(!result.supports_hlg());
        assert!(!result.supports_scrgb);
    }

    #[test]
    fn hdr10_requires_sink_and_link_support_for_every_feature() {
        let sink = hdr_sink();
        let link = DrmHdrLinkCapabilities::verified(true, true, true, true);
        assert!(link.intersect(sink).supports_hdr10());

        let missing_metadata =
            DrmHdrLinkCapabilities::verified(true, true, true, false).intersect(sink);
        assert!(!missing_metadata.supports_hdr10());
    }

    #[test]
    fn hlg_is_intersected_independently_from_pq() {
        let sink = hdr_sink();
        let link = DrmHdrLinkCapabilities::verified(true, false, true, false);
        let result = link.intersect(sink);
        assert!(!result.supports_hdr10());
        assert!(result.supports_hlg());
    }
}
