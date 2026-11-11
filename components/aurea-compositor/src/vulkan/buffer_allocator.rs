// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan render-target allocation and DMA-BUF export through Smithay.

use std::{error::Error, fmt};

use smithay::backend::{
    allocator::{
        dmabuf::{AsDmabuf, Dmabuf},
        vulkan::{Error as SmithayVulkanError, ImageUsageFlags, VulkanAllocator, VulkanImage},
    },
    drm::DrmNode,
    vulkan::PhysicalDevice,
};

use super::DrmVulkanFormat;

/// Validated request for a Vulkan render-target buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanRenderTargetRequest {
    width: u32,
    height: u32,
    format: DrmVulkanFormat,
}

impl VulkanRenderTargetRequest {
    /// Creates a render-target request with non-zero dimensions.
    pub fn new(
        width: u32,
        height: u32,
        format: DrmVulkanFormat,
    ) -> Result<Self, VulkanBufferError> {
        if width == 0 || height == 0 {
            return Err(VulkanBufferError::InvalidDimensions { width, height });
        }

        Ok(Self {
            width,
            height,
            format,
        })
    }

    /// Requested buffer width in pixels.
    pub fn width(self) -> u32 {
        self.width
    }

    /// Requested buffer height in pixels.
    pub fn height(self) -> u32 {
        self.height
    }

    /// Exact DRM format/modifier pair selected by compatibility negotiation.
    pub fn format(self) -> DrmVulkanFormat {
        self.format
    }
}

/// Errors returned by Vulkan render-target allocation or DMA-BUF export.
#[derive(Debug)]
pub enum VulkanBufferError {
    /// A render-target dimension was zero.
    InvalidDimensions { width: u32, height: u32 },
    /// Smithay could not initialize its Vulkan allocator or allocate an image.
    Allocation(SmithayVulkanError),
    /// The physical device could not report its DRM identity.
    DeviceIdentityQuery(String),
    /// The selected Vulkan physical device does not match the requested DRM node.
    DeviceNodeMismatch { requested: DrmNode },
    /// Smithay could not export the allocated image as a DMA-BUF.
    Export(smithay::backend::allocator::vulkan::ExportError),
}

impl fmt::Display for VulkanBufferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(
                    formatter,
                    "Invalid Vulkan buffer dimensions: {width}x{height}"
                )
            }
            Self::Allocation(error) => {
                write!(formatter, "Vulkan buffer allocation failed: {error}")
            }
            Self::DeviceIdentityQuery(message) => write!(
                formatter,
                "Failed to query Vulkan device DRM identity: {message}"
            ),
            Self::DeviceNodeMismatch { requested } => write!(
                formatter,
                "Vulkan physical device does not match DRM node {requested:?}"
            ),
            Self::Export(error) => write!(formatter, "Vulkan DMA-BUF export failed: {error}"),
        }
    }
}

impl Error for VulkanBufferError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDimensions { .. } => None,
            Self::Allocation(error) => Some(error),
            Self::DeviceIdentityQuery(_) | Self::DeviceNodeMismatch { .. } => None,
            Self::Export(error) => Some(error),
        }
    }
}

/// Owns Smithay's Vulkan allocator for one selected physical device.
///
/// Construct this with the same physical device selected for the DRM node
/// whose primary-plane capabilities were used during format negotiation.
/// The allocator owns its Vulkan logical device and exports images as DMA-BUFs.
pub struct VulkanBufferAllocator {
    allocator: VulkanAllocator,
}

impl VulkanBufferAllocator {
    /// Creates an allocator only when the Vulkan physical device belongs to the DRM node.
    ///
    /// The match is established through VK_EXT_physical_device_drm, checking
    /// both the primary and render node reported by the physical device.
    pub fn new_for_drm_node(
        device: &PhysicalDevice,
        requested_node: DrmNode,
    ) -> Result<Self, VulkanBufferError> {
        let primary_node = device
            .primary_node()
            .map_err(|error| VulkanBufferError::DeviceIdentityQuery(error.to_string()))?;
        let render_node = device
            .render_node()
            .map_err(|error| VulkanBufferError::DeviceIdentityQuery(error.to_string()))?;

        if primary_node.as_ref() != Some(&requested_node)
            && render_node.as_ref() != Some(&requested_node)
        {
            return Err(VulkanBufferError::DeviceNodeMismatch {
                requested: requested_node,
            });
        }

        Self::new(device)
    }

    fn new(device: &PhysicalDevice) -> Result<Self, VulkanBufferError> {
        let allocator = VulkanAllocator::new(device, ImageUsageFlags::COLOR_ATTACHMENT)
            .map_err(VulkanBufferError::Allocation)?;

        Ok(Self { allocator })
    }

    /// Allocates a render-target image using the negotiated exact format/modifier pair.
    ///
    /// Smithay validates the requested image usage and modifier support again
    /// for the actual dimensions before creating the Vulkan image.
    pub fn allocate_render_target(
        &mut self,
        request: VulkanRenderTargetRequest,
    ) -> Result<VulkanImage, VulkanBufferError> {
        self.allocator
            .create_buffer_with_usage(
                request.width,
                request.height,
                request.format.code,
                &[request.format.modifier],
                ImageUsageFlags::COLOR_ATTACHMENT,
            )
            .map_err(VulkanBufferError::Allocation)
    }

    /// Exports an allocated image and its per-plane layout as a DMA-BUF.
    pub fn export_dmabuf(image: &VulkanImage) -> Result<Dmabuf, VulkanBufferError> {
        image.export().map_err(VulkanBufferError::Export)
    }

    /// Returns whether Smithay can allocate the specified format for color rendering.
    pub fn supports_render_target(&self, format: DrmVulkanFormat) -> bool {
        self.allocator.is_format_supported(
            smithay::backend::allocator::Format {
                code: format.code,
                modifier: format.modifier,
            },
            ImageUsageFlags::COLOR_ATTACHMENT,
        )
    }
}
