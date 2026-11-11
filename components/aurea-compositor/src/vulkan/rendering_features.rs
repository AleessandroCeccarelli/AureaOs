// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Required Vulkan 1.3 rendering-feature negotiation.

use smithay::{
    backend::vulkan::PhysicalDevice,
    reexports::ash::vk,
};

/// Verified Vulkan 1.3 features required by the native renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanRenderingFeaturePlan {
    dynamic_rendering: bool,
    synchronization2: bool,
}

impl VulkanRenderingFeaturePlan {
    /// Queries core Vulkan 1.3 capabilities on the selected physical device.
    pub fn from_device(device: &PhysicalDevice) -> Result<Self, String> {
        let instance = device.instance().handle();
        let physical_device = device.handle();

        let properties = unsafe {
            instance.get_physical_device_properties(physical_device)
        };

        if properties.api_version < vk::API_VERSION_1_3 {
            return Err(format!(
                "GPU '{}' exposes Vulkan API version {}, but AureaOS requires Vulkan 1.3",
                device.name(),
                properties.api_version
            ));
        }

        let mut vulkan13 = vk::PhysicalDeviceVulkan13Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vulkan13);

        unsafe {
            instance.get_physical_device_features2(physical_device, &mut features);
        }

        let plan = Self {
            dynamic_rendering: vulkan13.dynamic_rendering == vk::TRUE,
            synchronization2: vulkan13.synchronization2 == vk::TRUE,
        };

        if !plan.dynamic_rendering || !plan.synchronization2 {
            return Err(format!(
                "GPU '{}' lacks required Vulkan 1.3 features: {}{}",
                device.name(),
                if plan.dynamic_rendering { "" } else { "dynamicRendering " },
                if plan.synchronization2 { "" } else { "synchronization2" },
            ));
        }

        Ok(plan)
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
