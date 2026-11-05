
 // SPDX-License-Identifier: GPL-3.0-or-later
 //
 // AureaOS
 // Copyright (C) 2026 Alessandro Ceccarelli

//! RAII wrapper for Vulkan command-buffer lifecycle management.

use smithay::reexports::ash::vk;

use super::VulkanLogicalDevice;

/// Internal lifecycle state of a Vulkan command buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandBufferState {
    Initial,
    Recording,
    Executable,
    Pending,
    Invalid,
}

/// Owns a primary Vulkan command buffer.
///
/// Lifecycle transitions are validated before Vulkan operations.
/// Submission waits synchronously for completion.
pub struct VulkanCommandBuffer<'device> {
    device: &'device VulkanLogicalDevice,
    handle: vk::CommandBuffer,
    state: CommandBufferState,
    one_time_submit: bool,
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
            state: CommandBufferState::Initial,
            one_time_submit: false,
        })
    }

    /// Begins recording commands.
    ///
    /// The command buffer must be in the initial state.
    pub fn begin(
        &mut self,
        usage: vk::CommandBufferUsageFlags,
    ) -> Result<(), vk::Result> {
        if self.state != CommandBufferState::Initial {
            return Err(vk::Result::ERROR_VALIDATION_FAILED_EXT);
        }

        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(usage);

        unsafe {
            self.device
                .handle()
                .begin_command_buffer(self.handle, &begin_info)?
        };

        self.one_time_submit =
            usage.contains(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.state = CommandBufferState::Recording;

        Ok(())
    }

    /// Ends command recording.
    ///
    /// The command buffer must currently be recording.
    pub fn end(&mut self) -> Result<(), vk::Result> {
        if self.state != CommandBufferState::Recording {
            return Err(vk::Result::ERROR_VALIDATION_FAILED_EXT);
        }

        unsafe {
            self.device
                .handle()
                .end_command_buffer(self.handle)?
        };

        self.state = CommandBufferState::Executable;

        Ok(())
    }

    /// Resets the command buffer so it can be recorded again.
    ///
    /// Recording or pending command buffers cannot be reset.
    pub fn reset(
        &mut self,
        flags: vk::CommandBufferResetFlags,
    ) -> Result<(), vk::Result> {
        match self.state {
            CommandBufferState::Recording
            | CommandBufferState::Pending
            | CommandBufferState::Invalid => {
                return Err(vk::Result::ERROR_VALIDATION_FAILED_EXT);
            }
            CommandBufferState::Initial
            | CommandBufferState::Executable => {}
        }

        unsafe {
            self.device
                .handle()
                .reset_command_buffer(self.handle, flags)?
        };

        self.state = CommandBufferState::Initial;
        self.one_time_submit = false;

        Ok(())
    }

    /// Submits the executable command buffer and waits for completion.
    ///
    /// A command buffer using ONE_TIME_SUBMIT becomes invalid after
    /// successful execution and must be reset before reuse.
    pub fn submit_and_wait(&mut self) -> Result<(), vk::Result> {
        if self.state != CommandBufferState::Executable {
            return Err(vk::Result::ERROR_VALIDATION_FAILED_EXT);
        }

        self.state = CommandBufferState::Pending;

        match self.device.submit_and_wait(self.handle) {
            Ok(()) => {
                self.state = if self.one_time_submit {
                    CommandBufferState::Invalid
                } else {
                    CommandBufferState::Executable
                };

                Ok(())
            }
            Err(error) => {
                // Completion is uncertain. Keep the buffer protected
                // until a successful device-idle operation is confirmed.
                Err(error)
            }
        }
    }
}

impl Drop for VulkanCommandBuffer<'_> {
    fn drop(&mut self) {
        if self.state == CommandBufferState::Pending {
            let wait_result = unsafe {
                self.device.handle().device_wait_idle()
            };

            if wait_result.is_err() {
                // Do not free a buffer that may still be in use.
                return;
            }
        }

        self.device.free_command_buffer(self.handle);
    }
}
