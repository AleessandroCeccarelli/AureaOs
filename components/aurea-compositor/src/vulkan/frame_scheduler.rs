// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded frames-in-flight scheduling for the AureaOS Vulkan renderer.

use std::{error::Error, fmt, os::fd::OwnedFd};

use smithay::reexports::ash::vk;

use super::{VulkanLogicalDevice, frame_slot::VulkanFrameSlot};

/// Default number of frames that may be submitted without blocking the CPU.
pub const DEFAULT_FRAMES_IN_FLIGHT: usize = 2;

/// Errors returned while creating or using the Vulkan frame scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanFrameSchedulerError {
    InvalidFrameCount,
    SyncFdUnavailable,
    Vulkan(vk::Result),
}

impl fmt::Display for VulkanFrameSchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrameCount => {
                formatter.write_str("Vulkan frames-in-flight count must be between one and three")
            }
            Self::SyncFdUnavailable => formatter
                .write_str("The selected Vulkan device cannot export Linux sync-file fences"),
            Self::Vulkan(error) => write!(
                formatter,
                "Vulkan frame scheduling operation failed: {error:?}"
            ),
        }
    }
}

impl Error for VulkanFrameSchedulerError {}

/// Owns a bounded ring of independently synchronized frame slots.
///
/// Each slot has its own command pool and completion fence. Acquiring a slot
/// waits only for that slot's previous submission, not for the whole device.
pub struct VulkanFrameScheduler<'device> {
    device: &'device VulkanLogicalDevice,
    slots: Vec<VulkanFrameSlot>,
    next_slot: usize,
}

impl<'device> VulkanFrameScheduler<'device> {
    /// Creates a scheduler with two frames in flight by default.
    pub fn new(device: &'device VulkanLogicalDevice) -> Result<Self, VulkanFrameSchedulerError> {
        Self::with_frame_count(device, DEFAULT_FRAMES_IN_FLIGHT)
    }

    /// Creates a scheduler with a bounded number of independent frame slots.
    pub fn with_frame_count(
        device: &'device VulkanLogicalDevice,
        frame_count: usize,
    ) -> Result<Self, VulkanFrameSchedulerError> {
        if !(1..=3).contains(&frame_count) {
            return Err(VulkanFrameSchedulerError::InvalidFrameCount);
        }

        let mut slots = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            match VulkanFrameSlot::new(device, device.graphics_queue_family()) {
                Ok(slot) => slots.push(slot),
                Err(error) => {
                    for slot in slots.drain(..) {
                        unsafe {
                            device.handle().destroy_fence(slot.completion_fence, None);
                            if let Some(semaphore) = slot.sync_fd_semaphore {
                                device.handle().destroy_semaphore(semaphore, None);
                            }
                            device
                                .handle()
                                .destroy_command_pool(slot.command_pool, None);
                        }
                    }
                    return Err(VulkanFrameSchedulerError::Vulkan(error));
                }
            }
        }

        Ok(Self {
            device,
            slots,
            next_slot: 0,
        })
    }

    /// Acquires the next slot and begins a one-time primary command buffer.
    ///
    /// The call waits for the selected slot only. Other submitted frames may
    /// continue executing on the GPU.
    pub fn begin_frame(&mut self) -> Result<VulkanFrame<'_, 'device>, VulkanFrameSchedulerError> {
        let slot_index = self.next_slot;
        let command_buffer = self.slots[slot_index]
            .prepare(self.device)
            .map_err(VulkanFrameSchedulerError::Vulkan)?;

        Ok(VulkanFrame {
            scheduler: self,
            slot_index,
            command_buffer,
        })
    }

    /// Number of independently synchronized frame slots.
    pub fn frame_count(&self) -> usize {
        self.slots.len()
    }

    /// Computes the next slot in a bounded frame ring.
    ///
    /// The current index is normalized first, keeping the result in range
    /// even when callers provide an out-of-range index.
    pub fn next_slot_index(current: usize, slot_count: usize) -> usize {
        if slot_count == 0 {
            return 0;
        }

        (current % slot_count + 1) % slot_count
    }
}

impl Drop for VulkanFrameScheduler<'_> {
    fn drop(&mut self) {
        unsafe {
            // Destroying pools or fences while the device may still be using
            // them is invalid. If the idle wait fails, intentionally leak the
            // Vulkan handles rather than risk freeing in-flight resources.
            if self.device.handle().device_wait_idle().is_err() {
                return;
            }

            for slot in &mut self.slots {
                if slot.completion_fence != vk::Fence::null() {
                    self.device
                        .handle()
                        .destroy_fence(slot.completion_fence, None);
                }
                if let Some(semaphore) = slot.sync_fd_semaphore.take() {
                    self.device.handle().destroy_semaphore(semaphore, None);
                }
                self.device
                    .handle()
                    .destroy_command_pool(slot.command_pool, None);
            }
        }
    }
}

/// A recording frame that can be submitted exactly once.
pub struct VulkanFrame<'scheduler, 'device> {
    scheduler: &'scheduler mut VulkanFrameScheduler<'device>,
    slot_index: usize,
    command_buffer: vk::CommandBuffer,
}

impl VulkanFrame<'_, '_> {
    /// Raw command buffer to which the renderer records its drawing commands.
    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    /// Ends recording and submits this frame without waiting for completion.
    ///
    /// The slot's fence is signaled when the GPU has finished using its
    /// resources. The scheduler waits for it only when this slot is reused.
    pub fn submit(self) -> Result<(), VulkanFrameSchedulerError> {
        let scheduler = self.scheduler;
        scheduler.slots[self.slot_index]
            .submit(scheduler.device, self.command_buffer)
            .map_err(VulkanFrameSchedulerError::Vulkan)?;

        scheduler.next_slot =
            VulkanFrameScheduler::next_slot_index(self.slot_index, scheduler.slots.len());
        Ok(())
    }

    /// Submits this frame and exports its completion semaphore as a Linux
    /// sync-file FD for DRM's plane IN_FENCE_FD property.
    ///
    /// Keep the returned descriptor open through the atomic TEST_ONLY and
    /// commit calls. DRM imports its own sync-file reference during each ioctl.
    pub fn submit_with_sync_fd(self) -> Result<OwnedFd, VulkanFrameSchedulerError> {
        let scheduler = self.scheduler;
        if !scheduler.device.supports_sync_fd() {
            return Err(VulkanFrameSchedulerError::SyncFdUnavailable);
        }

        let sync_fd = scheduler.slots[self.slot_index]
            .submit_with_sync_fd(scheduler.device, self.command_buffer)
            .map_err(VulkanFrameSchedulerError::Vulkan)?;

        scheduler.next_slot =
            VulkanFrameScheduler::next_slot_index(self.slot_index, scheduler.slots.len());
        Ok(sync_fd)
    }
}
