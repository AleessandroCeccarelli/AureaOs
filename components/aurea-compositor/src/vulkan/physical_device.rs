// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Physical-device information and selection.

use smithay::{backend::vulkan::PhysicalDevice, reexports::ash::vk};

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmNode;

use crate::{
    adaptive_graphics::{AureaAdaptiveGraphics, GraphicsCandidate, GraphicsCapabilities},
    renderer::RenderPreference,
};

use super::{
    VulkanContext, VulkanExternalMemoryCapabilities, VulkanRendererMode,
    VulkanRenderingCapabilities, VulkanRenderingFeaturePlan,
};

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

    /// Why the device cannot satisfy AureaOS's minimum rendering requirements.
    /// This includes a missing graphics queue or required Vulkan 1.3 features.
    pub rendering_error: Option<String>,

    /// Whether the device exposes all extensions required by Smithay's
    /// Vulkan DMA-BUF allocator.
    ///
    /// This does not guarantee support for every DRM format or modifier.
    pub has_dmabuf_export_extensions: bool,
}

impl VulkanDeviceInfo {
    /// Returns true when this GPU meets AureaOS's minimum Vulkan rendering requirements.
    pub fn supports_minimum_rendering(&self) -> bool {
        self.has_graphics_queue && self.rendering_error.is_none()
    }

    pub(super) fn from_device(device: &PhysicalDevice, has_graphics_queue: bool) -> Self {
        let external_memory = VulkanExternalMemoryCapabilities::from_device(device);
        let has_dmabuf_export_extensions = external_memory.supports_dmabuf_export;
        let rendering_error =
            VulkanContext::rendering_compatibility_error(device, has_graphics_queue);

        Self {
            name: device.name().to_owned(),
            api_version: format!("{:?}", device.api_version()),
            driver: format!("{:?}", device.driver()),
            device_type: VulkanContext::classify_device(device),
            has_graphics_queue,
            rendering_error,
            has_dmabuf_export_extensions,
        }
    }
}

impl VulkanContext {
    /// Builds AAG candidates from the capabilities queried for each physical device.
    /// The current renderer implements only the Vulkan 1.3 path; no legacy
    /// Vulkan 1.0–1.2 backend is advertised here.
    fn adaptive_graphics_candidates(&self) -> Vec<GraphicsCandidate> {
        self.physical_devices
            .iter()
            .enumerate()
            .map(|(device_index, device)| {
                let has_graphics_queue = self.graphics_queue_family(device).is_some();
                let properties = unsafe {
                    device
                        .instance()
                        .handle()
                        .get_physical_device_properties(device.handle())
                };
                let api_major = vk::api_version_major(properties.api_version);
                let api_minor = vk::api_version_minor(properties.api_version);
                let rendering_capabilities = VulkanRenderingCapabilities::inspect(device);
                GraphicsCandidate {
                    device_index,
                    name: device.name().to_owned(),
                    capabilities: GraphicsCapabilities {
                        api_major,
                        api_minor,
                        dynamic_rendering: rendering_capabilities.supports_dynamic_rendering(),
                        synchronization2: rendering_capabilities.supports_synchronization2(),
                        compatibility_renderer_available: false,
                        software_device: Self::classify_device(device) == VulkanDeviceType::Cpu,
                        has_graphics_queue,
                    },
                }
            })
            .collect()
    }

    /// Returns the AAG plan based on the live Vulkan physical-device inventory.
    pub fn adaptive_graphics_plan(&self) -> Option<crate::adaptive_graphics::AdaptiveGraphicsPlan> {
        AureaAdaptiveGraphics::select(&self.adaptive_graphics_candidates())
    }

    /// Returns the AAG plan for one already-discovered physical-device index.
    pub(super) fn adaptive_graphics_plan_for_device(
        &self,
        device_index: usize,
    ) -> Option<crate::adaptive_graphics::AdaptiveGraphicsPlan> {
        let candidate = self
            .adaptive_graphics_candidates()
            .into_iter()
            .find(|candidate| candidate.device_index == device_index)?;
        AureaAdaptiveGraphics::select(&[candidate])
    }

    fn rendering_compatibility_error(
        device: &PhysicalDevice,
        has_graphics_queue: bool,
    ) -> Option<String> {
        if !has_graphics_queue {
            return Some("No graphics-capable queue family is available".to_owned());
        }

        match VulkanRenderingFeaturePlan::from_device(device) {
            Err(error) => Some(error),
            Ok(plan) if plan.mode() == VulkanRendererMode::CompatibilityPending => Some(format!(
                "GPU '{}' exposes Vulkan API version {} but the complete compatibility frame renderer is not yet integrated",
                device.name(),
                device.api_version()
            )),
            Ok(_) => None,
        }
    }

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

        let mut candidates = self.adaptive_graphics_candidates();
        candidates.sort_by_key(|candidate| {
            let kind = Self::classify_device(&self.physical_devices[candidate.device_index]);
            match preferred_type {
                Some(preferred) if kind == preferred => 0,
                _ if candidate.capabilities.software_device => 2,
                _ => 1,
            }
        });
        AureaAdaptiveGraphics::select(&candidates)
            .and_then(|plan| self.physical_devices.get(plan.device_index))
    }

    /// Selects a Vulkan CPU device for host-readback compatibility rendering.
    ///
    /// This path is intentionally separate from DRM-device matching: software
    /// Vulkan renders into host-readable images, while GBM/KMS remains owned
    /// by the physical display GPU.
    pub fn select_software_device(&self) -> Option<&PhysicalDevice> {
        let candidates = self
            .adaptive_graphics_candidates()
            .into_iter()
            .filter(|candidate| candidate.capabilities.software_device)
            .collect::<Vec<_>>();
        AureaAdaptiveGraphics::select(&candidates)
            .filter(|plan| plan.path == crate::adaptive_graphics::GraphicsPath::SoftwareVulkan)
            .and_then(|plan| self.physical_devices.get(plan.device_index))
    }

    /// Selects a graphics-capable Vulkan device matching a DRM node.
    ///
    /// Both primary and render-node identities are considered. Selection only
    /// requires a usable graphics queue and Aurea's rendering feature plan.
    /// DMA-BUF export and foreign queue ownership are checked separately by
    /// the direct-presentation capability probe.
    #[cfg(feature = "drm")]
    pub fn select_device_for_drm_node(
        &self,
        node: DrmNode,
        preference: RenderPreference,
    ) -> Option<&PhysicalDevice> {
        let matching_indices = self
            .physical_devices
            .iter()
            .enumerate()
            .filter_map(|(index, device)| {
                let primary_matches = device.primary_node().ok().flatten() == Some(node);
                let render_matches = device.render_node().ok().flatten() == Some(node);
                (primary_matches || render_matches).then_some(index)
            })
            .collect::<std::collections::HashSet<_>>();

        let preferred_type = match preference {
            RenderPreference::PowerSaving => Some(VulkanDeviceType::Integrated),
            RenderPreference::Performance => Some(VulkanDeviceType::Discrete),
            RenderPreference::Automatic => None,
        };
        let mut candidates = self
            .adaptive_graphics_candidates()
            .into_iter()
            .filter(|candidate| matching_indices.contains(&candidate.device_index))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| {
            let kind = Self::classify_device(&self.physical_devices[candidate.device_index]);
            match preferred_type {
                Some(preferred) if kind == preferred => 0,
                _ if candidate.capabilities.software_device => 2,
                _ => 1,
            }
        });
        AureaAdaptiveGraphics::select(&candidates)
            .and_then(|plan| self.physical_devices.get(plan.device_index))
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
