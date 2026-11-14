// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan texture upload for owned Wayland SHM snapshots.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use crate::render::shm_buffer::{ShmBufferSnapshot, ShmPixelFormat};

use super::{
    VulkanCommandBuffer, VulkanImage, VulkanImageAllocator, VulkanImageError, VulkanImageRequest,
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanImageViewError,
    VulkanLogicalDevice, VulkanUploadBuffer, VulkanUploadError,
};

/// Errors returned while importing a Wayland SHM snapshot as a Vulkan texture.
#[derive(Debug)]
pub enum VulkanShmTextureError {
    UnsupportedPixelFormat,
    Image(VulkanImageError),
    ImageView(VulkanImageViewError),
    Upload(VulkanUploadError),
    Command(vk::Result),
}

impl fmt::Display for VulkanShmTextureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPixelFormat => formatter.write_str("Wayland SHM snapshot must be normalized to RGBA8 before Vulkan upload"),
            Self::Image(error) => write!(formatter, "SHM texture image allocation failed: {error}"),
            Self::ImageView(error) => write!(formatter, "SHM texture image-view creation failed: {error}"),
            Self::Upload(error) => write!(formatter, "SHM texture upload failed: {error}"),
            Self::Command(error) => write!(formatter, "SHM texture Vulkan command failed: {error:?}"),
        }
    }
}

impl Error for VulkanShmTextureError {}

/// Owns a sampled Vulkan image containing one uploaded Wayland SHM snapshot.
///
/// Upload is synchronous: when construction returns, the image is in
/// SHADER_READ_ONLY_OPTIMAL and the temporary staging resources are released.
pub struct VulkanShmTexture<'device> {
    image: VulkanImage<'device>,
}

impl<'device> VulkanShmTexture<'device> {
    /// Converts and uploads an owned SHM snapshot into a sampled RGBA8 image.
    pub fn from_snapshot(
        device: &'device VulkanLogicalDevice,
        snapshot: ShmBufferSnapshot,
    ) -> Result<Self, VulkanShmTextureError> {
        let snapshot = snapshot.into_rgba8();
        if snapshot.format() != ShmPixelFormat::Rgba8888 {
            return Err(VulkanShmTextureError::UnsupportedPixelFormat);
        }

        let image = VulkanImageAllocator::new(device)
            .allocate(VulkanImageRequest::render_target(
                snapshot.width(),
                snapshot.height(),
                vk::Format::R8G8B8A8_UNORM,
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            ))
            .map_err(VulkanShmTextureError::Image)?;
        let view = VulkanImageView::color(&image).map_err(VulkanShmTextureError::ImageView)?;
        let staging = VulkanUploadBuffer::new(device, snapshot.width(), snapshot.height())
            .map_err(VulkanShmTextureError::Upload)?;
        staging
            .write_rgba8(snapshot.pixels())
            .map_err(VulkanShmTextureError::Upload)?;

        let mut commands =
            VulkanCommandBuffer::new(device).map_err(VulkanShmTextureError::Command)?;
        commands
            .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .map_err(VulkanShmTextureError::Command)?;

        VulkanImageTransition::record_color(
            device,
            commands.handle(),
            image.handle(),
            VulkanImageState::Undefined,
            VulkanImageState::TransferDestination,
        );
        staging
            .record_upload_rgba8(commands.handle(), &view)
            .map_err(VulkanShmTextureError::Upload)?;
        VulkanImageTransition::record_color(
            device,
            commands.handle(),
            image.handle(),
            VulkanImageState::TransferDestination,
            VulkanImageState::ShaderRead,
        );

        commands.end().map_err(VulkanShmTextureError::Command)?;
        commands
            .submit_and_wait()
            .map_err(VulkanShmTextureError::Command)?;

        drop(view);
        drop(staging);

        Ok(Self { image })
    }

    /// Returns the sampled image. Its current layout is SHADER_READ_ONLY_OPTIMAL.
    pub fn image(&self) -> &VulkanImage<'device> {
        &self.image
    }

    /// Creates an image view suitable for a future descriptor-set binding.
    pub fn create_view(&self) -> Result<VulkanImageView<'_, 'device>, VulkanImageViewError> {
        VulkanImageView::color(&self.image)
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
}
