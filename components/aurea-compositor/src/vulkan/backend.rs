
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan rendering-backend initialization.

use smithay::backend::vulkan::PhysicalDevice;

use crate::renderer::RenderPreference;

use super::{
    VulkanContext, VulkanDeviceInfo, VulkanInitError, VulkanLogicalDevice,
};

/// Owns the Vulkan context, selected GPU and logical device.
pub struct VulkanBackend {
    context: VulkanContext,
    selected_device_info: VulkanDeviceInfo,
    device: VulkanLogicalDevice,
}

impl VulkanBackend {
    /// Initializes Vulkan using the requested GPU preference.
    pub fn new(preference: RenderPreference) -> Result<Self, VulkanInitError> {
        let context = VulkanContext::new()?;

        let physical_device = context
            .select_device(preference)
            .ok_or_else(|| VulkanInitError::new("No graphics-capable Vulkan device found"))?;

        let queue_family_index = context
            .graphics_queue_family(physical_device)
            .ok_or_else(|| VulkanInitError::new("Selected device has no graphics queue"))?;

        let selected_device_info =
            Self::build_selected_device_info(&context, physical_device);

        let device =
            VulkanLogicalDevice::new(physical_device, queue_family_index)?;

        Ok(Self {
            context,
            selected_device_info,
            device,
        })
    }

    /// Returns the Vulkan context.
    pub fn context(&self) -> &VulkanContext {
        &self.context
    }

    /// Returns information about the selected physical GPU.
    pub fn selected_device_info(&self) -> &VulkanDeviceInfo {
        &self.selected_device_info
    }

    /// Returns the logical device.
    pub fn device(&self) -> &VulkanLogicalDevice {
        &self.device
    }

    fn build_selected_device_info(
        context: &VulkanContext,
        device: &PhysicalDevice,
    ) -> VulkanDeviceInfo {
        let has_graphics_queue = context
            .queue_families(device)
            .iter()
            .any(|family| family.supports_graphics && family.queue_count > 0);

        VulkanDeviceInfo::from_device(device, has_graphics_queue)
    }
}
