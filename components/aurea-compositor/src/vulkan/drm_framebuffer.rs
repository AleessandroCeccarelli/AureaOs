// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM framebuffer registration for native Vulkan DMA-BUF images.

use std::{error::Error, fmt};

use smithay::{
    backend::{
        allocator::dmabuf::{Dmabuf, DmabufFlags, MAX_PLANES},
        drm::{
            DrmNode,
            gbm::{GbmFramebuffer, framebuffer_from_dmabuf},
        },
    },
    reexports::drm::control::framebuffer::Handle,
};

use crate::drm::{device::DrmDevice, gbm::GbmDeviceManager};

use super::VulkanDrmImage;

/// Errors returned while registering a Vulkan DMA-BUF as a KMS framebuffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulkanDrmFramebufferError {
    InvalidDimensions,
    InvalidPlaneCount,
    PlaneValueOutOfRange { plane: usize },
    ZeroPlaneStride { plane: usize },
    PlaneLayoutOverflow { plane: usize },
    PlaneLayoutOutOfBounds { plane: usize },
    DmabufDescriptorClone { plane: usize, message: String },
    DmabufBuildFailed,
    DrmNodeUnavailable(String),
    GbmDeviceUnavailable(DrmNode),
    ImportOrRegistrationFailed(String),
}

impl fmt::Display for VulkanDrmFramebufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => {
                formatter.write_str("DRM framebuffer dimensions are invalid")
            }
            Self::InvalidPlaneCount => formatter.write_str(
                "DRM framebuffer plane count does not match the negotiated Vulkan modifier",
            ),
            Self::PlaneValueOutOfRange { plane } => write!(
                formatter,
                "DMA-BUF plane {plane} metadata exceeds DRM limits"
            ),
            Self::ZeroPlaneStride { plane } => {
                write!(formatter, "DMA-BUF plane {plane} has a zero row stride")
            }
            Self::PlaneLayoutOverflow { plane } => {
                write!(formatter, "DMA-BUF plane {plane} offset and size overflow")
            }
            Self::PlaneLayoutOutOfBounds { plane } => write!(
                formatter,
                "DMA-BUF plane {plane} extends beyond the exported Vulkan allocation"
            ),
            Self::DmabufDescriptorClone { plane, message } => write!(
                formatter,
                "Could not duplicate DMA-BUF descriptor for plane {plane}: {message}"
            ),
            Self::DmabufBuildFailed => {
                formatter.write_str("Could not construct Smithay DMA-BUF metadata")
            }
            Self::DrmNodeUnavailable(message) => {
                write!(formatter, "Could not identify DRM device: {message}")
            }
            Self::GbmDeviceUnavailable(node) => write!(
                formatter,
                "No GBM import device is registered for DRM node {node}"
            ),
            Self::ImportOrRegistrationFailed(message) => write!(
                formatter,
                "GBM DMA-BUF import or DRM framebuffer registration failed: {message}"
            ),
        }
    }
}

impl Error for VulkanDrmFramebufferError {}

/// Owns Smithay's GBM framebuffer wrapper and borrows the backing Vulkan image.
///
/// Smithay imports the DMA-BUF into the GBM device associated with this DRM
/// node, obtains the driver's GEM handles and registers the framebuffer.
/// The GBM wrapper owns framebuffer retirement. Keep this object alive until
/// KMS has released the framebuffer; GPU completion alone is not sufficient.
pub struct VulkanDrmFramebuffer<'drm, 'image, 'vk> {
    image: &'image VulkanDrmImage<'vk>,
    framebuffer: GbmFramebuffer,
    node: DrmNode,
    _device: &'drm DrmDevice,
}

impl<'drm, 'image, 'vk> VulkanDrmFramebuffer<'drm, 'image, 'vk> {
    /// Imports the image into the GBM device for the exact DRM node and
    /// registers a modifier-aware framebuffer. Legacy fallback is disabled.
    pub fn register(
        device: &'drm DrmDevice,
        gbm_devices: &GbmDeviceManager,
        image: &'image VulkanDrmImage<'vk>,
    ) -> Result<Self, VulkanDrmFramebufferError> {
        if image.width() == 0
            || image.height() == 0
            || i32::try_from(image.width()).is_err()
            || i32::try_from(image.height()).is_err()
        {
            return Err(VulkanDrmFramebufferError::InvalidDimensions);
        }

        if image.planes().is_empty()
            || image.planes().len() > MAX_PLANES
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
            if stride == 0 {
                return Err(VulkanDrmFramebufferError::ZeroPlaneStride { plane: index });
            }
            let plane_end = u64::from(offset)
                .checked_add(plane.size)
                .ok_or(VulkanDrmFramebufferError::PlaneLayoutOverflow { plane: index })?;
            if plane.size == 0 || plane_end > image.allocation_size() {
                return Err(VulkanDrmFramebufferError::PlaneLayoutOutOfBounds { plane: index });
            }
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

        let dmabuf = dmabuf
            .build()
            .ok_or(VulkanDrmFramebufferError::DmabufBuildFailed)?;
        let node = device
            .node()
            .map_err(|error| VulkanDrmFramebufferError::DrmNodeUnavailable(error.to_string()))?;

        // Preserve the exact import target so Smithay/GBM cannot select a
        // different DRM device when resolving the DMA-BUF.
        dmabuf.set_node(node.clone());

        let gbm = gbm_devices
            .get(node.clone())
            .ok_or_else(|| VulkanDrmFramebufferError::GbmDeviceUnavailable(node.clone()))?;

        let framebuffer = framebuffer_from_dmabuf(device.fd(), gbm, &dmabuf, false, false)
            .map_err(|error| {
                VulkanDrmFramebufferError::ImportOrRegistrationFailed(error.to_string())
            })?;

        Ok(Self {
            image,
            framebuffer,
            node,
            _device: device,
        })
    }

    /// Returns the kernel framebuffer identifier.
    pub fn handle(&self) -> Handle {
        *self.framebuffer.as_ref()
    }

    /// Returns the DRM node that owns this framebuffer.
    pub fn node(&self) -> DrmNode {
        self.node
    }

    /// Returns the backing Vulkan image.
    pub fn image(&self) -> &'image VulkanDrmImage<'vk> {
        self.image
    }

    /// Explicitly retires the framebuffer.
    ///
    /// Call only after the KMS page-flip/release event confirms that no
    /// active plane references this framebuffer.
    pub fn retire(self) {
        drop(self);
    }
}
