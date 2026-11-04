// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan logical-device creation and graphics-queue access.

use smithay::{
    backend::vulkan::PhysicalDevice,
    reexports::ash::{self, vk},
};

use crate::renderer::RenderPreference;

use super::{VulkanContext, VulkanInitError};

/// A logical Vulkan device and its graphics queue.
pub struct VulkanLogicalDevice {
    device: ash::Device,
    graphics_queue: vk::Queue,
    graphics_queue_family: u32,
}

impl VulkanLogicalDevice {
    /// Creates a logical device with one graphics queue.
    pub(super) fn new(
        physical_device: &PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self, VulkanInitError> {
        let priorities = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];

        let create_info =
            vk::DeviceCreateInfo::default().queue_create_infos(&queue_info);

        let device = unsafe {
            physical_device
                .instance()
                .handle()
                .create_device(physical_device.handle(), &create_info, None)
        }
        .map_err(|error| {
            VulkanInitError::new(format!("Failed to create Vulkan logical device: {error}"))
        })?;

        let graphics_queue =
            unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            device,
            graphics_queue,
            graphics_queue_family: queue_family_index,
        })
    }

    /// Returns the underlying Vulkan device handle.
    pub fn handle(&self) -> &ash::Device {
        &self.device
    }

    /// Returns the graphics queue handle.
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    /// Returns the graphics queue-family index.
    pub fn graphics_queue_family(&self) -> u32 {
        self.graphics_queue_family
    }
}

impl Drop for VulkanLogicalDevice {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
        }
    }
}

impl VulkanContext {
    /// Creates a logical device for the selected physical device.
    pub fn create_logical_device(
        &self,
        preference: RenderPreference,
    ) -> Result<VulkanLogicalDevice, VulkanInitError> {
        let physical_device = self
            .select_device(preference)
            .ok_or_else(|| VulkanInitError::new("No graphics-capable Vulkan device found"))?;

        let queue_family_index = self
            .graphics_queue_family(physical_device)
            .ok_or_else(|| VulkanInitError::new("Selected device has no graphics queue"))?;

        VulkanLogicalDevice::new(physical_device, queue_family_index)
    }
}
