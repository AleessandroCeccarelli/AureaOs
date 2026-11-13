// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns Vulkan scanout images and their imported KMS framebuffer wrappers.

use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use crate::drm::{
    device::DrmDevice,
    frame_coordinator::{DrmOutputFrameCoordinator, DrmOutputFrameError},
    gbm::GbmDeviceManager,
    presentation::DrmFramebufferId,
};

use super::{
    DrmVulkanFormat, VulkanCommandBuffer, VulkanDrmFramebuffer, VulkanDrmFramebufferError,
    VulkanDrmImage, VulkanDrmImageAllocator, VulkanDrmImageError, VulkanImageState,
    VulkanImageTransition, VulkanImageTransitionError, VulkanLogicalDevice,
};

/// Errors produced while creating or retiring an owned scanout frame.
#[derive(Debug)]
pub enum VulkanDrmFramePoolError {
    Image(VulkanDrmImageError),
    Framebuffer(VulkanDrmFramebufferError),
    Lifecycle(DrmOutputFrameError),
    DuplicateFramebuffer(DrmFramebufferId),
    UnknownFramebuffer(DrmFramebufferId),
    FramebufferStillInUse(DrmFramebufferId),
    CommandBuffer(vk::Result),
    ImageTransition(VulkanImageTransitionError),
    ForeignQueueFamilyUnavailable,
}

impl fmt::Display for VulkanDrmFramePoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(error) => {
                write!(formatter, "Vulkan scanout image allocation failed: {error}")
            }
            Self::Framebuffer(error) => {
                write!(formatter, "KMS framebuffer registration failed: {error}")
            }
            Self::Lifecycle(error) => write!(
                formatter,
                "Scanout lifecycle rejected the operation: {error}"
            ),
            Self::DuplicateFramebuffer(id) => {
                write!(formatter, "Framebuffer {} is already pooled", id.value())
            }
            Self::UnknownFramebuffer(id) => {
                write!(formatter, "Framebuffer {} is not pooled", id.value())
            }
            Self::FramebufferStillInUse(id) => write!(
                formatter,
                "Framebuffer {} is still owned by GPU or KMS",
                id.value()
            ),
            Self::CommandBuffer(error) => {
                write!(formatter, "Vulkan acquire command failed: {error:?}")
            }
            Self::ForeignQueueFamilyUnavailable => formatter.write_str(
                "Vulkan foreign queue-family ownership is unavailable for DRM scanout",
            ),
            Self::ImageTransition(error) => {
                write!(
                    formatter,
                    "Vulkan foreign-ownership acquire failed: {error}"
                )
            }
        }
    }
}

impl Error for VulkanDrmFramePoolError {}

impl From<VulkanDrmImageError> for VulkanDrmFramePoolError {
    fn from(error: VulkanDrmImageError) -> Self {
        Self::Image(error)
    }
}

impl From<VulkanDrmFramebufferError> for VulkanDrmFramePoolError {
    fn from(error: VulkanDrmFramebufferError) -> Self {
        Self::Framebuffer(error)
    }
}

impl From<DrmOutputFrameError> for VulkanDrmFramePoolError {
    fn from(error: DrmOutputFrameError) -> Self {
        Self::Lifecycle(error)
    }
}

/// One allocation whose Vulkan image and GBM framebuffer share a lifetime.
pub struct VulkanDrmScanoutFrame {
    // Drop the imported KMS framebuffer before releasing its Vulkan image.
    framebuffer: VulkanDrmFramebuffer,
    image: VulkanDrmImage,
}

impl VulkanDrmScanoutFrame {
    pub fn allocate(
        logical_device: Arc<VulkanLogicalDevice>,
        drm_device: &DrmDevice,
        gbm_devices: &GbmDeviceManager,
        width: u32,
        height: u32,
        format: DrmVulkanFormat,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self, VulkanDrmFramePoolError> {
        if !logical_device.supports_foreign_queue_family() {
            return Err(VulkanDrmFramePoolError::ForeignQueueFamilyUnavailable);
        }
        let allocator = VulkanDrmImageAllocator::new(logical_device)?;
        let image = if format.modifier == smithay::backend::allocator::Modifier::Linear {
            allocator.allocate_linear_exportable(width, height, format, usage)?
        } else {
            allocator.allocate_exportable(width, height, format, usage)?
        };
        let framebuffer = VulkanDrmFramebuffer::register(drm_device, gbm_devices, &image)?;
        Ok(Self { framebuffer, image })
    }

    pub fn framebuffer_id(&self) -> DrmFramebufferId {
        DrmFramebufferId::new(u64::from(u32::from(self.framebuffer.handle())))
    }

    pub fn framebuffer_handle(&self) -> smithay::reexports::drm::control::framebuffer::Handle {
        self.framebuffer.handle()
    }

    fn acquire_from_foreign(&self) -> Result<(), VulkanDrmFramePoolError> {
        let device = self.image.device();
        let mut command_buffer =
            VulkanCommandBuffer::new(device).map_err(VulkanDrmFramePoolError::CommandBuffer)?;

        command_buffer
            .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .map_err(VulkanDrmFramePoolError::CommandBuffer)?;

        VulkanImageTransition::record_acquire_from_foreign(
            device,
            command_buffer.handle(),
            self.image.image(),
            VulkanImageState::General,
        )
        .map_err(VulkanDrmFramePoolError::ImageTransition)?;

        command_buffer
            .end()
            .map_err(VulkanDrmFramePoolError::CommandBuffer)?;
        command_buffer
            .submit_and_wait()
            .map_err(VulkanDrmFramePoolError::CommandBuffer)?;

        Ok(())
    }

    pub fn image(&self) -> &VulkanDrmImage {
        &self.image
    }
}

/// Retains each image/framebuffer pair until GPU and KMS ownership are released.
#[derive(Default)]
pub struct VulkanDrmFramePool {
    frames: HashMap<DrmFramebufferId, VulkanDrmScanoutFrame>,
}

impl VulkanDrmFramePool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        frame: VulkanDrmScanoutFrame,
        coordinator: &mut DrmOutputFrameCoordinator,
    ) -> Result<DrmFramebufferId, VulkanDrmFramePoolError> {
        let id = frame.framebuffer_id();
        if self.frames.contains_key(&id) {
            return Err(VulkanDrmFramePoolError::DuplicateFramebuffer(id));
        }
        coordinator.register_framebuffer(id)?;
        self.frames.insert(id, frame);
        Ok(id)
    }

    pub fn get(&self, id: DrmFramebufferId) -> Option<&VulkanDrmScanoutFrame> {
        self.frames.get(&id)
    }

    /// Acquires a frame back from foreign ownership after KMS release or a skipped presentation.
    ///
    /// The acquire barrier is submitted and waited synchronously before the
    /// lifecycle record is allowed to become reusable.
    pub fn acquire_after_foreign_release(
        &self,
        id: DrmFramebufferId,
        coordinator: &mut DrmOutputFrameCoordinator,
    ) -> Result<(), VulkanDrmFramePoolError> {
        let frame = self
            .frames
            .get(&id)
            .ok_or(VulkanDrmFramePoolError::UnknownFramebuffer(id))?;

        coordinator.mark_foreign_acquire_submitted(id)?;
        frame.acquire_from_foreign()?;
        coordinator.mark_foreign_acquired(id)?;

        Ok(())
    }

    /// Retires a frame only after the lifecycle coordinator marks it reusable.
    pub fn retire(
        &mut self,
        id: DrmFramebufferId,
        coordinator: &mut DrmOutputFrameCoordinator,
    ) -> Result<VulkanDrmScanoutFrame, VulkanDrmFramePoolError> {
        if !self.frames.contains_key(&id) {
            return Err(VulkanDrmFramePoolError::UnknownFramebuffer(id));
        }
        if !coordinator.is_available(id)? {
            return Err(VulkanDrmFramePoolError::FramebufferStillInUse(id));
        }
        coordinator.unregister_framebuffer(id)?;
        self.frames
            .remove(&id)
            .ok_or(VulkanDrmFramePoolError::UnknownFramebuffer(id))
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}
