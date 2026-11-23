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

/// Capabilities declared by backend adapters for one exact GPU pair.
///
/// GPU identities are supplied by the existing device managers. This record
/// owns no device, graphics resource, or KMS lifecycle and performs no probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationBackendSupport<G> {
    pub rendering_gpu: G,
    pub presentation_gpu: G,
    pub direct_vulkan: bool,
    pub compatibility: bool,
}

/// Failure returned when neither backend supports the requested GPU pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoCompatiblePresentationPath;

impl fmt::Display for NoCompatiblePresentationPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .write_str("No presentation backend supports the selected rendering and output GPUs")
    }
}

impl Error for NoCompatiblePresentationPath {}

/// Stateless router for presentation backend selection.
#[derive(Debug, Default, Clone, Copy)]
pub struct PresentationSelector;

impl PresentationSelector {
    /// Selects the preferred path for the exact rendering/output GPU pair.
    ///
    /// A capability record for another pair is never considered. Direct
    /// Vulkan is preferred; compatibility is used only when direct scanout
    /// is unavailable for this pair.
    pub fn select<G: Eq>(
        rendering_gpu: &G,
        presentation_gpu: &G,
        support: &[PresentationBackendSupport<G>],
    ) -> Result<PresentationPath, NoCompatiblePresentationPath> {
        let pair_support = support
            .iter()
            .find(|entry| {
                &entry.rendering_gpu == rendering_gpu && &entry.presentation_gpu == presentation_gpu
            })
            .ok_or(NoCompatiblePresentationPath)?;

        if pair_support.direct_vulkan {
            return Ok(PresentationPath::DirectVulkan);
        }

        if pair_support.compatibility {
            return Ok(PresentationPath::Compatibility);
        }

        Err(NoCompatiblePresentationPath)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn support(
        rendering_gpu: &str,
        presentation_gpu: &str,
        direct_vulkan: bool,
        compatibility: bool,
    ) -> PresentationBackendSupport<String> {
        PresentationBackendSupport {
            rendering_gpu: rendering_gpu.to_owned(),
            presentation_gpu: presentation_gpu.to_owned(),
            direct_vulkan,
            compatibility,
        }
    }

    #[test]
    fn prefers_direct_vulkan_when_both_paths_are_available() {
        let entries = [support("gpu0", "gpu0", true, true)];

        assert_eq!(
            PresentationSelector::select(&"gpu0".to_owned(), &"gpu0".to_owned(), &entries),
            Ok(PresentationPath::DirectVulkan)
        );
    }

    #[test]
    fn selects_compatibility_when_direct_path_is_unavailable() {
        let entries = [support("gpu0", "gpu1", false, true)];

        assert_eq!(
            PresentationSelector::select(&"gpu0".to_owned(), &"gpu1".to_owned(), &entries),
            Ok(PresentationPath::Compatibility)
        );
    }

    #[test]
    fn ignores_capabilities_for_a_different_gpu_pair() {
        let entries = [support("gpu0", "gpu0", true, false)];

        assert_eq!(
            PresentationSelector::select(&"gpu0".to_owned(), &"gpu1".to_owned(), &entries),
            Err(NoCompatiblePresentationPath)
        );
    }

    #[test]
    fn rejects_a_pair_without_a_supported_backend() {
        let entries = [support("gpu0", "gpu1", false, false)];

        assert_eq!(
            PresentationSelector::select(&"gpu0".to_owned(), &"gpu1".to_owned(), &entries),
            Err(NoCompatiblePresentationPath)
        );
    }
}
