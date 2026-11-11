// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Physical-device information and selection.

use smithay::{
    backend::vulkan::PhysicalDevice,
    reexports::ash::vk,
};

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmNode;

use crate::renderer::RenderPreference;

use super::{VulkanContext, VulkanExternalMemoryCapabilities, VulkanRenderingFeaturePlan};

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

    /// Whether the device exposes all extensions required by Smithay's
    /// Vulkan DMA-BUF allocator.
    ///
    /// This does not guarantee support for every DRM format or modifier.
    pub has_dmabuf_export_extensions: bool,
}

impl VulkanDeviceInfo {
    pub(super) fn from_device(device: &PhysicalDevice, has_graphics_queue: bool) -> Self {
        let external_memory = VulkanExternalMemoryCapabilities::from_device(device);
        let has_dmabuf_export_extensions = external_memory.supports_dmabuf_export;

        Self {
            name: device.name().to_owned(),
            api_version: format!("{:?}", device.api_version()),
            driver: format!("{:?}", device.driver()),
            device_type: VulkanContext::classify_device(device),
            has_graphics_queue,
            has_dmabuf_export_extensions,
        }
    }
}

impl VulkanContext {
    /// Returns summaries of all discovered physical devices.
    pub fn device_info(&self) -> Vec<VulkanDeviceInfo> {
        self.physical_devices
            .iter()
            .map(|device| {
                let has_graphics_queue = self
                    .queue_families(device)
                    .iter()
                    .any(|family| family.supports_graphics && family.queue_count > 0);

                VulkanDeviceInfo::from_device(device, has_graphics_queue)
            })
            .collect()
    }

    /// Selects a graphics-capable physical device by preference.
    ///
    /// This selection does not guarantee compatibility with a particular
    /// DRM output. Use `select_device_for_drm_node` when an output's DRM
    /// device identity is known.
    pub fn select_device(&self, preference: RenderPreference) -> Option<&PhysicalDevice> {
        let preferred_type = match preference {
            RenderPreference::PowerSaving => Some(VulkanDeviceType::Integrated),
            RenderPreference::Performance => Some(VulkanDeviceType::Discrete),
            RenderPreference::Automatic => None,
        };

        self.physical_devices
            .iter()
            .filter(|device| self.graphics_queue_family(device).is_some() && VulkanRenderingFeaturePlan::from_device(device).is_ok())
            .find(|device| {
                preferred_type
                    .map(|kind| Self::classify_device(device) == kind)
                    .unwrap_or(true)
            })
            .or_else(|| {
                self.physical_devices
                    .iter()
                    .find(|device| {
                        self.graphics_queue_family(device).is_some()
                            && VulkanRenderingFeaturePlan::from_device(device).is_ok()
                    })
            })
    }

    /// Selects a graphics-capable Vulkan device matching a DRM node.
    ///
    /// Both primary and render-node identities are considered. Devices whose
    /// DRM identity cannot be queried are excluded. This is useful when the
    /// caller has a render node rather than the primary node.
    ///
    /// The preference is used when multiple matching Vulkan devices exist.
    #[cfg(feature = "drm")]
    pub fn select_device_for_drm_node(
        &self,
        node: DrmNode,
        preference: RenderPreference,
    ) -> Option<&PhysicalDevice> {
        let matching_devices = self
            .physical_devices
            .iter()
            .filter(|device| {
                let primary_matches = device.primary_node().ok().flatten() == Some(node);
                let render_matches = device.render_node().ok().flatten() == Some(node);

                (primary_matches || render_matches)
                    && self.graphics_queue_family(device).is_some()
                    && VulkanRenderingFeaturePlan::from_device(device).is_ok()
            })
            .collect::<Vec<_>>();

        let preferred_type = match preference {
            RenderPreference::PowerSaving => Some(VulkanDeviceType::Integrated),
            RenderPreference::Performance => Some(VulkanDeviceType::Discrete),
            RenderPreference::Automatic => None,
        };

        matching_devices
            .iter()
            .copied()
            .find(|device| {
                preferred_type
                    .map(|kind| Self::classify_device(device) == kind)
                    .unwrap_or(true)
            })
            .or_else(|| matching_devices.into_iter().next())
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

    pub(super) fn graphics_queue_family(&self, device: &PhysicalDevice) -> Option<u32> {
        self.queue_families(device)
            .into_iter()
            .find(|family| family.supports_graphics && family.queue_count > 0)
            .map(|family| family.index)
    }
}
