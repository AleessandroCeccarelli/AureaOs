// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Static Vulkan external-memory and DMA-BUF extension capabilities.

use smithay::{
    backend::{allocator::vulkan::VulkanAllocator, vulkan::PhysicalDevice},
};

/// Static DMA-BUF export capability reported by a Vulkan physical device.
///
/// Extension presence is a prerequisite for the export path, not proof that
/// a particular image format, tiling mode or DRM modifier is supported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VulkanExternalMemoryCapabilities {
    /// All extensions required by Smithay's Vulkan allocator are available.
    pub supports_dmabuf_export: bool,

    /// Required extension names missing from the physical device.
    pub missing_extensions: Vec<String>,
}

impl VulkanExternalMemoryCapabilities {
    /// Inspects the device extensions required by Smithay's DMA-BUF allocator.
    pub fn from_device(device: &PhysicalDevice) -> Self {
        let missing_extensions = VulkanAllocator::required_extensions(device)
            .iter()
            .filter(|extension| !device.has_device_extension(extension))
            .map(|extension| extension.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        Self {
            supports_dmabuf_export: missing_extensions.is_empty(),
            missing_extensions,
        }
    }
}
