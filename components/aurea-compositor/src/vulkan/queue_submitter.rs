// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan command submission and fence synchronization.

use std::sync::Mutex;

use smithay::reexports::ash::{self, vk};

/// Distinguishes a rejected queue submission from uncertain GPU completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VulkanQueueSubmitError {
    Rejected(vk::Result),
    CompletionUncertain { wait_error: vk::Result, idle_error: vk::Result },
}

impl VulkanQueueSubmitError {
    pub(super) const fn result(self) -> vk::Result {
        match self {
            Self::Rejected(error) => error,
            Self::CompletionUncertain { wait_error, .. } => wait_error,
        }
    }
}

/// Submits command buffers to a Vulkan queue and waits for completion.
pub struct VulkanQueueSubmitter<'device> {
    device: &'device ash::Device,
    queue: vk::Queue,
    queue_submit_lock: &'device Mutex<()>,
}

impl<'device> VulkanQueueSubmitter<'device> {
    /// Creates a submitter for the specified device queue.
    pub(super) fn new(
        device: &'device ash::Device,
        queue: vk::Queue,
        queue_submit_lock: &'device Mutex<()>,
    ) -> Self {
        Self { device, queue, queue_submit_lock }
    }

    /// Notifies the caller after queue acceptance, then waits for completion.
    pub(super) fn submit_and_wait_with(
        &self,
        command_buffer: vk::CommandBuffer,
        on_submitted: impl FnOnce(),
    ) -> Result<(), VulkanQueueSubmitError> {
        let command_buffers = [command_buffer];
        let submit_info = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
        let fence_info = vk::FenceCreateInfo::default();
        let fence = unsafe { self.device.create_fence(&fence_info, None) }
            .map_err(VulkanQueueSubmitError::Rejected)?;

        let submit_result = {
            let _queue_guard = match self.queue_submit_lock.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    unsafe { self.device.destroy_fence(fence, None) };
                    return Err(VulkanQueueSubmitError::Rejected(vk::Result::ERROR_UNKNOWN));
                }
            };
            unsafe { self.device.queue_submit(self.queue, &submit_info, fence) }
        };

        if let Err(error) = submit_result {
            unsafe { self.device.destroy_fence(fence, None) };
            return Err(VulkanQueueSubmitError::Rejected(error));
        }

        // The queue owns the work from this point onward.
        on_submitted();

        let wait_result = unsafe { self.device.wait_for_fences(&[fence], true, u64::MAX) };
        if wait_result.is_ok() {
            unsafe { self.device.destroy_fence(fence, None) };
            return Ok(());
        }

        // A successful device-idle fallback is also proof of completion.
        let idle_result = unsafe { self.device.device_wait_idle() };
        if idle_result.is_ok() {
            unsafe { self.device.destroy_fence(fence, None) };
            return Ok(());
        }

        // Keep the fence alive while completion remains uncertain.
        Err(VulkanQueueSubmitError::CompletionUncertain {
            wait_error: wait_result.expect_err("wait result checked above"),
            idle_error: idle_result.expect_err("idle result checked above"),
        })
    }

    /// Submits a recorded command buffer and waits for its completion.
    pub fn submit_and_wait(&self, command_buffer: vk::CommandBuffer) -> Result<(), vk::Result> {
        self.submit_and_wait_with(command_buffer, || {})
            .map_err(VulkanQueueSubmitError::result)
    }
}
