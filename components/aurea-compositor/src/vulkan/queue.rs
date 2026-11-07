// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Physical-device queue-family inspection.

use smithay::{backend::vulkan::PhysicalDevice, reexports::ash::vk};

use super::VulkanContext;

/// Capabilities of a physical-device queue family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanQueueFamilyInfo {
    pub index: u32,
    pub queue_count: u32,
    pub supports_graphics: bool,
    pub supports_compute: bool,
    pub supports_transfer: bool,
}

impl VulkanContext {
    /// Returns queue-family capabilities for a physical device.
    pub fn queue_families(&self, device: &PhysicalDevice) -> Vec<VulkanQueueFamilyInfo> {
        // SAFETY: Smithay's `PhysicalDevice` is enumerated from the same live
        // instance returned by `device.instance()`, so both handles are valid.
        let properties = unsafe {
            device
                .instance()
                .handle()
                .get_physical_device_queue_family_properties(device.handle())
        };

        properties
            .iter()
            .enumerate()
            .map(|(index, family)| VulkanQueueFamilyInfo {
                index: index as u32,
                queue_count: family.queue_count,
                supports_graphics: family.queue_flags.contains(vk::QueueFlags::GRAPHICS),
                supports_compute: family.queue_flags.contains(vk::QueueFlags::COMPUTE),
                supports_transfer: family.queue_flags.contains(vk::QueueFlags::TRANSFER),
            })
            .collect()
    }
}
