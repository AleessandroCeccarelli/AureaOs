// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Static Vulkan external-memory and DMA-BUF extension capabilities.

use smithay::backend::vulkan::PhysicalDevice;

use super::VulkanDeviceExtensionPlan;

/// Static DMA-BUF interoperability capabilities of one physical device.
///
/// Extension presence is a prerequisite, not proof that a particular image
/// format, modifier, allocation or import operation will succeed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VulkanExternalMemoryCapabilities {
    /// All extensions required for DMA-BUF import/export are available.
    pub supports_dmabuf_export: bool,
    /// Required extension names missing from the physical device.
    pub missing_extensions: Vec<String>,
}

impl VulkanExternalMemoryCapabilities {
    /// Inspects AureaOS's DMA-BUF extension requirements.
    pub fn from_device(device: &PhysicalDevice) -> Self {
        let plan = VulkanDeviceExtensionPlan::from_device(device);
        Self {
            supports_dmabuf_export: plan.supports_dmabuf_interop(),
            missing_extensions: plan.missing_extensions().to_vec(),
        }
    }
}
