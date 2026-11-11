// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Conservative compatibility checks between Vulkan DMA-BUF export and DRM scan-out.

use smithay::backend::allocator::{Fourcc, Modifier};
use smithay::reexports::ash::vk;

use crate::drm::DrmPrimaryPlaneFormatCapabilities;

use super::{VulkanExternalMemoryCapabilities, VulkanFormatCapabilities};

/// Result of checking a Vulkan/DRM format pair for direct scan-out eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrmVulkanFormat {
    /// DRM pixel format.
    pub code: Fourcc,
    /// Exact DRM modifier shared by Vulkan and the primary plane.
    pub modifier: Modifier,
    /// Number of memory planes reported by Vulkan for this modifier.
    pub memory_plane_count: u32,
}

/// Why a device/plane combination cannot use the direct DMA-BUF path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrmVulkanCompatibilityStatus {
    /// At least one exact format/modifier pair is supported by both sides.
    Compatible,
    /// Vulkan does not expose all extensions required for DMA-BUF export.
    DmaBufExportUnavailable,
    /// No exact format/modifier pair with color-attachment support was found.
    NoCommonRenderableFormat,
}

/// Conservative capability intersection for one DRM primary plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrmVulkanCompatibility {
    status: DrmVulkanCompatibilityStatus,
    formats: Vec<DrmVulkanFormat>,
}

impl DrmVulkanCompatibility {
    /// Evaluates the Vulkan export path against one DRM primary plane.
    ///
    /// A pair is accepted only when:
    /// - Vulkan exposes the required DMA-BUF export extensions;
    /// - the FourCC and modifier match exactly on both devices;
    /// - Vulkan reports at least one memory plane;
    /// - Vulkan supports color-attachment image usage for that modifier.
    ///
    /// The caller must pass Vulkan capabilities from the physical device
    /// associated with the DRM device that owns this plane. Plane identifiers
    /// are local to that DRM device and are not globally unique.
    ///
    /// This is a capability intersection only. It does not guarantee that a
    /// particular image allocation, DMA-BUF import, framebuffer creation or
    /// atomic KMS commit will succeed.
    pub fn evaluate(
        plane: &DrmPrimaryPlaneFormatCapabilities,
        vulkan_formats: &[VulkanFormatCapabilities],
        external_memory: &VulkanExternalMemoryCapabilities,
    ) -> Self {
        if !external_memory.supports_dmabuf_export {
            return Self {
                status: DrmVulkanCompatibilityStatus::DmaBufExportUnavailable,
                formats: Vec::new(),
            };
        }

        let mut formats = Vec::new();

        for drm_format in &plane.formats {
            let modifier = vulkan_formats
                .iter()
                .filter(|capability| capability.format == drm_format.code)
                .flat_map(|capability| capability.modifiers.iter())
                .find(|modifier| {
                    modifier.modifier == drm_format.modifier
                        && modifier.plane_count > 0
                        && modifier
                            .tiling_features
                            .contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT)
                });

            let Some(modifier) = modifier else {
                continue;
            };

            let candidate = DrmVulkanFormat {
                code: drm_format.code,
                modifier: drm_format.modifier,
                memory_plane_count: modifier.plane_count,
            };

            if !formats.contains(&candidate) {
                formats.push(candidate);
            }
        }

        let status = if formats.is_empty() {
            DrmVulkanCompatibilityStatus::NoCommonRenderableFormat
        } else {
            DrmVulkanCompatibilityStatus::Compatible
        };

        Self { status, formats }
    }

    /// Returns the compatibility status.
    pub fn status(&self) -> DrmVulkanCompatibilityStatus {
        self.status
    }

    /// Returns the exact format/modifier pairs accepted by both sides.
    pub fn formats(&self) -> &[DrmVulkanFormat] {
        &self.formats
    }

    /// Returns whether at least one compatible renderable pair exists.
    pub fn is_compatible(&self) -> bool {
        self.status == DrmVulkanCompatibilityStatus::Compatible
    }
}
