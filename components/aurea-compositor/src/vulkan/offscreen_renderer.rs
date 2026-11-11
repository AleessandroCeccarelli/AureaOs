// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Synchronous offscreen Vulkan clear-and-readback execution path.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanClearPass, VulkanClearPassError, VulkanCommandBuffer, VulkanImageAllocator,
    VulkanImageError, VulkanImageRequest, VulkanImageState, VulkanImageView,
    VulkanImageViewError, VulkanLogicalDevice, VulkanReadbackBuffer, VulkanReadbackError,
};

/// Errors returned while executing an offscreen RGBA8 frame.
#[derive(Debug)]
pub enum VulkanOffscreenError {
    InvalidDimensions,
    Image(VulkanImageError),
    ImageView(VulkanImageViewError),
    Readback(VulkanReadbackError),
    ClearPass(VulkanClearPassError),
    Command(vk::Result),
}

impl fmt::Display for VulkanOffscreenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => formatter.write_str("Offscreen frame dimensions must be non-zero"),
            Self::Image(error) => write!(formatter, "Offscreen image error: {error}"),
            Self::ImageView(error) => write!(formatter, "Offscreen image-view error: {error}"),
            Self::Readback(error) => write!(formatter, "Offscreen readback error: {error}"),
            Self::ClearPass(error) => write!(formatter, "Offscreen clear-pass error: {error}"),
            Self::Command(error) => write!(formatter, "Offscreen Vulkan command failed: {error:?}"),
        }
    }
}

impl Error for VulkanOffscreenError {}

/// Executes one complete offscreen color-clear frame and reads its pixels back.
///
/// This is a synchronous diagnostic path, not the interactive presentation
/// scheduler. It allocates an RGBA8 render target and host-visible staging
/// buffer, records transitions/clear/copy, submits and waits, then returns
/// tightly packed RGBA bytes. It is suitable for validating the Vulkan core
/// on a headless device such as Mesa lavapipe.
pub struct VulkanOffscreenRenderer<'device> {
    device: &'device VulkanLogicalDevice,
}

impl<'device> VulkanOffscreenRenderer<'device> {
    pub fn new(device: &'device VulkanLogicalDevice) -> Self {
        Self { device }
    }

    /// Clears an RGBA8 image and returns the completed pixel data.
    pub fn render_clear(
        &self,
        width: u32,
        height: u32,
        clear_color: [f32; 4],
    ) -> Result<Vec<u8>, VulkanOffscreenError> {
        if width == 0 || height == 0 {
            return Err(VulkanOffscreenError::InvalidDimensions);
        }

        let allocator = VulkanImageAllocator::new(self.device);
        let image = allocator.allocate(VulkanImageRequest::render_target(
            width,
            height,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        )).map_err(VulkanOffscreenError::Image)?;
        let view = VulkanImageView::color(&image)
            .map_err(VulkanOffscreenError::ImageView)?;
        let readback = VulkanReadbackBuffer::new(self.device, width, height)
            .map_err(VulkanOffscreenError::Readback)?;

        let mut commands = VulkanCommandBuffer::new(self.device)
            .map_err(VulkanOffscreenError::Command)?;
        commands.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .map_err(VulkanOffscreenError::Command)?;

        VulkanClearPass::record_color(
            commands.handle(),
            &view,
            VulkanImageState::Undefined,
            VulkanImageState::TransferSource,
            clear_color,
        ).map_err(VulkanOffscreenError::ClearPass)?;

        readback.record_copy_rgba8(commands.handle(), &view)
            .map_err(VulkanOffscreenError::Readback)?;

        commands.end().map_err(VulkanOffscreenError::Command)?;
        commands.submit_and_wait().map_err(VulkanOffscreenError::Command)?;

        readback.read_rgba8().map_err(VulkanOffscreenError::Readback)
    }
}
