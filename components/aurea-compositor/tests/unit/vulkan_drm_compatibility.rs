// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Hardware-independent tests for Vulkan/DRM capability intersection.

use aurea_compositor::{
    drm::DrmPrimaryPlaneFormatCapabilities,
    vulkan::{
        DrmVulkanCompatibility, DrmVulkanCompatibilityStatus,
        VulkanExternalMemoryCapabilities, VulkanFormatCapabilities,
        VulkanFormatModifierInfo,
    },
};
use smithay::{
    backend::allocator::{Format, Fourcc, Modifier},
    reexports::ash::vk,
};

fn drm_plane(formats: Vec<Format>) -> DrmPrimaryPlaneFormatCapabilities {
    DrmPrimaryPlaneFormatCapabilities {
        crtc_id: 7,
        plane_id: 11,
        formats,
    }
}

fn vulkan_format(
    code: Fourcc,
    modifier: Modifier,
    plane_count: u32,
    features: vk::FormatFeatureFlags,
) -> VulkanFormatCapabilities {
    VulkanFormatCapabilities {
        format: code,
        modifiers: vec![VulkanFormatModifierInfo {
            modifier,
            plane_count,
            tiling_features: features,
        }],
    }
}

fn export_capabilities(supported: bool) -> VulkanExternalMemoryCapabilities {
    VulkanExternalMemoryCapabilities {
        supports_dmabuf_export: supported,
        missing_extensions: if supported {
            Vec::new()
        } else {
            vec!["VK_KHR_external_memory_fd".to_owned()]
        },
    }
}

#[test]
fn accepts_only_an_exact_fourcc_modifier_match() {
    let code = Fourcc::Argb8888;
    let modifier = Modifier::Linear;
    let plane = drm_plane(vec![Format { code, modifier }]);
    let vulkan = [vulkan_format(
        code,
        modifier,
        1,
        vk::FormatFeatureFlags::COLOR_ATTACHMENT,
    )];

    let result = DrmVulkanCompatibility::evaluate(&plane, &vulkan, &export_capabilities(true));

    assert_eq!(result.status(), DrmVulkanCompatibilityStatus::Compatible);
    assert!(result.is_compatible());
    assert_eq!(result.formats().len(), 1);
    assert_eq!(result.formats()[0].code, code);
    assert_eq!(result.formats()[0].modifier, modifier);
    assert_eq!(result.formats()[0].memory_plane_count, 1);
}

#[test]
fn rejects_a_modifier_supported_only_by_vulkan() {
    let code = Fourcc::Argb8888;
    let plane = drm_plane(vec![Format {
        code,
        modifier: Modifier::Linear,
    }]);
    let vulkan = [vulkan_format(
        code,
        Modifier::Invalid,
        1,
        vk::FormatFeatureFlags::COLOR_ATTACHMENT,
    )];

    let result = DrmVulkanCompatibility::evaluate(&plane, &vulkan, &export_capabilities(true));

    assert_eq!(
        result.status(),
        DrmVulkanCompatibilityStatus::NoCommonRenderableFormat
    );
    assert!(result.formats().is_empty());
}

#[test]
fn rejects_missing_dma_buf_export_extensions_before_matching_formats() {
    let code = Fourcc::Argb8888;
    let plane = drm_plane(vec![Format {
        code,
        modifier: Modifier::Linear,
    }]);
    let vulkan = [vulkan_format(
        code,
        Modifier::Linear,
        1,
        vk::FormatFeatureFlags::COLOR_ATTACHMENT,
    )];

    let result = DrmVulkanCompatibility::evaluate(&plane, &vulkan, &export_capabilities(false));

    assert_eq!(
        result.status(),
        DrmVulkanCompatibilityStatus::DmaBufExportUnavailable
    );
    assert!(result.formats().is_empty());
}

#[test]
fn rejects_formats_without_color_attachment_support() {
    let code = Fourcc::Argb8888;
    let plane = drm_plane(vec![Format {
        code,
        modifier: Modifier::Linear,
    }]);
    let vulkan = [vulkan_format(
        code,
        Modifier::Linear,
        1,
        vk::FormatFeatureFlags::SAMPLED_IMAGE,
    )];

    let result = DrmVulkanCompatibility::evaluate(&plane, &vulkan, &export_capabilities(true));

    assert_eq!(
        result.status(),
        DrmVulkanCompatibilityStatus::NoCommonRenderableFormat
    );
}

#[test]
fn rejects_zero_memory_plane_count() {
    let code = Fourcc::Argb8888;
    let plane = drm_plane(vec![Format {
        code,
        modifier: Modifier::Linear,
    }]);
    let vulkan = [vulkan_format(
        code,
        Modifier::Linear,
        0,
        vk::FormatFeatureFlags::COLOR_ATTACHMENT,
    )];

    let result = DrmVulkanCompatibility::evaluate(&plane, &vulkan, &export_capabilities(true));

    assert_eq!(
        result.status(),
        DrmVulkanCompatibilityStatus::NoCommonRenderableFormat
    );
}

#[test]
fn ignores_vulkan_formats_not_advertised_by_the_drm_plane() {
    let plane = drm_plane(vec![Format {
        code: Fourcc::Argb8888,
        modifier: Modifier::Linear,
    }]);
    let vulkan = [vulkan_format(
        Fourcc::Xrgb8888,
        Modifier::Linear,
        1,
        vk::FormatFeatureFlags::COLOR_ATTACHMENT,
    )];

    let result = DrmVulkanCompatibility::evaluate(&plane, &vulkan, &export_capabilities(true));

    assert_eq!(
        result.status(),
        DrmVulkanCompatibilityStatus::NoCommonRenderableFormat
    );
}
