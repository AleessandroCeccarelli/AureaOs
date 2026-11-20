// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Technology-independent HDR capabilities and automatic output policy.
//!
//! This module describes the signal presented by the composed frame, not the
//! codec used by an individual application. The compositor must aggregate
//! visible layers before asking this policy to select an output mode.

/// HDR transfer or content encoding carried by a composed frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayContentEncoding {
    /// Standard-dynamic-range content (normally sRGB / BT.709).
    Sdr,
    /// HDR10 content using BT.2020 primaries and SMPTE ST 2084 (PQ).
    Hdr10Pq,
    /// Hybrid Log-Gamma HDR content.
    Hlg,
    /// Extended linear sRGB content.
    ScRgb,
    /// HDR10+ content; its compatible static base is HDR10 PQ.
    Hdr10Plus,
}

/// HDR output signaling selected for one physical display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayHdrOutputMode {
    /// SDR output. HDR sources must be tone-mapped before presentation.
    Sdr,
    /// HDR10 static metadata signaling with PQ transfer.
    Hdr10Pq,
    /// HLG transfer signaling.
    Hlg,
    /// Extended linear sRGB output.
    ScRgb,
}

/// Capabilities verified for one connected display and its active link.
///
/// Luminance values use milli-nits to avoid floating-point values in the
/// technology-independent API. Unknown measurements remain `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayHdrCapabilities {
    pub supports_bt2020: bool,
    pub supports_pq: bool,
    pub supports_hlg: bool,
    pub supports_scrgb: bool,
    pub supports_static_metadata: bool,
    pub min_luminance_millinit: Option<u32>,
    pub max_luminance_millinit: Option<u32>,
    pub max_frame_average_luminance_millinit: Option<u32>,
}

impl DisplayHdrCapabilities {
    /// Returns whether the display and link expose a usable HDR10 PQ path.
    pub const fn supports_hdr10(self) -> bool {
        self.supports_bt2020 && self.supports_pq && self.supports_static_metadata
    }

    /// Returns whether at least one HDR output encoding is available.
    pub const fn supports_any_hdr(self) -> bool {
        self.supports_hdr10() || self.supports_hlg() || self.supports_scrgb
    }

    /// Returns whether HLG can be signaled without conversion to another HDR
    /// encoding. HLG requires BT.2020 primaries and HLG transfer support.
    pub const fn supports_hlg(self) -> bool {
        self.supports_bt2020 && self.supports_hlg
    }

    /// Validates luminance ordering when both ends of the range are known.
    pub const fn has_valid_luminance_range(self) -> bool {
        match (self.min_luminance_millinit, self.max_luminance_millinit) {
            (Some(minimum), Some(maximum)) => minimum <= maximum,
            _ => true,
        }
    }
}

/// Result of evaluating the composed frame against one display's capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayHdrDecision {
    pub output_mode: DisplayHdrOutputMode,
    /// True when the compositor must tone-map the source into the output mode.
    pub requires_tone_mapping: bool,
    /// True when dynamic metadata cannot be forwarded and the static HDR10
    /// base layer must be used instead.
    pub drops_dynamic_metadata: bool,
}

/// Stateless automatic policy. SDR content deliberately remains SDR even on
/// an HDR-capable display; HDR is enabled only when the composed frame needs it.
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayHdrPolicy;

impl DisplayHdrPolicy {
    pub const fn decide(
        capabilities: DisplayHdrCapabilities,
        content: DisplayContentEncoding,
    ) -> DisplayHdrDecision {
        match content {
            DisplayContentEncoding::Sdr => DisplayHdrDecision {
                output_mode: DisplayHdrOutputMode::Sdr,
                requires_tone_mapping: false,
                drops_dynamic_metadata: false,
            },
            DisplayContentEncoding::Hdr10Pq => {
                if capabilities.supports_hdr10() {
                    direct(DisplayHdrOutputMode::Hdr10Pq)
                } else {
                    tone_map_to_sdr()
                }
            }
            DisplayContentEncoding::Hdr10Plus => {
                if capabilities.supports_hdr10() {
                    DisplayHdrDecision {
                        output_mode: DisplayHdrOutputMode::Hdr10Pq,
                        requires_tone_mapping: false,
                        drops_dynamic_metadata: true,
                    }
                } else {
                    tone_map_to_sdr()
                }
            }
            DisplayContentEncoding::Hlg => {
                if capabilities.supports_hlg() {
                    direct(DisplayHdrOutputMode::Hlg)
                } else {
                    tone_map_to_sdr()
                }
            }
            DisplayContentEncoding::ScRgb => {
                if capabilities.supports_scrgb {
                    direct(DisplayHdrOutputMode::ScRgb)
                } else {
                    tone_map_to_sdr()
                }
            }
        }
    }
}

const fn direct(mode: DisplayHdrOutputMode) -> DisplayHdrDecision {
    DisplayHdrDecision {
        output_mode: mode,
        requires_tone_mapping: false,
        drops_dynamic_metadata: false,
    }
}

const fn tone_map_to_sdr() -> DisplayHdrDecision {
    DisplayHdrDecision {
        output_mode: DisplayHdrOutputMode::Sdr,
        requires_tone_mapping: true,
        drops_dynamic_metadata: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayContentEncoding as Content, DisplayHdrCapabilities as Capabilities,
        DisplayHdrOutputMode as Output, DisplayHdrPolicy,
    };

    fn hdr_display() -> Capabilities {
        Capabilities {
            supports_bt2020: true,
            supports_pq: true,
            supports_hlg: true,
            supports_scrgb: true,
            supports_static_metadata: true,
            min_luminance_millinit: Some(50),
            max_luminance_millinit: Some(1_000_000),
            max_frame_average_luminance_millinit: Some(400_000),
        }
    }

    #[test]
    fn sdr_content_stays_sdr_on_an_hdr_display() {
        let decision = DisplayHdrPolicy::decide(hdr_display(), Content::Sdr);
        assert_eq!(decision.output_mode, Output::Sdr);
        assert!(!decision.requires_tone_mapping);
    }

    #[test]
    fn hdr10_content_uses_pq_when_the_display_supports_it() {
        let decision = DisplayHdrPolicy::decide(hdr_display(), Content::Hdr10Pq);
        assert_eq!(decision.output_mode, Output::Hdr10Pq);
        assert!(!decision.requires_tone_mapping);
    }

    #[test]
    fn hdr_content_is_tone_mapped_for_an_sdr_only_display() {
        let decision = DisplayHdrPolicy::decide(Capabilities::default(), Content::Hdr10Pq);
        assert_eq!(decision.output_mode, Output::Sdr);
        assert!(decision.requires_tone_mapping);
    }

    #[test]
    fn hdr10_plus_falls_back_to_its_static_hdr10_base() {
        let decision = DisplayHdrPolicy::decide(hdr_display(), Content::Hdr10Plus);
        assert_eq!(decision.output_mode, Output::Hdr10Pq);
        assert!(decision.drops_dynamic_metadata);
        assert!(!decision.requires_tone_mapping);
    }

    #[test]
    fn hlg_requires_its_own_signal_support() {
        let mut capabilities = hdr_display();
        capabilities.supports_hlg = false;
        let decision = DisplayHdrPolicy::decide(capabilities, Content::Hlg);
        assert!(decision.requires_tone_mapping);
    }
}
