// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan command submission and fence synchronization.

use std::sync::Mutex;

use smithay::reexports::ash::{self, vk};

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

    /// Submits a recorded command buffer and waits for its completion.
    ///
    /// The command buffer must have been ended and must not already be
    /// pending execution.
    pub fn submit_and_wait(&self, command_buffer: vk::CommandBuffer) -> Result<(), vk::Result> {
        let command_buffers = [command_buffer];

        let submit_info = [vk::SubmitInfo::default().command_buffers(&command_buffers)];

        let fence_info = vk::FenceCreateInfo::default();

        let fence = unsafe { self.device.create_fence(&fence_info, None) }?;

        let submit_result = {
            let _queue_guard = self.queue_submit_lock.lock().map_err(|_| {
                unsafe { self.device.destroy_fence(fence, None) };
                vk::Result::ERROR_UNKNOWN
            })?;
            unsafe { self.device.queue_submit(self.queue, &submit_info, fence) }
        };

        if let Err(error) = submit_result {
            unsafe {
                self.device.destroy_fence(fence, None);
            }

            return Err(error);
        }

        let wait_result = unsafe { self.device.wait_for_fences(&[fence], true, u64::MAX) };

        if wait_result.is_ok() {
            unsafe {
                self.device.destroy_fence(fence, None);
            }

            return Ok(());
        }

        let idle_result = unsafe { self.device.device_wait_idle() };

        if idle_result.is_ok() {
            unsafe {
                self.device.destroy_fence(fence, None);
            }
        }

        wait_result
    }
}
