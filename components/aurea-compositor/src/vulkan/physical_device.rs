// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Physical-device information and selection.

use smithay::{
    backend::vulkan::PhysicalDevice,
    reexports::ash::vk,
};

use crate::renderer::RenderPreference;

use super::{queue::VulkanQueueFamilyInfo, VulkanContext};

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

impl VulkanContext {
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

    /// Selects a graphics-capable physical device by preference.
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

    pub(super) fn classify_device(device: &PhysicalDevice) -> VulkanDeviceType {
        match device.ty() {
            vk::PhysicalDeviceType::INTEGRATED_GPU => VulkanDeviceType::Integrated,
            vk::PhysicalDeviceType::DISCRETE_GPU => VulkanDeviceType::Discrete,
            vk::PhysicalDeviceType::VIRTUAL_GPU => VulkanDeviceType::Virtual,
            vk::PhysicalDeviceType::CPU => VulkanDeviceType::Cpu,
            _ => VulkanDeviceType::Other,
        }
    }

    pub(super) fn graphics_queue_family(
        &self,
        device: &PhysicalDevice,
    ) -> Option<u32> {
        self.queue_families(device)
            .into_iter()
            .find(|family| family.supports_graphics && family.queue_count > 0)
            .map(|family| family.index)
    }
}
