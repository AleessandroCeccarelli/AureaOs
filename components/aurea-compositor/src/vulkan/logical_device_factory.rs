// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan logical-device creation and context selection.

use std::sync::{Arc, Mutex, atomic::AtomicBool};

use smithay::{backend::vulkan::PhysicalDevice, reexports::ash::vk};

use super::{
    VulkanContext, VulkanDeviceExtensionPlan, VulkanInitError, VulkanLogicalDevice,
    VulkanRenderingFeaturePlan,
};

impl VulkanLogicalDevice {
    /// Creates a logical device and enables only extensions exposed by its GPU.
    pub(super) fn new(
        physical_device: &PhysicalDevice,
        queue_family_index: u32,
        extension_plan: VulkanDeviceExtensionPlan,
        context_owner: Arc<VulkanContext>,
    ) -> Result<Self, VulkanInitError> {
        let supports_sync_fd = if extension_plan.supports_sync_fd() {
            let semaphore_info = vk::PhysicalDeviceExternalSemaphoreInfo::default()
                .handle_type(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
            let mut semaphore_properties = vk::ExternalSemaphoreProperties::default();
            unsafe {
                physical_device
                    .instance()
                    .handle()
                    .get_physical_device_external_semaphore_properties(
                        physical_device.handle(),
                        &semaphore_info,
                        &mut semaphore_properties,
                    );
            }
            let features = semaphore_properties.external_semaphore_features;
            features.contains(vk::ExternalSemaphoreFeatureFlags::EXPORTABLE)
                && semaphore_properties
                    .compatible_handle_types
                    .contains(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD)
        } else {
            false
        };
        let rendering_features = VulkanRenderingFeaturePlan::from_device(physical_device)
            .map_err(VulkanInitError::new)?;
        let priorities = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];
        let extension_names = extension_plan.enabled_name_pointers();
        let mut enabled_rendering_features = rendering_features.enabled_features();
        let mut create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&extension_names);
        if let Some(features) = enabled_rendering_features.as_mut() {
            create_info = create_info.push_next(features);
        }

        let device = unsafe {
            physical_device.instance().handle().create_device(
                physical_device.handle(),
                &create_info,
                None,
            )
        }
        .map_err(|error| {
            VulkanInitError::new(format!("Failed to create Vulkan logical device: {error}"))
        })?;

        let memory_properties = unsafe {
            physical_device
                .instance()
                .handle()
                .get_physical_device_memory_properties(physical_device.handle())
        };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = match unsafe { device.create_command_pool(&pool_info, None) } {
            Ok(pool) => pool,
            Err(error) => {
                unsafe { device.destroy_device(None) };
                return Err(VulkanInitError::new(format!(
                    "Failed to create Vulkan command pool: {error}"
                )));
            }
        };

        let supports_foreign_queue_family = extension_plan.supports_foreign_queue_family();

        Ok(Self {
            _context: context_owner,
            device,
            graphics_queue,
            graphics_queue_family: queue_family_index,
            queue_submit_lock: Mutex::new(()),
            command_pool_lock: Mutex::new(()),
            command_pool,
            extension_plan,
            rendering_features,
            memory_properties,
            instance: physical_device.instance().handle().clone(),
            physical_device: physical_device.handle(),
            supports_sync_fd,
            supports_foreign_queue_family,
        })
    }
}

impl VulkanContext {
    /// Creates a logical device for the selected physical device.
    pub fn create_logical_device(
        self: &Arc<Self>,
        preference: crate::renderer::RenderPreference,
    ) -> Result<VulkanLogicalDevice, VulkanInitError> {
        let physical_device = self
            .select_device(preference)
            .ok_or_else(|| VulkanInitError::new("No graphics-capable Vulkan device found"))?;
        let queue_family_index = self
            .graphics_queue_family(physical_device)
            .ok_or_else(|| VulkanInitError::new("Selected device has no graphics queue"))?;
        let extension_plan = VulkanDeviceExtensionPlan::from_device(physical_device);
        VulkanLogicalDevice::new(
            physical_device,
            queue_family_index,
            extension_plan,
            Arc::clone(self),
        )
    }
}
