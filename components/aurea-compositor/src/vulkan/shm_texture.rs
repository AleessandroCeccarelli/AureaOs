// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Persistent Vulkan texture storage for owned Wayland SHM snapshots.

use std::{error::Error, fmt, sync::Arc};

use smithay::reexports::ash::vk;

use crate::render::shm_buffer::{ShmBufferSnapshot, ShmPixelFormat};

use super::{
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanImageViewError,
    VulkanLogicalDevice, VulkanPendingUpload, VulkanSharedImage, VulkanSharedImageError,
    VulkanUploadBatch, VulkanUploadBuffer, VulkanUploadError,
};

/// Errors returned while importing a Wayland SHM snapshot as a Vulkan texture.
#[derive(Debug)]
pub enum VulkanShmTextureError {
    UnsupportedPixelFormat,
    Image(VulkanSharedImageError),
    ImageView(VulkanImageViewError),
    Upload(VulkanUploadError),
    Command(vk::Result),
}

impl fmt::Display for VulkanShmTextureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPixelFormat => formatter
                .write_str("Wayland SHM snapshot must be normalized to RGBA8 before Vulkan upload"),
            Self::Image(error) => write!(formatter, "SHM texture image allocation failed: {error}"),
            Self::ImageView(error) => write!(formatter, "SHM texture image-view creation failed: {error}"),
            Self::Upload(error) => write!(formatter, "SHM texture upload failed: {error}"),
            Self::Command(error) => write!(formatter, "SHM texture Vulkan command failed: {error:?}"),
        }
    }
}

impl Error for VulkanShmTextureError {}

/// Owns a sampled Vulkan image containing one uploaded Wayland SHM snapshot.
pub struct VulkanShmTexture {
    image: Arc<VulkanSharedImage>,
    view: VulkanImageView<'static, 'static>,
}

impl VulkanShmTexture {
    /// Converts and uploads one owned SHM snapshot into a sampled RGBA8 image.
    pub fn from_snapshot(
        device: Arc<VulkanLogicalDevice>,
        snapshot: ShmBufferSnapshot,
    ) -> Result<Self, VulkanShmTextureError> {
        let mut textures = Self::submit_snapshots(device, vec![snapshot])?
            .wait()
            .map_err(VulkanShmTextureError::Command)?;
        let texture = textures
            .pop()
            .ok_or(VulkanShmTextureError::Command(vk::Result::ERROR_UNKNOWN))?;
        Arc::try_unwrap(texture)
            .map_err(|_| VulkanShmTextureError::Command(vk::Result::ERROR_UNKNOWN))
    }

    /// Starts a batch upload and returns while the GPU may still be processing it.
    pub(crate) fn submit_snapshots(
        device: Arc<VulkanLogicalDevice>,
        snapshots: Vec<ShmBufferSnapshot>,
    ) -> Result<VulkanPendingUpload<Vec<Arc<Self>>>, VulkanShmTextureError> {
        if snapshots.is_empty() {
            return Err(VulkanShmTextureError::Command(
                vk::Result::ERROR_VALIDATION_FAILED_EXT,
            ));
        }

        let mut textures = Vec::with_capacity(snapshots.len());
        let mut segments = Vec::with_capacity(snapshots.len());
        let mut total_bytes = 0usize;

        for snapshot in snapshots {
            let snapshot = snapshot.into_rgba8();
            if snapshot.format() != ShmPixelFormat::Rgba8888 {
                return Err(VulkanShmTextureError::UnsupportedPixelFormat);
            }

            let byte_len =
                VulkanUploadBuffer::required_byte_len(snapshot.width(), snapshot.height())
                    .map_err(VulkanShmTextureError::Upload)?;
            if snapshot.pixels().len() != byte_len {
                return Err(VulkanShmTextureError::Upload(
                    VulkanUploadError::InvalidByteLength,
                ));
            }

            let offset = total_bytes;
            total_bytes = total_bytes
                .checked_add(byte_len)
                .ok_or(VulkanShmTextureError::Upload(
                    VulkanUploadError::SizeOverflow,
                ))?;

            let image = Arc::new(
                VulkanSharedImage::allocate(
                    Arc::clone(&device),
                    snapshot.width(),
                    snapshot.height(),
                    vk::Format::R8G8B8A8_UNORM,
                    vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                )
                .map_err(VulkanShmTextureError::Image)?,
            );
            let view = VulkanImageView::color_shared_owned(Arc::clone(&image))
                .map_err(VulkanShmTextureError::ImageView)?;
            textures.push(Arc::new(Self { image, view }));
            segments.push((offset, snapshot.pixels().to_vec()));
        }

        let staging = VulkanUploadBatch::new(Arc::clone(&device), total_bytes)
            .map_err(VulkanShmTextureError::Upload)?;
        let segment_refs = segments
            .iter()
            .map(|(offset, pixels)| (*offset, pixels.as_slice()))
            .collect::<Vec<_>>();
        staging
            .write_segments(&segment_refs)
            .map_err(VulkanShmTextureError::Upload)?;

        let pending = VulkanPendingUpload::new(Arc::clone(&device), staging, textures)
            .map_err(VulkanShmTextureError::Command)?;
        pending.staging().record_host_write_barrier(pending.command_buffer());

        for (texture, (offset, _)) in pending.resources().iter().zip(segments.iter()) {
            VulkanImageTransition::record_color(
                device.as_ref(),
                pending.command_buffer(),
                texture.image.handle(),
                VulkanImageState::Undefined,
                VulkanImageState::TransferDestination,
            );
            pending
                .staging()
                .record_upload_rgba8(pending.command_buffer(), &texture.view, *offset)
                .map_err(VulkanShmTextureError::Upload)?;
            VulkanImageTransition::record_color(
                device.as_ref(),
                pending.command_buffer(),
                texture.image.handle(),
                VulkanImageState::TransferDestination,
                VulkanImageState::ShaderRead,
            );
        }

        pending
            .submit()
            .map_err(VulkanShmTextureError::Command)
    }

    /// Returns the cached image view suitable for a sampled-image descriptor.
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
