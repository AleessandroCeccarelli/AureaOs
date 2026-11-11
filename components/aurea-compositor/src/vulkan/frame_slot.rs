// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-frame Vulkan command-pool and synchronization ownership.

use std::os::fd::{FromRawFd, OwnedFd};

use smithay::reexports::ash::{self, vk};

use super::VulkanLogicalDevice;

/// Resources that may be reused only after their previous GPU submission ends.
pub(super) struct VulkanFrameSlot {
    pub(super) command_pool: vk::CommandPool,
    pub(super) completion_fence: vk::Fence,
    command_buffer: Option<vk::CommandBuffer>,
    pub(super) sync_fd_semaphore: Option<vk::Semaphore>,
}

impl VulkanFrameSlot {
    pub(super) fn new(device: &VulkanLogicalDevice, queue_family: u32) -> Result<Self, vk::Result> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let command_pool = unsafe { device.handle().create_command_pool(&pool_info, None) }?;

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let completion_fence = match unsafe { device.handle().create_fence(&fence_info, None) } {
            Ok(fence) => fence,
            Err(error) => {
                unsafe { device.handle().destroy_command_pool(command_pool, None) };
                return Err(error);
            }
        };

        let sync_fd_semaphore = if device.supports_sync_fd() {
            match Self::create_sync_fd_semaphore(device) {
                Ok(semaphore) => Some(semaphore),
                Err(error) => {
                    unsafe {
                        device.handle().destroy_fence(completion_fence, None);
                        device.handle().destroy_command_pool(command_pool, None);
                    }
                    return Err(error);
                }
            }
        } else {
            None
        };

        Ok(Self {
            command_pool,
            completion_fence,
            command_buffer: None,
            sync_fd_semaphore,
        })
    }

    fn create_sync_fd_semaphore(device: &VulkanLogicalDevice) -> Result<vk::Semaphore, vk::Result> {
        let mut export_info = vk::ExportSemaphoreCreateInfo::default()
            .handle_types(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
        let create_info = vk::SemaphoreCreateInfo::default().push_next(&mut export_info);
        unsafe { device.handle().create_semaphore(&create_info, None) }
    }

    pub(super) fn prepare(
        &mut self,
        device: &VulkanLogicalDevice,
    ) -> Result<vk::CommandBuffer, vk::Result> {
        if self.completion_fence == vk::Fence::null() {
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        }

        unsafe {
            device
                .handle()
                .wait_for_fences(&[self.completion_fence], true, u64::MAX)?;
            device
                .handle()
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())?;
        }

        // Recreate the export semaphore only after this slot's completion
        // fence has signalled. This also recovers cleanly from an FD-export
        // failure after a queue submission was already accepted.
        if let Some(semaphore) = self.sync_fd_semaphore.take() {
            unsafe { device.handle().destroy_semaphore(semaphore, None) };
        }
        if device.supports_sync_fd() {
            self.sync_fd_semaphore = Some(Self::create_sync_fd_semaphore(device)?);
        }

        let command_buffer = match self.command_buffer {
            Some(command_buffer) => command_buffer,
            None => {
                let allocation = vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1);
                let command_buffer =
                    unsafe { device.handle().allocate_command_buffers(&allocation) }?[0];
                self.command_buffer = Some(command_buffer);
                command_buffer
            }
        };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .handle()
                .begin_command_buffer(command_buffer, &begin_info)?;
        }

        Ok(command_buffer)
    }

    pub(super) fn submit(
        &mut self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), vk::Result> {
        self.submit_internal(device, command_buffer, None)
    }

    pub(super) fn submit_with_sync_fd(
        &mut self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
    ) -> Result<OwnedFd, vk::Result> {
        let semaphore = self
            .sync_fd_semaphore
            .ok_or(vk::Result::ERROR_FEATURE_NOT_PRESENT)?;
        self.submit_internal(device, command_buffer, Some(semaphore))?;

        let fd_info = vk::SemaphoreGetFdInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
        let loader =
            ash::khr::external_semaphore_fd::Device::new(device.instance(), device.handle());
        let raw_fd = unsafe { loader.get_semaphore_fd(&fd_info) }?;
        Ok(unsafe { OwnedFd::from_raw_fd(raw_fd) })
    }

    fn submit_internal(
        &mut self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        signal_semaphore: Option<vk::Semaphore>,
    ) -> Result<(), vk::Result> {
        unsafe {
            device.handle().end_command_buffer(command_buffer)?;
            device.handle().reset_fences(&[self.completion_fence])?;
        }

        let command_buffers = [command_buffer];
        let signal_semaphores = signal_semaphore.into_iter().collect::<Vec<_>>();
        let mut submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
        if !signal_semaphores.is_empty() {
            submit_info = submit_info.signal_semaphores(&signal_semaphores);
        }
        let result = unsafe {
            device.handle().queue_submit(
                device.graphics_queue(),
                &[submit_info],
                self.completion_fence,
            )
        };

        if let Err(error) = result {
            unsafe { device.handle().destroy_fence(self.completion_fence, None) };
            let signaled = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            self.completion_fence = unsafe { device.handle().create_fence(&signaled, None) }
                .unwrap_or(vk::Fence::null());
            return Err(error);
        }

        Ok(())
    }
}
