// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan instance and physical-device discovery.

use std::error::Error;
use std::fmt;

use smithay::backend::vulkan::{version::Version, Instance, PhysicalDevice};

/// Error returned while initializing Vulkan.
#[derive(Debug)]
pub struct VulkanInitError {
    message: String,
}

impl VulkanInitError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for VulkanInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for VulkanInitError {}

/// Holds the Vulkan instance and the physical devices discovered on the system.
pub struct VulkanContext {
    instance: Instance,
    physical_devices: Vec<PhysicalDevice>,
}

impl VulkanContext {
    /// Creates a Vulkan instance and enumerates available physical devices.
    pub fn new() -> Result<Self, VulkanInitError> {
        let instance = Instance::new(Version::VERSION_1_3, None).map_err(|error| {
            VulkanInitError::new(format!("Failed to create Vulkan instance: {error}"))
        })?;

        let physical_devices =
            PhysicalDevice::enumerate(&instance)
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
