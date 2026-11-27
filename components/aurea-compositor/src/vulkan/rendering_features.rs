// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan renderer capability inspection and modern feature negotiation.

use smithay::{backend::vulkan::PhysicalDevice, reexports::ash::vk};

/// Renderer implementation selected from verified physical-device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanRendererMode {
    /// Vulkan 1.3 dynamic rendering and synchronization2.
    ModernDynamic,
    /// The device can use the Vulkan 1.0 core feature model, but the complete
    /// compatibility renderer is not yet enabled by the compositor.
    CompatibilityPending,
}

/// Physical-device rendering capabilities, kept separate from presentation support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanRenderingCapabilities {
    api_version: u32,
    dynamic_rendering: bool,
    synchronization2: bool,
    mode: VulkanRendererMode,
}

impl VulkanRenderingCapabilities {
    /// Queries Vulkan 1.0 core features on every device and Vulkan 1.3 feature
    /// structures only when the physical device advertises Vulkan 1.3 or newer.
    ///
    /// This is a capability inventory only: CompatibilityPending must not be
    /// interpreted as proof that the legacy pipeline and DRM presentation path
    /// are implemented.
    pub fn inspect(device: &PhysicalDevice) -> Self {
        let instance = device.instance().handle();
        let physical_device = device.handle();
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let api_version = properties.api_version;

        // This core query is valid for Vulkan 1.0 devices and does not chain
        // Vulkan 1.1/1.3 feature structures onto an older physical device.
        let _core_features =
            unsafe { instance.get_physical_device_features(physical_device) };

        let (dynamic_rendering, synchronization2) =
            if api_version >= vk::API_VERSION_1_3 {
                let mut vulkan13 = vk::PhysicalDeviceVulkan13Features::default();
                let mut features =
                    vk::PhysicalDeviceFeatures2::default().push_next(&mut vulkan13);
                unsafe {
                    instance.get_physical_device_features2(physical_device, &mut features);
                }
                (
                    vulkan13.dynamic_rendering == vk::TRUE,
                    vulkan13.synchronization2 == vk::TRUE,
                )
            } else {
                (false, false)
            };

        let mode = if dynamic_rendering && synchronization2 {
            VulkanRendererMode::ModernDynamic
        } else {
            VulkanRendererMode::CompatibilityPending
        };

        Self {
            api_version,
            dynamic_rendering,
            synchronization2,
            mode,
        }
    }

    pub fn api_version(&self) -> u32 {
        self.api_version
    }

    pub fn mode(&self) -> VulkanRendererMode {
        self.mode
    }

    pub fn supports_dynamic_rendering(&self) -> bool {
        self.dynamic_rendering
    }

    pub fn supports_synchronization2(&self) -> bool {
        self.synchronization2
    }
}

/// Verified Vulkan 1.3 features required by the currently implemented renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanRenderingFeaturePlan {
    dynamic_rendering: bool,
    synchronization2: bool,
}

impl VulkanRenderingFeaturePlan {
    /// Creates a feature plan only for the currently implemented modern renderer.
    /// CompatibilityPending is deliberately not accepted until its render-pass
    /// pipelines, render scopes and synchronization path are integrated.
    pub fn from_device(device: &PhysicalDevice) -> Result<Self, String> {
        let capabilities = VulkanRenderingCapabilities::inspect(device);

        if capabilities.mode() != VulkanRendererMode::ModernDynamic {
            return Err(format!(
                "GPU '{}' exposes Vulkan API version {} but does not satisfy the implemented modern renderer; compatibility rendering is not yet integrated",
                device.name(),
                capabilities.api_version()
            ));
        }

        Ok(Self {
            dynamic_rendering: capabilities.supports_dynamic_rendering(),
            synchronization2: capabilities.supports_synchronization2(),
        })
    }

    /// Whether Vulkan dynamic rendering was verified and enabled.
    pub fn supports_dynamic_rendering(&self) -> bool {
        self.dynamic_rendering
    }

    /// Whether Vulkan Synchronization 2 was verified and enabled.
    pub fn supports_synchronization2(&self) -> bool {
        self.synchronization2
    }

    /// Builds the feature structure passed to vkCreateDevice.
    pub(super) fn enabled_features(&self) -> vk::PhysicalDeviceVulkan13Features<'static> {
        vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(self.dynamic_rendering)
            .synchronization2(self.synchronization2)
    }
}
