// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan command submission and fence synchronization.

use std::sync::Mutex;

use smithay::reexports::ash::{self, vk};

/// Distinguishes rejected submissions from submissions with uncertain completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VulkanQueueSubmitError {
    Rejected(vk::Result),
    SubmissionUncertain(vk::Result),
    CompletionUncertain {
        wait_error: vk::Result,
        idle_error: vk::Result,
    },
}

impl VulkanQueueSubmitError {
    pub(super) const fn from_submit_error(error: vk::Result) -> Self {
        if matches!(error, vk::Result::ERROR_DEVICE_LOST) {
            Self::SubmissionUncertain(error)
        } else {
            Self::Rejected(error)
        }
    }

    pub(super) const fn is_uncertain(self) -> bool {
        matches!(
            self,
            Self::SubmissionUncertain(_) | Self::CompletionUncertain { .. }
        )
    }

    pub(super) const fn result(self) -> vk::Result {
        match self {
            Self::Rejected(error) | Self::SubmissionUncertain(error) => error,
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
        Self {
            device,
            queue,
            queue_submit_lock,
        }
    }

    /// Notifies the caller after acceptance or uncertain acceptance, then waits for completion.
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
            let submit_error = VulkanQueueSubmitError::from_submit_error(error);
            if submit_error.is_uncertain() {
                // Device loss makes queue acceptance uncertain. Conservatively
                // advance the caller's resource lifecycle and preserve the fence.
                on_submitted();
            } else {
                unsafe { self.device.destroy_fence(fence, None) };
            }
            return Err(submit_error);
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

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_lost_submission_is_not_treated_as_rejected() {
        assert_eq!(
            VulkanQueueSubmitError::from_submit_error(vk::Result::ERROR_DEVICE_LOST),
            VulkanQueueSubmitError::SubmissionUncertain(vk::Result::ERROR_DEVICE_LOST)
        );
    }

    #[test]
    fn ordinary_submit_errors_are_rejected() {
        assert_eq!(
            VulkanQueueSubmitError::from_submit_error(vk::Result::ERROR_OUT_OF_HOST_MEMORY),
            VulkanQueueSubmitError::Rejected(vk::Result::ERROR_OUT_OF_HOST_MEMORY)
        );
    }
}
