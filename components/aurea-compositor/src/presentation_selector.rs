// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Selects the presentation path for a rendering/presentation GPU pair.

use std::{error::Error, fmt};

/// Presentation paths available to the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationPath {
    /// Direct Vulkan rendering with DMA-BUF scanout.
    DirectVulkan,
    /// Vulkan rendering followed by CPU readback and GBM/KMS presentation.
    Compatibility,
}

/// Capabilities already established by the respective backend adapters.
///
/// Values describe the exact rendering-GPU/presentation-GPU pair. This type
/// performs no probing and owns no graphics or KMS resources.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PresentationBackendSupport {
    pub direct_vulkan: bool,
    pub compatibility: bool,
}

/// Failure returned when neither backend supports the requested GPU pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoCompatiblePresentationPath;

impl fmt::Display for NoCompatiblePresentationPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "No presentation backend supports the selected rendering and output GPUs",
        )
    }
}

impl Error for NoCompatiblePresentationPath {}

/// Stateless router for presentation backend selection.
#[derive(Debug, Default, Clone, Copy)]
pub struct PresentationSelector;

impl PresentationSelector {
    /// Selects the preferred available path for one GPU pair.
    ///
    /// Direct Vulkan is preferred. Compatibility is selected only when the
    /// direct path is unavailable and the compatibility backend reports
    /// support for this same pair.
    pub fn select(
        support: PresentationBackendSupport,
    ) -> Result<PresentationPath, NoCompatiblePresentationPath> {
        if support.direct_vulkan {
            return Ok(PresentationPath::DirectVulkan);
        }

        if support.compatibility {
            return Ok(PresentationPath::Compatibility);
        }

        Err(NoCompatiblePresentationPath)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_direct_vulkan_when_both_paths_are_available() {
        assert_eq!(
            PresentationSelector::select(PresentationBackendSupport {
                direct_vulkan: true,
                compatibility: true,
            }),
            Ok(PresentationPath::DirectVulkan)
        );
    }

    #[test]
    fn selects_compatibility_when_direct_path_is_unavailable() {
        assert_eq!(
            PresentationSelector::select(PresentationBackendSupport {
                direct_vulkan: false,
                compatibility: true,
            }),
            Ok(PresentationPath::Compatibility)
        );
    }

    #[test]
    fn rejects_gpu_pair_without_a_supported_path() {
        assert_eq!(
            PresentationSelector::select(PresentationBackendSupport::default()),
            Err(NoCompatiblePresentationPath)
        );
    }
}
