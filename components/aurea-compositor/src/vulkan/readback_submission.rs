// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owned lifetime for asynchronous Vulkan image readback submissions.

use std::sync::Arc;

use smithay::reexports::ash::vk;

use super::{
    VulkanCaptureTargetLease, VulkanImageView, VulkanLogicalDevice, VulkanReadbackBufferLease,
    VulkanReadbackBufferPool, VulkanReadbackError,
};

/// Owns every resource referenced by an asynchronous image readback.
///
/// The source view must own a shared image. Capture-target submissions also
/// retain the pool lease, preventing the target from being checked out again
/// until readback has completed and the CPU pixel copy has been consumed.
pub struct VulkanReadbackSubmission {
    device: Arc<VulkanLogicalDevice>,
    source_view: Option<VulkanImageView<'static, 'static>>,
    readback: Option<VulkanReadbackBufferLease>,
    _capture_target: Option<VulkanCaptureTargetLease>,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    completed: bool,
    pixels_read: bool,
}

impl VulkanReadbackSubmission {
    /// Records and submits a copy from an owned image view.
    ///
    /// Call this method from a capture worker, not the compositor presentation
    /// path. The caller must transition the source image to TRANSFER_SRC_OPTIMAL
    /// and synchronize preceding writes before calling this method.
    pub fn submit(
        view: VulkanImageView<'static, 'static>,
        pool: &VulkanReadbackBufferPool,
    ) -> Result<Self, VulkanReadbackError> {
        Self::submit_inner(view, pool, None)
    }

    /// Submits readback while retaining the capture target's pool lease.
    ///
    /// The lease is released only when this submission is retired or safely
    /// dropped. If queue acceptance becomes uncertain, the target is
    /// intentionally quarantined with the other referenced GPU resources.
    pub fn submit_capture_target(
        target: VulkanCaptureTargetLease,
        pool: &VulkanReadbackBufferPool,
    ) -> Result<Self, VulkanReadbackError> {
        let view = target.create_owned_view().map_err(|error| match error {
            super::VulkanImageViewError::Creation(result) => VulkanReadbackError::ImageView(result),
        })?;
        Self::submit_inner(view, pool, Some(target))
    }

    fn submit_inner(
        view: VulkanImageView<'static, 'static>,
        pool: &VulkanReadbackBufferPool,
        capture_target: Option<VulkanCaptureTargetLease>,
    ) -> Result<Self, VulkanReadbackError> {
        let device = view
            .shared_device_owner()
            .ok_or(VulkanReadbackError::ImageNotOwned)?;
        if device.is_unusable() {
            quarantine_readback_inputs(&device, view, None, capture_target);
            return Err(VulkanReadbackError::Submission(vk::Result::ERROR_DEVICE_LOST));
        }
        if !pool.owns_device(&device) {
            return Err(VulkanReadbackError::DeviceMismatch);
        }
        let readback = pool.acquire(view.width(), view.height())?;
        let command_buffer = device
            .allocate_primary_command_buffer()
            .map_err(VulkanReadbackError::CommandBuffer)?;

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        if let Err(error) = unsafe {
            device
                .handle()
                .begin_command_buffer(command_buffer, &begin_info)
        } {
            if error == vk::Result::ERROR_DEVICE_LOST {
                device.mark_unusable();
                quarantine_readback_inputs(&device, view, Some(readback), capture_target);
            } else {
                device.free_command_buffer(command_buffer);
            }
            return Err(VulkanReadbackError::CommandBuffer(error));
        }

        if let Err(error) = readback.record_copy_rgba8(command_buffer, &view) {
            let reset_result = unsafe {
                device
                    .handle()
                    .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
            };
            let device_lost = matches!(
                error,
                VulkanReadbackError::CommandBuffer(vk::Result::ERROR_DEVICE_LOST)
            ) || reset_result == Err(vk::Result::ERROR_DEVICE_LOST);
            if device_lost {
                device.mark_unusable();
                quarantine_readback_inputs(&device, view, Some(readback), capture_target);
            } else {
                device.free_command_buffer(command_buffer);
            }
            return Err(error);
        }

        if let Err(error) = unsafe { device.handle().end_command_buffer(command_buffer) } {
            if error == vk::Result::ERROR_DEVICE_LOST {
                device.mark_unusable();
                quarantine_readback_inputs(&device, view, Some(readback), capture_target);
            } else {
                device.free_command_buffer(command_buffer);
            }
            return Err(VulkanReadbackError::CommandBuffer(error));
        }

        let fence_info = vk::FenceCreateInfo::default();
        let fence = match unsafe { device.handle().create_fence(&fence_info, None) } {
            Ok(fence) => fence,
            Err(error) => {
                if error == vk::Result::ERROR_DEVICE_LOST {
                    device.mark_unusable();
                    quarantine_readback_inputs(&device, view, Some(readback), capture_target);
                } else {
                    device.free_command_buffer(command_buffer);
                }
                return Err(VulkanReadbackError::FenceCreation(error));
            }
        };

        if let Err(error) = device.submit_async(command_buffer, fence) {
            if error.is_uncertain() || device.is_unusable() {
                // Queue acceptance is uncertain. Quarantine the complete
                // resource set, including the capture target and device owner.
                std::mem::forget(view);
                std::mem::forget(readback);
                std::mem::forget(capture_target);
                return Err(VulkanReadbackError::SubmissionUncertain(error.result()));
            }

            unsafe { device.handle().destroy_fence(fence, None) };
            device.free_command_buffer(command_buffer);
            return Err(VulkanReadbackError::Submission(error.result()));
        }

        Ok(Self {
            device,
            source_view: Some(view),
            readback: Some(readback),
            _capture_target: capture_target,
            command_buffer,
            fence,
            completed: false,
            pixels_read: false,
        })
    }

    /// Checks completion without waiting. False means the GPU is still working.
    pub fn poll(&mut self) -> Result<bool, VulkanReadbackError> {
        if self.completed {
            return Ok(true);
        }

        match unsafe { self.device.handle().get_fence_status(self.fence) } {
            Ok(true) => {
                self.completed = true;
                Ok(true)
            }
            Ok(false) | Err(vk::Result::NOT_READY) => Ok(false),
            Err(error) => {
                if error == vk::Result::ERROR_DEVICE_LOST {
                    self.device.mark_unusable();
                }
                Err(VulkanReadbackError::FenceStatus(error))
            },
        }
    }

    /// Consumes completed pixel data. This method never waits for the GPU.
    pub fn take_rgba8(&mut self) -> Result<Vec<u8>, VulkanReadbackError> {
        if !self.poll()? {
            return Err(VulkanReadbackError::SubmissionNotComplete);
        }
        if self.pixels_read {
            return Err(VulkanReadbackError::PixelsAlreadyRead);
        }

        let pixels = self
            .readback
            .as_ref()
            .ok_or(VulkanReadbackError::PixelsAlreadyRead)?
            .read_rgba8()?;
        self.pixels_read = true;
        Ok(pixels)
    }

    pub const fn is_complete(&self) -> bool {
        self.completed
    }

    pub fn byte_len(&self) -> usize {
        self.readback
            .as_ref()
            .map_or(0, |readback| readback.byte_len())
    }
}

impl Drop for VulkanReadbackSubmission {
    fn drop(&mut self) {
        if self.device.is_unusable() {
            if let Some(readback) = self.readback.take() {
                std::mem::forget(readback);
            }
            if let Some(view) = self.source_view.take() {
                std::mem::forget(view);
            }
            if let Some(target) = self._capture_target.take() {
                std::mem::forget(target);
            }
            std::mem::forget(std::sync::Arc::clone(&self.device));
            return;
        }
        // This fallback preserves resource safety for callers that release an
        // unfinished capture. Normal operation polls to completion on a worker.
        if !self.completed {
            let wait = unsafe {
                self.device
                    .handle()
                    .wait_for_fences(&[self.fence], true, u64::MAX)
            };
            if wait.is_err() && unsafe { self.device.handle().device_wait_idle() }.is_err() {
                self.device.mark_unusable();
                // Device loss leaves completion uncertain. Leak every resource
                // that may still be referenced by submitted GPU work.
                if let Some(readback) = self.readback.take() {
                    std::mem::forget(readback);
                }
                if let Some(view) = self.source_view.take() {
                    std::mem::forget(view);
                }
                if let Some(target) = self._capture_target.take() {
                    std::mem::forget(target);
                }
                return;
            }
        }

        unsafe { self.device.handle().destroy_fence(self.fence, None) };
        self.device.free_command_buffer(self.command_buffer);
    }
}


/// Retains every resource whose destructor may touch a lost Vulkan device.
fn quarantine_readback_inputs(
    device: &Arc<VulkanLogicalDevice>,
    view: VulkanImageView<'static, 'static>,
    readback: Option<VulkanReadbackBufferLease>,
    capture_target: Option<VulkanCaptureTargetLease>,
) {
    std::mem::forget(view);
    if let Some(readback) = readback {
        std::mem::forget(readback);
    }
    std::mem::forget(capture_target);
    std::mem::forget(Arc::clone(device));
}
