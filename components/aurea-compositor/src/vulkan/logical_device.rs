// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan logical-device and command-pool resource management.

use std::sync::{\n    Arc, Mutex,\n    atomic::{AtomicBool, Ordering},\n};

use smithay::reexports::ash::{self, vk};

use super::{
    VulkanContext, VulkanDeviceExtensionPlan, VulkanQueueSubmitter, VulkanRenderingFeaturePlan,
    queue_submitter::VulkanQueueSubmitError,
};

/// A logical Vulkan device, graphics queue and command pool.
pub struct VulkanLogicalDevice {
    // Keep the Vulkan instance alive until this logical device is destroyed.
    pub(super) _context: Arc<VulkanContext>,
    pub(super) device: ash::Device,
    pub(super) graphics_queue: vk::Queue,
    pub(super) graphics_queue_family: u32,
    pub(super) queue_submit_lock: Mutex<()>,
    pub(super) command_pool_lock: Mutex<()>,
    pub(super) command_pool: vk::CommandPool,
    pub(super) extension_plan: VulkanDeviceExtensionPlan,
    pub(super) rendering_features: VulkanRenderingFeaturePlan,
    pub(super) memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub(super) instance: ash::Instance,
    pub(super) physical_device: vk::PhysicalDevice,
    pub(super) supports_sync_fd: bool,
    pub(super) supports_foreign_queue_family: bool,
}

impl VulkanLogicalDevice {
    /// Returns the underlying Vulkan device handle.
    pub(super) fn handle(&self) -> &ash::Device {
        &self.device
    }

    /// Returns physical-device memory types for allocation decisions.
    pub(super) fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    /// Vulkan instance used to query physical-device image capabilities.
    pub(super) fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    /// Physical device paired with this logical device.
    pub(super) fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    /// Returns the graphics queue handle.
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    /// Returns the graphics queue-family index.
    pub fn graphics_queue_family(&self) -> u32 {
        self.graphics_queue_family
    }

    /// Returns the command pool used by this device.
    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    /// Returns the extension plan used to create this logical device.
    pub fn extension_plan(&self) -> &VulkanDeviceExtensionPlan {
        &self.extension_plan
    }

    /// Whether this device can export semaphore payloads as Linux sync files.
    pub fn supports_sync_fd(&self) -> bool {
        self.supports_sync_fd
    }

    /// Whether foreign queue-family ownership transfers are enabled on this device.
    pub fn supports_foreign_queue_family(&self) -> bool {
        self.supports_foreign_queue_family
    }

    /// Returns the rendering feature plan used to create this device.
    pub fn rendering_features(&self) -> &VulkanRenderingFeaturePlan {
        &self.rendering_features
    }

    /// Returns the rendering mode represented by this logical device.
    ///
    /// CompatibilityPending means device creation used only the legacy core
    /// feature model; it does not imply that frame composition or presentation
    /// is ready to use on this device.
    pub fn rendering_mode(&self) -> super::VulkanRendererMode {
        self.rendering_features.mode()
    }

    /// Allocates one primary command buffer from this device's command pool.
    pub(super) fn allocate_primary_command_buffer(&self) -> Result<vk::CommandBuffer, vk::Result> {
        // Vulkan requires external synchronization for command-pool access.
        let _pool_guard = self
            .command_pool_lock
            .lock()
            .map_err(|_| vk::Result::ERROR_UNKNOWN)?;
        let allocation_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffers = unsafe { self.device.allocate_command_buffers(&allocation_info) }?;
        Ok(command_buffers[0])
    }

    /// Frees a command buffer allocated from this device's pool.
    pub(super) fn free_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        // If synchronization state is poisoned, leaking this handle is safer
        // than making an unsynchronized Vulkan command-pool call.
        let Ok(_pool_guard) = self.command_pool_lock.lock() else {
            return;
        };
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &[command_buffer]);
        }
    }

    /// Submits an asynchronous command buffer under Vulkan queue external synchronization.
    pub(super) fn submit_async(
        &self,
        command_buffer: vk::CommandBuffer,
        fence: vk::Fence,
    ) -> Result<(), VulkanQueueSubmitError> {
        self.submit_async_with_signal(command_buffer, fence, None)
    }

    /// Submits a command buffer while signaling an optional semaphore.
    pub(super) fn submit_async_with_signal(
        &self,
        command_buffer: vk::CommandBuffer,
        fence: vk::Fence,
        signal_semaphore: Option<vk::Semaphore>,
    ) -> Result<(), VulkanQueueSubmitError> {
        if self.is_unusable() {
            return Err(VulkanQueueSubmitError::Rejected(vk::Result::ERROR_DEVICE_LOST));
        }
        let _queue_guard = self
            .queue_submit_lock
            .lock()
            .map_err(|_| VulkanQueueSubmitError::Rejected(vk::Result::ERROR_UNKNOWN))?;
        let command_buffers = [command_buffer];
        let signal_semaphores = signal_semaphore.into_iter().collect::<Vec<_>>();
        let mut submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
        if !signal_semaphores.is_empty() {
            submit_info = submit_info.signal_semaphores(&signal_semaphores);
        }
        let result = unsafe {
            self.device
                .queue_submit(self.graphics_queue, &[submit_info], fence)
        }
        .map_err(VulkanQueueSubmitError::from_submit_error);
        if result.as_ref().is_err_and(|error| error.is_uncertain()) {
            self.mark_unusable();
        }
        result
    }

    /// Submits work and invokes its lifecycle hook only after queue acceptance.
    pub(super) fn submit_and_wait_with(
        &self,
        command_buffer: vk::CommandBuffer,
        on_submitted: impl FnOnce(),
    ) -> Result<(), VulkanQueueSubmitError> {
        if self.is_unusable() {
            return Err(VulkanQueueSubmitError::Rejected(vk::Result::ERROR_DEVICE_LOST));
        }
        let result = VulkanQueueSubmitter::new(\n            &self.device,\n            self.graphics_queue,\n            &self.queue_submit_lock,\n            &self.device_unusable,\n        )
            .submit_and_wait_with(command_buffer, on_submitted);
        if result.as_ref().is_err_and(|error| error.is_uncertain()) {
            self.mark_unusable();
        }
        result
    }
}

impl Drop for VulkanLogicalDevice {
    fn drop(&mut self) {
        // A failed idle wait does not prove that submitted work has stopped
        // using device-owned objects. Keep the device and its parent instance
        // alive rather than destroying the command pool under uncertain work.
        if unsafe { self.device.device_wait_idle() }.is_err() {
            // Keep the VkInstance alive for the intentionally retained VkDevice.
            std::mem::forget(Arc::clone(&self._context));
            return;
        }

        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
        }
    }
}
