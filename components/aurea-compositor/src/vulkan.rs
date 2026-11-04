
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan instance, physical-device discovery and logical-device setup.

use std::error::Error;
use std::fmt;

use smithay::{
    backend::vulkan::{version::Version, Instance, PhysicalDevice},
    reexports::ash::{self, vk},
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

/// Capabilities of a physical-device queue family.
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

/// A logical Vulkan device and its graphics queue.
pub struct VulkanLogicalDevice {
    device: ash::Device,
    graphics_queue: vk::Queue,
    graphics_queue_family: u32,
}

impl VulkanLogicalDevice {
    /// Creates a logical device with one graphics queue.
    fn new(
        physical_device: &PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self, VulkanInitError> {
        let priorities = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info);

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
                has_graphics_queue: self
                    .queue_families(device)
                    .iter()
                    .any(|family| family.supports_graphics && family.queue_count > 0),
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

    /// Selects a graphics-capable physical device according to the preference.
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
            .filter(|device| self.graphics_queue_family(device).is_some())
            .find(|device| {
                preferred_type
                    .map(|kind| Self::classify_device(device) == kind)
                    .unwrap_or(true)
            })
            .or_else(|| {
                self.physical_devices
                    .iter()
                    .find(|device| self.graphics_queue_family(device).is_some())
            })
    }

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

    fn graphics_queue_family(&self, device: &PhysicalDevice) -> Option<u32> {
        self.queue_families(device)
            .into_iter()
            .find(|family| family.supports_graphics && family.queue_count > 0)
            .map(|family| family.index)
    }

    /// Returns the number of discovered physical devices.
    pub fn device_count(&self) -> usize {
        self.physical_devices.len()
    }

    /// Returns whether at least one physical device was discovered.
    pub fn has_devices(&self) -> bool {
        !self.physical_devices.is_empty()
    }

    fn classify_device(device: &PhysicalDevice) -> VulkanDeviceType {
        match device.ty() {
            vk::PhysicalDeviceType::INTEGRATED_GPU => VulkanDeviceType::Integrated,
            vk::PhysicalDeviceType::DISCRETE_GPU => VulkanDeviceType::Discrete,
            vk::PhysicalDeviceType::VIRTUAL_GPU => VulkanDeviceType::Virtual,
            vk::PhysicalDeviceType::CPU => VulkanDeviceType::Cpu,
            _ => VulkanDeviceType::Other,
        }
    }
}
