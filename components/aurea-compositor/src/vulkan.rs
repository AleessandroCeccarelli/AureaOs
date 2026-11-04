// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan instance, physical-device discovery and queue-family inspection.

use std::error::Error;
use std::fmt;

use smithay::{
    backend::vulkan::{version::Version, Instance, PhysicalDevice},
    reexports::ash::vk,
};

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
    pub name: String,
    pub api_version: String,
    pub driver: String,
    pub device_type: VulkanDeviceType,
    pub has_graphics_queue: bool,
}

/// Capabilities of a physical device queue family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanQueueFamilyInfo {
    pub index: u32,
    pub queue_count: u32,
    pub supports_graphics: bool,
    pub supports_compute: bool,
    pub supports_transfer: bool,
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
                has_graphics_queue: self.queue_families(device)
                    .iter()
                    .any(|family| family.supports_graphics),
            })
            .collect()
    }

    /// Returns queue-family capabilities for a physical device.
    pub fn queue_families(
        &self,
        device: &PhysicalDevice,
    ) -> Vec<VulkanQueueFamilyInfo> {
        let properties = device
            .instance()
            .handle()
            .get_physical_device_queue_family_properties(device.handle());

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

    /// Returns the number of discovered physical devices.
    pub fn device_count(&self) -> usize {
        self.physical_devices.len()
    }

    /// Returns whether at least one physical device was discovered.
    pub fn has_devices(&self) -> bool {
        !self.physical_devices.is_empty()
    }

    /// Selects a graphics-capable device according to the requested preference.
    ///
    /// Presentation compatibility is not checked here because it depends on
    /// the display surface, which is created by the window-system backend.
    pub fn select_device(
        &self,
        preference: RenderPreference,
    ) -> Option<&PhysicalDevice> {
        let preferred_type = match preference {
            RenderPreference::PowerSaving => Some(VulkanDeviceType::Integrated),
            RenderPreference::Performance => Some(VulkanDeviceType::Discrete),
            RenderPreference::Automatic => None,
        };

        self.physical_devices
            .iter()
            .filter(|device| {
                self.queue_families(device)
                    .iter()
                    .any(|family| family.supports_graphics && family.queue_count > 0)
            })
            .find(|device| {
                preferred_type
                    .map(|kind| Self::classify_device(device) == kind)
                    .unwrap_or(true)
            })
            .or_else(|| {
                self.physical_devices.iter().find(|device| {
                    self.queue_families(device)
                        .iter()
                        .any(|family| family.supports_graphics && family.queue_count > 0)
                })
            })
    }

    fn classify_device(device: &PhysicalDevice) -> VulkanDeviceType {
        match device.ty().as_raw() {
            1 => VulkanDeviceType::Integrated,
            2 => VulkanDeviceType::Discrete,
            3 => VulkanDeviceType::Virtual,
            4 => VulkanDeviceType::Cpu,
            _ => VulkanDeviceType::Other,
        }
    }
}
