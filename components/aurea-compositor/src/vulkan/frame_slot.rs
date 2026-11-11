// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-frame Vulkan command-pool and fence ownership.

use smithay::reexports::ash::vk;

use super::VulkanLogicalDevice;

/// Resources that may be reused only after their previous GPU submission ends.
pub(super) struct VulkanFrameSlot {
    pub(super) command_pool: vk::CommandPool,
    pub(super) completion_fence: vk::Fence,
}

impl VulkanFrameSlot {
    pub(super) fn new(
        device: &VulkanLogicalDevice,
        queue_family: u32,
    ) -> Result<Self, vk::Result> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);

        let command_pool = unsafe { device.handle().create_command_pool(&pool_info, None) }?;

        let fence_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let completion_fence = match unsafe {
            device.handle().create_fence(&fence_info, None)
        } {
            Ok(fence) => fence,
            Err(error) => {
                unsafe { device.handle().destroy_command_pool(command_pool, None) };
                return Err(error);
            }
        };

        Ok(Self { command_pool, completion_fence })
    }

    pub(super) fn prepare(
        &mut self,
        device: &VulkanLogicalDevice,
    ) -> Result<vk::CommandBuffer, vk::Result> {
        if self.completion_fence == vk::Fence::null() {
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        }

        unsafe {
            device.handle().wait_for_fences(
                &[self.completion_fence],
                true,
                u64::MAX,
            )?;
            device.handle().reset_command_pool(
                self.command_pool,
                vk::CommandPoolResetFlags::empty(),
            )?;
        }

        let allocation = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe {
            device.handle().allocate_command_buffers(&allocation)
        }?[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device.handle().begin_command_buffer(command_buffer, &begin_info)?;
        }

        Ok(command_buffer)
    }

    pub(super) fn submit(
        &mut self,
        device: &VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), vk::Result> {
        unsafe {
            device.handle().end_command_buffer(command_buffer)?;
            device.handle().reset_fences(&[self.completion_fence])?;
        }

        let command_buffers = [command_buffer];
        let submit_info = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
        let result = unsafe {
            device.handle().queue_submit(
                device.graphics_queue(),
                &submit_info,
                self.completion_fence,
            )
        };

        if let Err(error) = result {
            // queue_submit did not accept the work. Restore a signaled fence so
            // the slot cannot deadlock on its next acquisition.
            unsafe {
                device.handle().destroy_fence(self.completion_fence, None);
            }
            let signaled = vk::FenceCreateInfo::default()
                .flags(vk::FenceCreateFlags::SIGNALED);
            self.completion_fence = unsafe {
                device.handle().create_fence(&signaled, None)
            }.unwrap_or(vk::Fence::null());
            return Err(error);
        }

        Ok(())
    }
}

