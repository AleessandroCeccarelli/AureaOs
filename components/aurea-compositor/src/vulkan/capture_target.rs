// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Dedicated optimal-tiled image storage for compositor capture frames.

use std::{cell::Cell, error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use super::{
    VulkanImageState, VulkanImageView, VulkanImageViewError, VulkanLogicalDevice,
    VulkanSharedImage, VulkanSharedImageError,
};

/// Pixel format used by the initial CPU readback path.
pub const VULKAN_CAPTURE_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;

/// Errors returned while allocating a capture-only render target.
#[derive(Debug)]
pub enum VulkanCaptureTargetError {
    InvalidDimensions,
    DeviceLost,
    UnsupportedFormat(vk::Format),
    Image(VulkanSharedImageError),
    InvalidPoolLimits,
    PoolExhausted,
    PoolBusy,
    PoolLockPoisoned,
}

impl fmt::Display for VulkanCaptureTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceLost => {
                formatter.write_str("Vulkan logical device is unusable after device loss")
            }
            Self::InvalidDimensions => {
                formatter.write_str("Capture target dimensions must be non-zero")
            }
            Self::UnsupportedFormat(format) => write!(
                formatter,
                "Vulkan format {format:?} does not support capture color-attachment and transfer-source usage"
            ),
            Self::Image(error) => {
                write!(formatter, "Capture target image allocation failed: {error}")
            }
            Self::InvalidPoolLimits => {
                formatter.write_str("Capture target pool limits must be non-zero")
            }
            Self::PoolExhausted => {
                formatter.write_str("No capture target is available within pool limits")
            }
            Self::PoolBusy => formatter.write_str("Capture target pool is busy"),
            Self::PoolLockPoisoned => formatter.write_str("Capture target pool lock is poisoned"),
        }
    }
}

impl Error for VulkanCaptureTargetError {}

impl From<VulkanSharedImageError> for VulkanCaptureTargetError {
    fn from(error: VulkanSharedImageError) -> Self {
        Self::Image(error)
    }
}

/// Owns a capture-only image independently of every DRM/KMS scanout image.
///
/// The image is optimal-tiled and deliberately not exported as a DMA-BUF.
/// Its state is managed by the compositor that records writes and transitions.
pub struct VulkanCaptureTarget {
    image: Arc<VulkanSharedImage>,
    state: Cell<VulkanImageState>,
}

impl VulkanCaptureTarget {
    /// Allocates an RGBA8 target after checking the exact Vulkan format features.
    pub fn allocate(
        device: Arc<VulkanLogicalDevice>,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanCaptureTargetError> {
        if width == 0 || height == 0 {
            return Err(VulkanCaptureTargetError::InvalidDimensions);
        }

        let properties = unsafe {
            device.instance().get_physical_device_format_properties(
                device.physical_device(),
                VULKAN_CAPTURE_FORMAT,
            )
        };
        let required =
            vk::FormatFeatureFlags::COLOR_ATTACHMENT | vk::FormatFeatureFlags::TRANSFER_SRC;
        if !properties.optimal_tiling_features.contains(required) {
            return Err(VulkanCaptureTargetError::UnsupportedFormat(
                VULKAN_CAPTURE_FORMAT,
            ));
        }

        let image = VulkanSharedImage::allocate(
            device,
            width,
            height,
            VULKAN_CAPTURE_FORMAT,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        )?;

        Ok(Self {
            image: Arc::new(image),
            state: Cell::new(VulkanImageState::Undefined),
        })
    }

    /// Creates an independently owned view suitable for asynchronous readback.
    pub fn create_owned_view(
        &self,
    ) -> Result<VulkanImageView<'static, 'static>, VulkanImageViewError> {
        VulkanImageView::color_shared_owned(Arc::clone(&self.image))
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    /// Actual Vulkan memory requirement, including implementation alignment.
    pub fn allocation_size(&self) -> usize {
        self.image.allocation_size()
    }

    /// Returns the last state committed by a completed compositor submission.
    pub fn state(&self) -> VulkanImageState {
        self.state.get()
    }

    /// Updates state only after the submission recording that transition completes.
    pub fn set_state_after_completion(&self, state: VulkanImageState) {
        self.state.set(state);
    }
}
