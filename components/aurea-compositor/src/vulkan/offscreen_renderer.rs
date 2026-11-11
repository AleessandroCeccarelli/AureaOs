// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Synchronous offscreen Vulkan clear-and-readback execution path.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanClearPass, VulkanClearPassError, VulkanCommandBuffer, VulkanDrawPass,
    VulkanDrawPassError, VulkanGraphicsPipeline, VulkanGraphicsPipelineError,
    VulkanGraphicsShaders, VulkanImageAllocator, VulkanImageError, VulkanImageRequest,
    VulkanImageState, VulkanImageView, VulkanImageViewError, VulkanLogicalDevice,
    VulkanReadbackBuffer, VulkanReadbackError, VulkanShaderError, VulkanShaderModule,
};

/// Errors returned while executing an offscreen RGBA8 frame.
#[derive(Debug)]
pub enum VulkanOffscreenError {
    InvalidDimensions,
    Image(VulkanImageError),
    ImageView(VulkanImageViewError),
    Readback(VulkanReadbackError),
    ClearPass(VulkanClearPassError),
    DrawPass(VulkanDrawPassError),
    Shader(VulkanShaderError),
    Pipeline(VulkanGraphicsPipelineError),
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
            Self::DrawPass(error) => write!(formatter, "Offscreen draw-pass error: {error}"),
            Self::Shader(error) => write!(formatter, "Offscreen shader error: {error}"),
            Self::Pipeline(error) => write!(formatter, "Offscreen pipeline error: {error}"),
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
    pub fn validate_dimensions(width: u32, height: u32) -> Result<(), VulkanOffscreenError> {
        if width == 0 || height == 0 {
            return Err(VulkanOffscreenError::InvalidDimensions);
        }
        Ok(())
    }

    pub fn render_clear(
        &self,
        width: u32,
        height: u32,
        clear_color: [f32; 4],
    ) -> Result<Vec<u8>, VulkanOffscreenError> {
        Self::validate_dimensions(width, height)?;

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

    /// Draws a shader-driven RGB triangle and returns completed RGBA8 pixels.
    ///
    /// Shader inputs must contain little-endian SPIR-V compiled for Vulkan 1.3.
    /// The method is synchronous and intended for renderer bring-up and GPU
    /// diagnostics, not the interactive presentation path.
    pub fn render_triangle(
        &self,
        width: u32,
        height: u32,
        vertex_spirv: &[u8],
        fragment_spirv: &[u8],
    ) -> Result<Vec<u8>, VulkanOffscreenError> {
        Self::validate_dimensions(width, height)?;

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

        let vertex = VulkanShaderModule::from_bytes(self.device, vertex_spirv)
            .map_err(VulkanOffscreenError::Shader)?;
        let fragment = VulkanShaderModule::from_bytes(self.device, fragment_spirv)
            .map_err(VulkanOffscreenError::Shader)?;
        let pipeline = VulkanGraphicsPipeline::new(
            self.device,
            VulkanGraphicsShaders { vertex: &vertex, fragment: &fragment },
            vk::Format::R8G8B8A8_UNORM,
        ).map_err(VulkanOffscreenError::Pipeline)?;

        let mut commands = VulkanCommandBuffer::new(self.device)
            .map_err(VulkanOffscreenError::Command)?;
        commands.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .map_err(VulkanOffscreenError::Command)?;

        VulkanDrawPass::record(
            commands.handle(),
            &view,
            &pipeline,
            VulkanImageState::Undefined,
            VulkanImageState::TransferSource,
            3,
            1,
        ).map_err(VulkanOffscreenError::DrawPass)?;

        readback.record_copy_rgba8(commands.handle(), &view)
            .map_err(VulkanOffscreenError::Readback)?;

        commands.end().map_err(VulkanOffscreenError::Command)?;
        commands.submit_and_wait().map_err(VulkanOffscreenError::Command)?;

        readback.read_rgba8().map_err(VulkanOffscreenError::Readback)
    }
}
