// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Owns Vulkan scanout images and their imported KMS framebuffer wrappers.

use std::{cell::Cell, collections::HashMap, error::Error, fmt, sync::Arc};

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
    VulkanImageTransition, VulkanImageTransitionError, VulkanLogicalDevice, VulkanSharedImage,
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
            Self::ForeignQueueFamilyUnavailable => formatter
                .write_str("Vulkan foreign queue-family ownership is unavailable for DRM scanout"),
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
    intermediate: Option<VulkanSharedImage>,
    intermediate_state: Cell<VulkanImageState>,
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
        let allocator = VulkanDrmImageAllocator::new(Arc::clone(&logical_device))?;
        let image = if format.modifier == smithay::backend::allocator::Modifier::Linear {
            allocator.allocate_linear_exportable(width, height, format, usage)?
        } else {
            allocator.allocate_exportable(width, height, format, usage)?
        };
        let framebuffer = VulkanDrmFramebuffer::register(drm_device, gbm_devices, &image)?;
        Ok(Self {
            framebuffer,
            image,
            intermediate: None,
            intermediate_state: Cell::new(VulkanImageState::Undefined),
        })
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

    /// Returns the optimal-tiled composition image when this format supports
    /// both color attachment writes and linearly sampled reads.
    pub fn intermediate_image(&self) -> Option<&VulkanSharedImage> {
        self.intermediate.as_ref()
    }

    /// Allocates the optional composition image only when an output effect
    /// actually needs it. Ordinary frames therefore retain no extra full-size
    /// GPU image, and unsupported formats keep the direct scanout path.
    pub(crate) fn ensure_intermediate(&mut self, device: Arc<VulkanLogicalDevice>) -> bool {
        if self.intermediate.is_some() {
            return true;
        }

        self.intermediate = Self::allocate_intermediate(
            device,
            self.image.width(),
            self.image.height(),
            self.image.format(),
        );
        if self.intermediate.is_some() {
            self.intermediate_state.set(VulkanImageState::Undefined);
            true
        } else {
            false
        }
    }

    /// Returns the tracked layout/access state of the reusable intermediate.
    pub fn intermediate_state(&self) -> Option<VulkanImageState> {
        self.intermediate.as_ref()?;
        Some(self.intermediate_state.get())
    }

    /// Updates the state only after the command buffer has completed.
    pub(crate) fn set_intermediate_state(&self, state: VulkanImageState) {
        if self.intermediate.is_some() {
            self.intermediate_state.set(state);
        }
    }

    fn allocate_intermediate(
        device: Arc<VulkanLogicalDevice>,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Option<VulkanSharedImage> {
        let properties = unsafe {
            device
                .instance()
                .get_physical_device_format_properties(device.physical_device(), format)
        };
        let required = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if !properties.optimal_tiling_features.contains(required) {
            return None;
        }

        VulkanSharedImage::allocate(
            device,
            width,
            height,
            format,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        )
        .ok()
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

    /// Maximum number of scanout images retained for reuse by one output.
    ///
    /// Two images are sufficient for ordinary page-flip rotation. The third
    /// slot allows one additional allocation during a mode/format transition.
    pub const MAX_RETAINED_FRAMES: usize = 3;

    /// Finds a released image matching the current output size and negotiated formats.
    pub fn find_reusable(
        &self,
        width: u32,
        height: u32,
        formats: &[DrmVulkanFormat],
        coordinator: &DrmOutputFrameCoordinator,
    ) -> Result<Option<DrmFramebufferId>, VulkanDrmFramePoolError> {
        for (id, frame) in &self.frames {
            let image = frame.image();
            if image.width() != width || image.height() != height {
                continue;
            }
            if !formats.iter().any(|format| image.drm_format() == *format) {
                continue;
            }
            if coordinator.is_available(*id)? {
                return Ok(Some(*id));
            }
        }
        Ok(None)
    }

    /// Retires reusable frames from obsolete modes and trims excess idle allocations.
    /// Busy frames remain owned until the lifecycle coordinator confirms release.
    pub fn prune(
        &mut self,
        width: u32,
        height: u32,
        formats: &[DrmVulkanFormat],
        coordinator: &mut DrmOutputFrameCoordinator,
    ) -> Result<usize, VulkanDrmFramePoolError> {
        let mut removable = Vec::new();
        for (id, frame) in &self.frames {
            let image = frame.image();
            let obsolete = image.width() != width
                || image.height() != height
                || !formats.iter().any(|format| image.drm_format() == *format);
            if coordinator.is_available(*id)?
                && (obsolete || self.frames.len() - removable.len() > Self::MAX_RETAINED_FRAMES)
            {
                removable.push(*id);
            }
        }
        for id in &removable {
            self.retire(*id, coordinator)?;
        }
        Ok(removable.len())
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

    pub(crate) fn get_mut(&mut self, id: DrmFramebufferId) -> Option<&mut VulkanDrmScanoutFrame> {
        self.frames.get_mut(&id)
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

    /// Reacquires a released image and retains it for the next frame when the pool is small.
    ///
    /// Images beyond the bounded pool size are retired only after Vulkan has
    /// completed the foreign-ownership acquire and the lifecycle confirms reuse.
    pub fn reacquire_and_recycle(
        &mut self,
        id: DrmFramebufferId,
        coordinator: &mut DrmOutputFrameCoordinator,
    ) -> Result<bool, VulkanDrmFramePoolError> {
        self.acquire_after_foreign_release(id, coordinator)?;
        if self.frames.len() > Self::MAX_RETAINED_FRAMES {
            self.retire(id, coordinator)?;
            Ok(false)
        } else {
            Ok(true)
        }
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
