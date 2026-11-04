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

/// General category of a Vulkan physical device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanDeviceType {
    Integrated,
    Discrete,
    Virtual,
    Cpu,
    Other,
}

/// Preference used when selecting a physical device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPreference {
    /// Prefer integrated graphics for lower power consumption.
    PowerSaving,

    /// Prefer discrete graphics for higher performance.
    Performance,
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
                device_type: match format!("{:?}", device.ty()).as_str() {
                    "INTEGRATED_GPU" => VulkanDeviceType::Integrated,
                    "DISCRETE_GPU" => VulkanDeviceType::Discrete,
                    "VIRTUAL_GPU" => VulkanDeviceType::Virtual,
                    "CPU" => VulkanDeviceType::Cpu,
                    _ => VulkanDeviceType::Other,
                },
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

    /// Selects a device according to the requested preference.
    ///
    /// This is an initial policy only. Display compatibility, queue
    /// capabilities and power-management state must also be considered
    /// before the compositor uses the selected device.
    pub fn select_device(&self, preference: GpuPreference) -> Option<&PhysicalDevice> {
        let preferred_type = match preference {
            GpuPreference::PowerSaving => "INTEGRATED_GPU",
            GpuPreference::Performance => "DISCRETE_GPU",
        };

        self.physical_devices
            .iter()
            .find(|device| format!("{:?}", device.ty()) == preferred_type)
            .or_else(|| self.physical_devices.first())
    }
}
