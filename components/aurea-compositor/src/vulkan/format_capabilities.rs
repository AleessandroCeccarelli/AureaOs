// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM format and modifier capabilities reported by Vulkan.

use smithay::{
    backend::{
        allocator::{Fourcc, Modifier, vulkan::format::get_vk_format},
        vulkan::PhysicalDevice,
    },
    reexports::ash::vk,
};

/// Vulkan support information for one DRM format modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanFormatModifierInfo {
    /// DRM format modifier.
    pub modifier: Modifier,

    /// Number of memory planes required by this format and modifier.
    pub plane_count: u32,

    /// Vulkan features supported for images using this modifier.
    pub tiling_features: vk::FormatFeatureFlags,
}

/// DRM format capabilities advertised by one Vulkan physical device.
///
/// This is a device-level capability inventory. It does not prove that a
/// format/modifier pair supports a particular image usage or buffer size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VulkanFormatCapabilities {
    /// DRM FourCC format queried.
    pub format: Fourcc,

    /// Modifiers advertised by the Vulkan physical device for this format.
    pub modifiers: Vec<VulkanFormatModifierInfo>,
}

impl VulkanFormatCapabilities {
    /// Queries the DRM modifiers advertised for one format by a Vulkan device.
    ///
    /// Returns an empty modifier list when the format has no Vulkan mapping
    /// or the device advertises no modifiers for it.
    ///
    /// An error is returned when the device does not expose the
    /// VK_EXT_image_drm_format_modifier capability.
    pub fn query(device: &PhysicalDevice, format: Fourcc) -> Result<Self, String> {
        let Some(vulkan_format) = get_vk_format(format) else {
            return Ok(Self {
                format,
                modifiers: Vec::new(),
            });
        };

        let mut modifiers = Vec::new();

        // Linear tiling is part of core Vulkan format properties and does not
        // depend on VK_EXT_image_drm_format_modifier.
        let format_properties = unsafe {
            device
                .instance()
                .handle()
                .get_physical_device_format_properties(device.handle(), vulkan_format)
        };
        if format_properties
            .linear_tiling_features
            .contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT)
        {
            modifiers.push(VulkanFormatModifierInfo {
                modifier: Modifier::Linear,
                plane_count: 1,
                tiling_features: format_properties.linear_tiling_features,
            });
        }

        if device.has_device_extension(c"VK_EXT_image_drm_format_modifier") {
            let properties = device
                .get_format_modifier_properties(vulkan_format)
                .map_err(|error| error.to_string())?;

            for property in properties {
                let candidate = VulkanFormatModifierInfo {
                    modifier: Modifier::from(property.drm_format_modifier),
                    plane_count: property.drm_format_modifier_plane_count,
                    tiling_features: property.drm_format_modifier_tiling_features,
                };
                if !modifiers
                    .iter()
                    .any(|existing| existing.modifier == candidate.modifier)
                {
                    modifiers.push(candidate);
                }
            }
        }

        Ok(Self { format, modifiers })
    }
}
