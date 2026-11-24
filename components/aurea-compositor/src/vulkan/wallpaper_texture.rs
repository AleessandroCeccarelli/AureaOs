// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent Vulkan texture storage for decoded desktop wallpaper pixels.

use std::{error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use crate::desktop::WallpaperPixels;

use super::{
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanImageViewError,
    VulkanLogicalDevice, VulkanPendingUpload, VulkanSharedImage, VulkanSharedImageError,
    VulkanUploadBatch, VulkanUploadBuffer, VulkanUploadError,
};

/// Errors returned while uploading decoded wallpaper pixels to Vulkan.
#[derive(Debug)]
pub enum VulkanWallpaperTextureError {
    Image(VulkanSharedImageError),
    ImageView(VulkanImageViewError),
    Upload(VulkanUploadError),
    Command(vk::Result),
}

impl fmt::Display for VulkanWallpaperTextureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(error) => write!(formatter, "Wallpaper image allocation failed: {error}"),
            Self::ImageView(error) => {
                write!(formatter, "Wallpaper image-view creation failed: {error}")
            }
            Self::Upload(error) => write!(formatter, "Wallpaper pixel upload failed: {error}"),
            Self::Command(error) => {
                write!(formatter, "Wallpaper Vulkan command failed: {error:?}")
            }
        }
    }
}

impl Error for VulkanWallpaperTextureError {}

/// Owns a sampled Vulkan image containing one decoded RGBA8 wallpaper.
pub struct VulkanWallpaperTexture {
    image: Arc<VulkanSharedImage>,
    view: VulkanImageView<'static, 'static>,
}

impl VulkanWallpaperTexture {
    /// Uploads validated pixels and waits for the one-time transfer to complete.
    ///
    /// Prefer `submit_pixels` when preparing resources for the compositor.
    /// This blocking convenience method must not run on the render callback.
    pub fn from_pixels(
        device: Arc<VulkanLogicalDevice>,
        pixels: WallpaperPixels,
    ) -> Result<Self, VulkanWallpaperTextureError> {
        let texture = Self::submit_pixels(device, pixels)?
            .wait()
            .map_err(VulkanWallpaperTextureError::Command)?;
        Arc::try_unwrap(texture).map_err(|_| {
            VulkanWallpaperTextureError::Command(vk::Result::ERROR_UNKNOWN)
        })
    }

    /// Starts a wallpaper upload and returns without waiting for GPU completion.
    ///
    /// The returned fence-backed upload retains the staging allocation and
    /// texture until completion. A resource cache should poll it outside the
    /// draw submission and only expose the texture after `wait` succeeds.
    pub(crate) fn submit_pixels(
        device: Arc<VulkanLogicalDevice>,
        pixels: WallpaperPixels,
    ) -> Result<VulkanPendingUpload<Arc<Self>>, VulkanWallpaperTextureError> {
        let width = pixels.width();
        let height = pixels.height();
        let rgba8 = pixels.into_rgba8();

        let expected_len = VulkanUploadBuffer::required_byte_len(width, height)
            .map_err(VulkanWallpaperTextureError::Upload)?;
        if rgba8.len() != expected_len {
            return Err(VulkanWallpaperTextureError::Upload(
                VulkanUploadError::InvalidByteLength,
            ));
        }

        let image = Arc::new(
            VulkanSharedImage::allocate(
                Arc::clone(&device),
                width,
                height,
                vk::Format::R8G8B8A8_UNORM,
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            )
            .map_err(VulkanWallpaperTextureError::Image)?,
        );
        let view = VulkanImageView::color_shared_owned(Arc::clone(&image))
            .map_err(VulkanWallpaperTextureError::ImageView)?;
        let texture = Arc::new(Self { image, view });

        let staging = VulkanUploadBatch::new(Arc::clone(&device), expected_len)
            .map_err(VulkanWallpaperTextureError::Upload)?;
        staging
            .write_segments(&[(0, rgba8.as_slice())])
            .map_err(VulkanWallpaperTextureError::Upload)?;

        let pending = VulkanPendingUpload::new(Arc::clone(&device), staging, texture)
            .map_err(VulkanWallpaperTextureError::Command)?;
        pending
            .staging()
            .record_host_write_barrier(pending.command_buffer());

        let texture = pending.resources();
        VulkanImageTransition::record_color(
            device.as_ref(),
            pending.command_buffer(),
            texture.image.handle(),
            VulkanImageState::Undefined,
            VulkanImageState::TransferDestination,
        );
        pending
            .staging()
            .record_upload_rgba8(pending.command_buffer(), &texture.view, 0)
            .map_err(VulkanWallpaperTextureError::Upload)?;
        VulkanImageTransition::record_color(
            device.as_ref(),
            pending.command_buffer(),
            texture.image.handle(),
            VulkanImageState::TransferDestination,
            VulkanImageState::ShaderRead,
        );

        pending
            .submit()
            .map_err(VulkanWallpaperTextureError::Command)
    }

    /// Returns the sampled-image view in SHADER_READ_ONLY_OPTIMAL layout.
    pub fn view(&self) -> &VulkanImageView<'static, 'static> {
        &self.view
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
}
