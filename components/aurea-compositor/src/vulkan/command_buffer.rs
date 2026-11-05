
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! RAII wrapper for Vulkan command-buffer recording and submission.

use smithay::reexports::ash::vk;

use super::VulkanLogicalDevice;

/// Owns a primary Vulkan command buffer.
///
/// Submission is synchronous. If completion cannot be confirmed,
/// the command buffer is not freed while it may still be in use.
pub struct VulkanCommandBuffer<'device> {
    device: &'device VulkanLogicalDevice,
    handle: vk::CommandBuffer,
    may_be_pending: bool,
}

impl<'device> VulkanCommandBuffer<'device> {
    /// Allocates a primary command buffer from the device's command pool.
    pub fn new(
        device: &'device VulkanLogicalDevice,
    ) -> Result<Self, vk::Result> {
        let handle = device.allocate_primary_command_buffer()?;

        Ok(Self {
            device,
            handle,
            may_be_pending: false,
        })
    }

    /// Begins recording commands.
    pub fn begin(
        &self,
        usage: vk::CommandBufferUsageFlags,
    ) -> Result<(), vk::Result> {
        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(usage);

        unsafe {
            self.device
                .handle()
                .begin_command_buffer(self.handle, &begin_info)
        }
    }

    /// Ends recording commands.
    pub fn end(&self) -> Result<(), vk::Result> {
        unsafe {
            self.device
                .handle()
                .end_command_buffer(self.handle)
        }
    }

    /// Resets the command buffer so it can be recorded again.
    ///
    /// The buffer must not be pending execution.
    pub fn reset(
        &self,
        flags: vk::CommandBufferResetFlags,
    ) -> Result<(), vk::Result> {
        if self.may_be_pending {
            return Err(vk::Result::NOT_READY);
        }

        unsafe {
            self.device
                .handle()
                .reset_command_buffer(self.handle, flags)
        }
    }

    /// Submits the recorded command buffer and waits for completion.
    ///
    /// The command buffer must have been ended before submission.
    pub fn submit_and_wait(&mut self) -> Result<(), vk::Result> {
        self.may_be_pending = true;

        self.device.submit_and_wait(self.handle)?;

        self.may_be_pending = false;

        Ok(())
    }
}

impl Drop for VulkanCommandBuffer<'_> {
    fn drop(&mut self) {
        if self.may_be_pending {
            // Do not free a command buffer that may still be in use.
            let wait_result = unsafe {
                self.device.handle().device_wait_idle()
            };

            if wait_result.is_err() {
                // Keep the command buffer allocated rather than risk
                // freeing a resource still referenced by the GPU.
                return;
            }
        }

        self.device.free_command_buffer(self.handle);
    }
}
