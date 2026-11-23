// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan instance and physical-device context.

use smithay::backend::vulkan::{Instance, PhysicalDevice, version::Version};

use super::VulkanInitError;

/// Owns the Vulkan instance and its discovered physical devices.
pub struct VulkanContext {
    pub(super) instance: Instance,
    pub(super) physical_devices: Vec<PhysicalDevice>,
}

impl VulkanContext {
    /// Creates a Vulkan instance and enumerates physical devices.
    pub fn new() -> Result<Self, VulkanInitError> {
        let instance = Instance::new(Version::VERSION_1_3, None).map_err(|error| {
            VulkanInitError::new(format!("Failed to create Vulkan instance: {error}"))
        })?;

        let physical_devices = PhysicalDevice::enumerate(&instance)
            .map_err(|error| {
                VulkanInitError::new(format!(
                    "Failed to enumerate Vulkan physical devices: {error}"
                ))
            })?
            .collect();

        Ok(Self {
            instance,
            physical_devices,
        })
    }

    /// Returns the Vulkan instance.
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Returns the discovered physical devices.
    pub fn physical_devices(&self) -> &[PhysicalDevice] {
        &self.physical_devices
    }

    /// Returns the number of discovered physical devices.
    pub fn device_count(&self) -> usize {
        self.physical_devices.len()
    }

    /// Returns whether at least one physical device was discovered.
    pub fn has_devices(&self) -> bool {
        !self.physical_devices.is_empty()
    }
}
