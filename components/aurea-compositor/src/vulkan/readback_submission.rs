// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owned lifetime for asynchronous Vulkan image readback submissions.

use std::sync::Arc;

use smithay::reexports::ash::vk;

use super::{
    VulkanImageView, VulkanLogicalDevice, VulkanReadbackBufferLease, VulkanReadbackBufferPool,
    VulkanReadbackError,
};

/// Owns every resource referenced by an asynchronous image readback.
///
/// The source view must own a shared image. The submission then retains both
/// that image and its logical device independently of the caller's stack.
pub struct VulkanReadbackSubmission {
    device: Arc<VulkanLogicalDevice>,
    source_view: Option<VulkanImageView<'static, 'static>>,
    readback: Option<VulkanReadbackBufferLease>,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    completed: bool,
    pixels_read: bool,
}

impl VulkanReadbackSubmission {
    /// Records and submits a copy from an owned image view.
    ///
    /// The caller must transition the source image to TRANSFER_SRC_OPTIMAL
    /// and synchronize prior writes before calling this method. The submission
    /// retains the view, image, staging allocation and device until completion.
    /// Returns after queue acceptance; poll it from a capture worker.
    ///
    /// The supplied pool must belong to the same logical device as the image.
    pub fn submit(
        view: VulkanImageView<'static, 'static>,
        pool: &VulkanReadbackBufferPool,
    ) -> Result<Self, VulkanReadbackError> {
        let device = view
            .shared_device_owner()
            .ok_or(VulkanReadbackError::ImageNotOwned)?;
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
            device.free_command_buffer(command_buffer);
            return Err(VulkanReadbackError::CommandBuffer(error));
        }

        if let Err(error) = readback.record_copy_rgba8(command_buffer, &view) {
            unsafe {
                device
                    .handle()
                    .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                    .ok();
            }
            device.free_command_buffer(command_buffer);
            return Err(error);
        }

        if let Err(error) = unsafe { device.handle().end_command_buffer(command_buffer) } {
            device.free_command_buffer(command_buffer);
            return Err(VulkanReadbackError::CommandBuffer(error));
        }

        let fence_info = vk::FenceCreateInfo::default();
        let fence = match unsafe { device.handle().create_fence(&fence_info, None) } {
            Ok(fence) => fence,
            Err(error) => {
                device.free_command_buffer(command_buffer);
                return Err(VulkanReadbackError::FenceCreation(error));
            }
        };

        if let Err(error) = device.submit_async(command_buffer, fence) {
            if error.is_uncertain() {
                // Queue acceptance is uncertain. Quarantine the complete
                // resource set, including the source image and device owner.
                std::mem::forget(view);
                std::mem::forget(readback);
                std::mem::forget(Arc::clone(&device));
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
            Err(error) => Err(VulkanReadbackError::FenceStatus(error)),
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
        if !self.completed {
            let wait = unsafe {
                self.device
                    .handle()
                    .wait_for_fences(&[self.fence], true, u64::MAX)
            };
            if wait.is_err()
                && unsafe { self.device.handle().device_wait_idle() }.is_err()
            {
                // Device loss leaves completion uncertain. Leak all resources
                // that may still be referenced by submitted GPU work.
                if let Some(readback) = self.readback.take() {
                    std::mem::forget(readback);
                }
                if let Some(view) = self.source_view.take() {
                    std::mem::forget(view);
                }
                std::mem::forget(Arc::clone(&self.device));
                return;
            }
        }

        unsafe { self.device.handle().destroy_fence(self.fence, None) };
        self.device.free_command_buffer(self.command_buffer);
    }
}
