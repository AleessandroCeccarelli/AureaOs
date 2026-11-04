// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan instance and physical-device discovery.

use std::error::Error;
use std::fmt;

use smithay::backend::vulkan::{version::Version, Instance, PhysicalDevice};

use crate::renderer::RenderPreference;

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

/// General category of a Vulkan physical device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanDeviceType {
    Integrated,
    Discrete,
    Virtual,
    Cpu,
    Other,
}

/// Summary of a Vulkan physical device.
#[derive(Debug, Clone)]
pub struct VulkanDeviceInfo {
    /// Device name reported by the Vulkan implementation.
    pub name: String,

    /// Vulkan API version supported by the device.
    pub api_version: String,

    /// Driver information reported by Smithay.
    pub driver: String,

    /// General device category.
    pub device_type: VulkanDeviceType,
}

/// Holds the Vulkan instance and discovered physical devices.
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

    /// Returns summaries of all discovered physical devices.
    pub fn device_info(&self) -> Vec<VulkanDeviceInfo> {
        self.physical_devices
            .iter()
            .map(|device| VulkanDeviceInfo {
                name: device.name().to_owned(),
                api_version: format!("{:?}", device.api_version()),
                driver: format!("{:?}", device.driver()),
                device_type: Self::classify_device(device),
            })
            .collect()
    }

    /// Returns the number of discovered physical devices.
    pub fn device_count(&self) -> usize {
        self.physical_devices.len()
    }

    /// Returns whether at least one physical device was discovered.
    pub fn has_devices(&self) -> bool {
        !self.physical_devices.is_empty()
    }

    /// Selects a device according to the requested rendering preference.
    ///
    /// This is an initial preference-based policy. Display compatibility,
    /// queue capabilities and power-management state must also be checked
    /// before using the selected device.
    pub fn select_device(
        &self,
        preference: RenderPreference,
    ) -> Option<&PhysicalDevice> {
        let preferred_type = match preference {
            RenderPreference::PowerSaving => VulkanDeviceType::Integrated,
            RenderPreference::Performance => VulkanDeviceType::Discrete,
            RenderPreference::Automatic => return self.physical_devices.first(),
        };

        self.physical_devices
            .iter()
            .find(|device| Self::classify_device(device) == preferred_type)
            .or_else(|| self.physical_devices.first())
    }

    fn classify_device(device: &PhysicalDevice) -> VulkanDeviceType {
        // Vulkan VkPhysicalDeviceType values:
        // 0 = OTHER, 1 = INTEGRATED_GPU, 2 = DISCRETE_GPU,
        // 3 = VIRTUAL_GPU, 4 = CPU.
        match device.ty().as_raw() {
            1 => VulkanDeviceType::Integrated,
            2 => VulkanDeviceType::Discrete,
            3 => VulkanDeviceType::Virtual,
            4 => VulkanDeviceType::Cpu,
            _ => VulkanDeviceType::Other,
        }
    }
}
