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
    DrmVulkanFormat, VulkanDrmFramebuffer, VulkanDrmFramebufferError, VulkanDrmImage,
    VulkanDrmImageAllocator, VulkanDrmImageError, VulkanLogicalDevice,
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
        drm_device: &'drm DrmDevice,
        gbm_devices: &GbmDeviceManager,
        width: u32,
        height: u32,
        format: DrmVulkanFormat,
        usage: vk::ImageUsageFlags,
     ) -> Result<Self, VulkanDrmFramePoolError> {
        let allocator = VulkanDrmImageAllocator::new(logical_device)?;
        let image = allocator.allocate_exportable(width, height, format, usage)?;
        let framebuffer = VulkanDrmFramebuffer::register(drm_device, gbm_devices, &image)?;
        Ok(Self { framebuffer, image })
    }

    pub fn framebuffer_id(&self) -> DrmFramebufferId {
        DrmFramebufferId::new(u64::from(u32::from(self.framebuffer.handle())))
    }

    pub fn framebuffer_handle(&self) -> smithay::reexports::drm::control::framebuffer::Handle {
        self.framebuffer.handle()
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
