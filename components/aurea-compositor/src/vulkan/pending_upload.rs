// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Fence-backed ownership for asynchronous Vulkan upload submissions.

use std::{mem, sync::Arc};

use smithay::reexports::ash::vk;

use super::{VulkanLogicalDevice, VulkanUploadBatch};

/// Owns every temporary resource until its queue submission has completed.
pub(crate) struct VulkanPendingUpload<T> {
    device: Arc<VulkanLogicalDevice>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    staging: Option<VulkanUploadBatch>,
    resources: Option<T>,
    submitted: bool,
}

impl<T> VulkanPendingUpload<T> {
    pub(super) fn new(
        device: Arc<VulkanLogicalDevice>,
        staging: VulkanUploadBatch,
        resources: T,
    ) -> Result<Self, vk::Result> {
        if device.is_unusable() {
            return Err(vk::Result::ERROR_DEVICE_LOST);
        }
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(device.graphics_queue_family())
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let command_pool = unsafe { device.handle().create_command_pool(&pool_info, None) }?;

        let allocation = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = match unsafe { device.handle().allocate_command_buffers(&allocation) }
        {
            Ok(buffers) => buffers[0],
            Err(error) => {
                unsafe { device.handle().destroy_command_pool(command_pool, None) };
                return Err(error);
            }
        };

        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        if let Err(error) = unsafe { device.handle().begin_command_buffer(command_buffer, &begin) }
        {
            unsafe { device.handle().destroy_command_pool(command_pool, None) };
            return Err(error);
        }

        Ok(Self {
            device,
            command_pool,
            command_buffer,
            fence: vk::Fence::null(),
            staging: Some(staging),
            resources: Some(resources),
            submitted: false,
        })
    }

    pub(super) fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub(super) fn staging(&self) -> &VulkanUploadBatch {
        self.staging
            .as_ref()
            .expect("staging allocation retained until completion")
    }

    pub(super) fn resources(&self) -> &T {
        self.resources
            .as_ref()
            .expect("upload resources retained until completion")
    }

    /// Ends and submits the command buffer without waiting for the GPU.
    pub(super) fn submit(mut self) -> Result<Self, vk::Result> {
        unsafe {
            self.device
                .handle()
                .end_command_buffer(self.command_buffer)?
        };

        let fence_info = vk::FenceCreateInfo::default();
        self.fence = unsafe { self.device.handle().create_fence(&fence_info, None) }?;
        if let Err(error) = self.device.submit_async(self.command_buffer, self.fence) {
            if error.is_uncertain() {
                // Device loss makes queue acceptance uncertain. Keep every
                // upload resource and synchronization handle quarantined.
                self.submitted = true;
            } else {
                unsafe { self.device.handle().destroy_fence(self.fence, None) };
                self.fence = vk::Fence::null();
            }
            return Err(error.result());
        }

        self.submitted = true;
        Ok(self)
    }

    /// Polls the fence without blocking the calling thread.
    pub(super) fn is_complete(&self) -> Result<bool, vk::Result> {
        if !self.submitted || self.fence == vk::Fence::null() {
            return Err(vk::Result::ERROR_VALIDATION_FAILED_EXT);
        }
        match unsafe { self.device.handle().wait_for_fences(&[self.fence], true, 0) } {
            Ok(()) => Ok(true),
            Err(vk::Result::TIMEOUT) => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Waits only for this upload and transfers its resources to the caller.
    pub(super) fn wait(mut self) -> Result<T, vk::Result> {
        if !self.is_complete()? {
            unsafe {
                self.device
                    .handle()
                    .wait_for_fences(&[self.fence], true, u64::MAX)?;
            }
        }

        self.cleanup();
        drop(self.staging.take());
        self.resources
            .take()
            .ok_or(vk::Result::ERROR_VALIDATION_FAILED_EXT)
    }

    fn cleanup(&mut self) {
        if self.device.is_unusable() {
            return;
        }
        unsafe {
            if self.fence != vk::Fence::null() {
                self.device.handle().destroy_fence(self.fence, None);
                self.fence = vk::Fence::null();
            }
            if self.command_pool != vk::CommandPool::null() {
                self.device
                    .handle()
                    .destroy_command_pool(self.command_pool, None);
                self.command_pool = vk::CommandPool::null();
            }
        }
        self.submitted = false;
    }
}

impl<T> Drop for VulkanPendingUpload<T> {
    fn drop(&mut self) {
        if self.device.is_unusable() {
            if let Some(staging) = self.staging.take() {
                mem::forget(staging);
            }
            if let Some(resources) = self.resources.take() {
                mem::forget(resources);
            }
            mem::forget(Arc::clone(&self.device));
            self.fence = vk::Fence::null();
            self.command_pool = vk::CommandPool::null();
            self.submitted = false;
            return;
        }
        if self.submitted {
            let fence_result = unsafe {
                self.device
                    .handle()
                    .wait_for_fences(&[self.fence], true, u64::MAX)
            };
            if fence_result.is_err() && unsafe { self.device.handle().device_wait_idle() }.is_err()
            {
                if let Some(staging) = self.staging.take() {
                    mem::forget(staging);
                }
                if let Some(resources) = self.resources.take() {
                    mem::forget(resources);
                }
                mem::forget(Arc::clone(&self.device));
                self.fence = vk::Fence::null();
                self.command_pool = vk::CommandPool::null();
                self.submitted = false;
                return;
            }
        }
        self.cleanup();
    }
}
