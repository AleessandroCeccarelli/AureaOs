// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Conservative HDR capability resolution when sink declarations are missing
//! or incomplete.

use super::hdr::DisplayHdrCapabilities;

/// How the physical display describes its HDR capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrSinkDeclaration {
    /// The sink explicitly advertises the supplied capabilities.
    Advertised,
    /// The sink advertises some color features but omits HDR declarations.
    Incomplete,
    /// No usable HDR declaration was found in EDID/DisplayID.
    NotAdvertised,
    /// Available declarations disagree or cannot be trusted.
    Conflicting,
}

/// Capabilities supplied by a user or a trusted monitor-specific profile.
///
/// These values are claims about the sink, not proof that the active GPU/link
/// can transmit the requested signal. They must be bound to a physical display
/// identity and explicitly confirmed by the user before use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayHdrManualOverride {
    pub capabilities: DisplayHdrCapabilities,
    pub confirmed: bool,
}

/// Origin of the effective HDR capabilities returned by the resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayHdrCapabilitySource {
    /// The sink declaration and active output path were both verified.
    Automatic,
    /// The user confirmed a manual sink description and the output path passed
    /// independent verification.
    Manual,
    /// The sink declaration is incomplete, absent, or conflicting.
    NeedsReview,
    /// The output path has not been independently verified.
    LinkUnverified,
}

/// Conservative result used by the HDR output policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayHdrResolution {
    pub capabilities: DisplayHdrCapabilities,
    pub source: DisplayHdrCapabilitySource,
}

impl DisplayHdrResolution {
    /// Returns true when the shell should offer advanced/manual HDR setup.
    pub const fn needs_manual_review(self) -> bool {
        matches!(
            self.source,
            DisplayHdrCapabilitySource::NeedsReview
                | DisplayHdrCapabilitySource::LinkUnverified
        )
    }
}

/// Resolves sink declarations against independently verified link support.
///
/// Missing EDID/DisplayID metadata never proves that a monitor is SDR. It does,
/// however, prevent automatic HDR activation. A confirmed manual override can
/// describe the sink, but cannot bypass verification of the active DRM/KMS
/// output path.
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayHdrCapabilityResolver;

impl DisplayHdrCapabilityResolver {
    pub fn resolve(
        declaration: HdrSinkDeclaration,
        advertised: DisplayHdrCapabilities,
        link: Option<DisplayHdrCapabilities>,
        manual: Option<DisplayHdrManualOverride>,
    ) -> DisplayHdrResolution {
        let Some(link) = link else {
            return resolution(
                DisplayHdrCapabilities::default(),
                DisplayHdrCapabilitySource::LinkUnverified,
            );
        };

        match declaration {
            HdrSinkDeclaration::Advertised => resolution(
                intersect(advertised, link),
                DisplayHdrCapabilitySource::Automatic,
            ),
            HdrSinkDeclaration::Incomplete
            | HdrSinkDeclaration::NotAdvertised
            | HdrSinkDeclaration::Conflicting => {
                if let Some(override_) = manual.filter(|value| value.confirmed) {
                    resolution(
                        intersect(override_.capabilities, link),
                        DisplayHdrCapabilitySource::Manual,
                    )
                } else {
                    resolution(
                        DisplayHdrCapabilities::default(),
                        DisplayHdrCapabilitySource::NeedsReview,
                    )
                }
            }
        }
    }
}

fn resolution(
    capabilities: DisplayHdrCapabilities,
    source: DisplayHdrCapabilitySource,
) -> DisplayHdrResolution {
    DisplayHdrResolution {
        capabilities,
        source,
    }
}

fn intersect(
    sink: DisplayHdrCapabilities,
    link: DisplayHdrCapabilities,
) -> DisplayHdrCapabilities {
    DisplayHdrCapabilities {
        supports_bt2020: sink.supports_bt2020 && link.supports_bt2020,
        supports_pq: sink.supports_pq && link.supports_pq,
        supports_hlg: sink.supports_hlg && link.supports_hlg,
        supports_scrgb: sink.supports_scrgb && link.supports_scrgb,
        supports_static_metadata: sink.supports_static_metadata
            && link.supports_static_metadata,
        min_luminance_millinit: sink.min_luminance_millinit,
        max_luminance_millinit: sink.max_luminance_millinit,
        max_frame_average_luminance_millinit: sink.max_frame_average_luminance_millinit,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayHdrCapabilityResolver, DisplayHdrCapabilitySource, DisplayHdrManualOverride,
        HdrSinkDeclaration,
    };
    use crate::display::hdr::DisplayHdrCapabilities;

    fn hdr_capabilities() -> DisplayHdrCapabilities {
        DisplayHdrCapabilities {
            supports_bt2020: true,
            supports_pq: true,
            supports_hlg: true,
            supports_scrgb: false,
            supports_static_metadata: true,
            min_luminance_millinit: Some(50),
            max_luminance_millinit: Some(1_000_000),
            max_frame_average_luminance_millinit: Some(400_000),
        }
    }

    #[test]
    fn advertised_capabilities_require_a_verified_link() {
        let result = DisplayHdrCapabilityResolver::resolve(
            HdrSinkDeclaration::Advertised,
            hdr_capabilities(),
            Some(hdr_capabilities()),
            None,
        );
        assert_eq!(result.source, DisplayHdrCapabilitySource::Automatic);
        assert!(result.capabilities.supports_hdr10());
    }

    #[test]
    fn missing_metadata_is_not_treated_as_proof_of_sdr_or_hdr() {
        let result = DisplayHdrCapabilityResolver::resolve(
            HdrSinkDeclaration::NotAdvertised,
            DisplayHdrCapabilities::default(),
            Some(hdr_capabilities()),
            None,
        );
        assert_eq!(result.source, DisplayHdrCapabilitySource::NeedsReview);
        assert!(result.needs_manual_review());
        assert!(!result.capabilities.supports_any_hdr());
    }

    #[test]
    fn incomplete_declaration_can_be_recovered_by_confirmed_override() {
        let result = DisplayHdrCapabilityResolver::resolve(
            HdrSinkDeclaration::Incomplete,
            DisplayHdrCapabilities::default(),
            Some(hdr_capabilities()),
            Some(DisplayHdrManualOverride {
                capabilities: hdr_capabilities(),
                confirmed: true,
            }),
        );
        assert_eq!(result.source, DisplayHdrCapabilitySource::Manual);
        assert!(result.capabilities.supports_hdr10());
    }

    #[test]
    fn unconfirmed_override_never_enables_hdr() {
        let result = DisplayHdrCapabilityResolver::resolve(
            HdrSinkDeclaration::NotAdvertised,
            DisplayHdrCapabilities::default(),
            Some(hdr_capabilities()),
            Some(DisplayHdrManualOverride {
                capabilities: hdr_capabilities(),
                confirmed: false,
            }),
        );
        assert_eq!(result.source, DisplayHdrCapabilitySource::NeedsReview);
        assert!(!result.capabilities.supports_any_hdr());
    }

    #[test]
    fn manual_override_cannot_bypass_unverified_output_path() {
        let result = DisplayHdrCapabilityResolver::resolve(
            HdrSinkDeclaration::NotAdvertised,
            DisplayHdrCapabilities::default(),
            None,
            Some(DisplayHdrManualOverride {
                capabilities: hdr_capabilities(),
                confirmed: true,
            }),
        );
        assert_eq!(result.source, DisplayHdrCapabilitySource::LinkUnverified);
        assert!(!result.capabilities.supports_any_hdr());
    }

    #[test]
    fn manual_sink_claim_is_intersected_with_verified_link() {
        let mut link = hdr_capabilities();
        link.supports_pq = false;
        let result = DisplayHdrCapabilityResolver::resolve(
            HdrSinkDeclaration::Conflicting,
            DisplayHdrCapabilities::default(),
            Some(link),
            Some(DisplayHdrManualOverride {
                capabilities: hdr_capabilities(),
                confirmed: true,
            }),
        );
        assert_eq!(result.source, DisplayHdrCapabilitySource::Manual);
        assert!(!result.capabilities.supports_hdr10());
        assert!(result.capabilities.supports_hlg());
    }
}
