// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM framebuffer registration for native Vulkan DMA-BUF images.

use std::{error::Error, fmt};

use smithay::{
    backend::allocator::dmabuf::{Dmabuf, DmabufFlags},
    reexports::drm::control::{Device as DrmControlDevice, FbCmd2Flags, framebuffer::Handle},
};

use crate::drm::device::DrmDevice;

use super::VulkanDrmImage;

/// Errors returned while registering or retiring a Vulkan DRM framebuffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulkanDrmFramebufferError {
    InvalidDimensions,
    InvalidPlaneCount,
    PlaneValueOutOfRange { plane: usize },
    DmabufDescriptorClone { plane: usize, message: String },
    DmabufBuildFailed,
    RegistrationFailed(String),
    RetirementFailed(String),
}

impl fmt::Display for VulkanDrmFramebufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter.write_str("DRM framebuffer dimensions are invalid"),
            Self::InvalidPlaneCount => formatter.write_str("DRM framebuffer plane count is invalid"),
            Self::PlaneValueOutOfRange { plane } => write!(formatter, "DMA-BUF plane {plane} metadata exceeds DRM limits"),
            Self::DmabufDescriptorClone { plane, message } => write!(formatter, "Could not duplicate DMA-BUF descriptor for plane {plane}: {message}"),
            Self::DmabufBuildFailed => formatter.write_str("Could not construct Smithay DMA-BUF metadata"),
            Self::RegistrationFailed(message) => write!(formatter, "DRM framebuffer registration failed: {message}"),
            Self::RetirementFailed(message) => write!(formatter, "DRM framebuffer retirement failed: {message}"),
        }
    }
}

impl Error for VulkanDrmFramebufferError {}

/// A KMS framebuffer whose backing Vulkan image is kept alive for its lifetime.
///
/// The borrow prevents the Vulkan image and its exported DMA-BUF memory from
/// being destroyed while the kernel framebuffer may still reference it.
/// The caller must additionally wait for KMS release before retiring this
/// object; GPU completion alone is not sufficient.
pub struct VulkanDrmFramebuffer<'drm, 'image, 'vk> {
    device: &'drm DrmDevice,
    image: &'image VulkanDrmImage<'vk>,
    handle: Option<Handle>,
}

impl<'drm, 'image, 'vk> VulkanDrmFramebuffer<'drm, 'image, 'vk> {
    /// Registers the exact image FourCC/modifier and plane layout with KMS.
    pub fn register(
        device: &'drm DrmDevice,
        image: &'image VulkanDrmImage<'vk>,
    ) -> Result<Self, VulkanDrmFramebufferError> {
        if image.width() == 0 || image.height() == 0
            || i32::try_from(image.width()).is_err()
            || i32::try_from(image.height()).is_err()
        {
            return Err(VulkanDrmFramebufferError::InvalidDimensions);
        }
        if image.planes().is_empty()
            || image.planes().len() > 4
            || image.planes().len() != image.drm_format().memory_plane_count as usize
        {
            return Err(VulkanDrmFramebufferError::InvalidPlaneCount);
        }

        let mut dmabuf = Dmabuf::builder(
            (image.width() as i32, image.height() as i32),
            image.drm_format().code,
            image.drm_format().modifier,
            DmabufFlags::empty(),
        );

        for (index, plane) in image.planes().iter().enumerate() {
            let offset = u32::try_from(plane.offset)
                .map_err(|_| VulkanDrmFramebufferError::PlaneValueOutOfRange { plane: index })?;
            let stride = u32::try_from(plane.stride)
                .map_err(|_| VulkanDrmFramebufferError::PlaneValueOutOfRange { plane: index })?;
            let fd = image.dmabuf().try_clone().map_err(|error| {
                VulkanDrmFramebufferError::DmabufDescriptorClone {
                    plane: index,
                    message: error.to_string(),
                }
            })?;
            if !dmabuf.add_plane(fd, index as u32, offset, stride) {
                return Err(VulkanDrmFramebufferError::InvalidPlaneCount);
            }
        }

        let dmabuf = dmabuf.build().ok_or(VulkanDrmFramebufferError::DmabufBuildFailed)?;
        let raw_device = device.device();
        let handle = raw_device
            .add_planar_framebuffer(&dmabuf, FbCmd2Flags::MODIFIERS)
            .map_err(|error| VulkanDrmFramebufferError::RegistrationFailed(error.to_string()))?;

        Ok(Self {
            device,
            image,
            handle: Some(handle),
        })
    }

    /// Returns the DRM framebuffer handle while it remains registered.
    pub fn handle(&self) -> Handle {
        self.handle.expect("live framebuffer always has a handle")
    }

    /// Returns the backing Vulkan image.
    pub fn image(&self) -> &'image VulkanDrmImage<'vk> {
        self.image
    }

    /// Explicitly unregisters the framebuffer after KMS has released it.
    pub fn retire(mut self) -> Result<(), VulkanDrmFramebufferError> {
        let Some(handle) = self.handle.take() else {
            return Ok(());
        };
        self.device
            .device()
            .destroy_framebuffer(handle)
            .map_err(|error| VulkanDrmFramebufferError::RetirementFailed(error.to_string()))
    }
}

impl Drop for VulkanDrmFramebuffer<'_, '_, '_> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // Best effort only. Call retire() when the caller needs an
            // observable error and has already received KMS release.
            let _ = self.device.device().destroy_framebuffer(handle);
        }
    }
}
