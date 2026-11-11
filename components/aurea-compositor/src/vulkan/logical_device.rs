// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan logical-device and command-pool resource management.

use smithay::{
    backend::vulkan::PhysicalDevice,
    reexports::ash::{self, vk},
};

use super::{
    VulkanContext, VulkanDeviceExtensionPlan, VulkanInitError, VulkanQueueSubmitter, VulkanRenderingFeaturePlan,
};

/// A logical Vulkan device, graphics queue and command pool.
pub struct VulkanLogicalDevice {
    device: ash::Device,
    graphics_queue: vk::Queue,
    graphics_queue_family: u32,
    command_pool: vk::CommandPool,
    extension_plan: VulkanDeviceExtensionPlan,
    rendering_features: VulkanRenderingFeaturePlan,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    supports_sync_fd: bool,
    supports_foreign_queue_family: bool,
}

impl VulkanLogicalDevice {
    /// Creates a logical device and enables only extensions exposed by its GPU.
    pub(super) fn new(
        physical_device: &PhysicalDevice,
        queue_family_index: u32,
        extension_plan: VulkanDeviceExtensionPlan,
    ) -> Result<Self, VulkanInitError> {
        let supports_sync_fd = if extension_plan.supports_sync_fd() {
            let semaphore_info = vk::PhysicalDeviceExternalSemaphoreInfo::default()
                .handle_type(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
            let mut semaphore_properties = vk::ExternalSemaphoreProperties::default();
            unsafe {
                physical_device.instance().handle()
                    .get_physical_device_external_semaphore_properties(
                        physical_device.handle(),
                        &semaphore_info,
                        &mut semaphore_properties,
                    );
            }
            let features = semaphore_properties.external_semaphore_features;
            features.contains(vk::ExternalSemaphoreFeatureFlags::EXPORTABLE)
                && semaphore_properties.compatible_handle_types
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
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&extension_names)
            .push_next(&mut enabled_rendering_features);

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
            physical_device.instance().handle().get_physical_device_memory_properties(physical_device.handle())
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

        Ok(Self {
            device,
            graphics_queue,
            graphics_queue_family: queue_family_index,
            command_pool,
            extension_plan,
            rendering_features,
            memory_properties,
            instance: physical_device.instance().handle().clone(),
            physical_device: physical_device.handle(),
            supports_sync_fd,
            supports_foreign_queue_family: extension_plan.supports_foreign_queue_family(),
        })
    }

    /// Returns the underlying Vulkan device handle.
    pub(super) fn handle(&self) -> &ash::Device { &self.device }

    /// Returns physical-device memory types for allocation decisions.
    pub(super) fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    /// Vulkan instance used to query physical-device image capabilities.
    pub(super) fn instance(&self) -> &ash::Instance { &self.instance }

    /// Physical device paired with this logical device.
    pub(super) fn physical_device(&self) -> vk::PhysicalDevice { self.physical_device }

    /// Returns the graphics queue handle.
    pub fn graphics_queue(&self) -> vk::Queue { self.graphics_queue }

    /// Returns the graphics queue-family index.
    pub fn graphics_queue_family(&self) -> u32 { self.graphics_queue_family }

    /// Returns the command pool used by this device.
    pub fn command_pool(&self) -> vk::CommandPool { self.command_pool }

    /// Returns the extension plan used to create this logical device.
    pub fn extension_plan(&self) -> &VulkanDeviceExtensionPlan { &self.extension_plan }

    /// Whether this device can export semaphore payloads as Linux sync files.
    pub fn supports_sync_fd(&self) -> bool { self.supports_sync_fd }

    /// Whether foreign queue-family ownership transfers are enabled on this device.
    pub fn supports_foreign_queue_family(&self) -> bool {
        self.supports_foreign_queue_family
    }

    /// Returns the Vulkan 1.3 rendering features enabled on this device.
    pub fn rendering_features(&self) -> &VulkanRenderingFeaturePlan { &self.rendering_features }

    /// Allocates one primary command buffer from this device's command pool.
    pub(super) fn allocate_primary_command_buffer(&self) -> Result<vk::CommandBuffer, vk::Result> {
        let allocation_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffers = unsafe { self.device.allocate_command_buffers(&allocation_info) }?;
        Ok(command_buffers[0])
    }

    /// Frees a command buffer allocated from this device's pool.
    pub(super) fn free_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe { self.device.free_command_buffers(self.command_pool, &[command_buffer]); }
    }

    /// Submits a recorded command buffer and waits for its completion.
    pub(super) fn submit_and_wait(&self, command_buffer: vk::CommandBuffer) -> Result<(), vk::Result> {
        VulkanQueueSubmitter::new(&self.device, self.graphics_queue).submit_and_wait(command_buffer)
    }
}

impl Drop for VulkanLogicalDevice {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
        }
    }
}

impl VulkanContext {
    /// Creates a logical device for the selected physical device.
    pub fn create_logical_device(
        &self,
        preference: crate::renderer::RenderPreference,
    ) -> Result<VulkanLogicalDevice, VulkanInitError> {
        let physical_device = self.select_device(preference)
            .ok_or_else(|| VulkanInitError::new("No graphics-capable Vulkan device found"))?;
        let queue_family_index = self.graphics_queue_family(physical_device)
            .ok_or_else(|| VulkanInitError::new("Selected device has no graphics queue"))?;
        let extension_plan = VulkanDeviceExtensionPlan::from_device(physical_device);
        VulkanLogicalDevice::new(physical_device, queue_family_index, extension_plan)
    }
}
