// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Minimal non-indexed graphics draw pass for Vulkan dynamic rendering.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanGraphicsPipeline, VulkanImageState,
    VulkanImageTransition, VulkanImageView, VulkanTextureBinding,
};

/// Invalid draw requests rejected before commands are recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanDrawPassError {
    EmptyDraw,
    UndefinedFinalState,
    MissingColorAttachmentUsage,
    IncompatiblePipeline,
    NonFiniteClearColor,
    InvalidViewport,
    InvalidScissor,
    InvalidPushConstants,
}

impl fmt::Display for VulkanDrawPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDraw => {
                formatter.write_str("A Vulkan draw requires at least one vertex and instance")
            }
            Self::UndefinedFinalState => {
                formatter.write_str("A draw pass cannot leave its image undefined")
            }
            Self::MissingColorAttachmentUsage => {
                formatter.write_str("The Vulkan image lacks color-attachment usage")
            }
            Self::IncompatiblePipeline => {
                formatter.write_str("The graphics pipeline is incompatible with the attachment")
            }
            Self::NonFiniteClearColor => {
                formatter.write_str("Draw-pass clear color components must all be finite")
            }
            Self::InvalidViewport => formatter.write_str("The Vulkan surface viewport is invalid"),
            Self::InvalidScissor => {
                formatter.write_str("The Vulkan surface scissor is outside the attachment")
            }
            Self::InvalidPushConstants => formatter.write_str(
                "Vulkan push constants must be non-empty, four-byte aligned, and at most 128 bytes",
            ),
        }
    }
}

impl Error for VulkanDrawPassError {}

/// Records one graphics pipeline draw into a single color attachment.
pub struct VulkanDrawPass;

impl VulkanDrawPass {
    /// Validates draw dimensions and the requested final image state.
    pub fn validate(
        vertex_count: u32,
        instance_count: u32,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
    ) -> Result<(), VulkanDrawPassError> {
        if vertex_count == 0 || instance_count == 0 {
            return Err(VulkanDrawPassError::EmptyDraw);
        }
        if final_state == VulkanImageState::Undefined {
            return Err(VulkanDrawPassError::UndefinedFinalState);
        }
        if let VulkanColorLoad::Clear(color) = load {
            if !color.iter().all(|component| component.is_finite()) {
                return Err(VulkanDrawPassError::NonFiniteClearColor);
            }
        }
        Ok(())
    }

    /// Records a regular pipeline draw without descriptor sets.
    pub fn record(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        vertex_count: u32,
        instance_count: u32,
    ) -> Result<(), VulkanDrawPassError> {
        Self::record_inner(
            command_buffer,
            view,
            pipeline,
            previous_state,
            final_state,
            load,
            vertex_count,
            instance_count,
            None,
            None,
            None,
        )
    }

    /// Records a draw with one sampled texture descriptor set.
    ///
    /// The pipeline must have been created with the matching descriptor-set
    /// layout at `set_index`. The caller must keep the texture binding alive
    /// until the submitted command buffer has completed.
    pub fn record_textured(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        texture: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        vertex_count: u32,
        instance_count: u32,
    ) -> Result<(), VulkanDrawPassError> {
        Self::record_inner(
            command_buffer,
            view,
            pipeline,
            previous_state,
            final_state,
            load,
            vertex_count,
            instance_count,
            Some((texture, set_index)),
            None,
            Some(&0_u32.to_ne_bytes()),
        )
    }

    /// Records a textured surface using a destination viewport and clipped scissor.
    ///
    /// The viewport describes the complete surface destination and may extend
    /// beyond the output. The scissor must be the intersection with the output
    /// bounds, preserving correct texture coordinates for partially visible
    /// surfaces.
    #[allow(clippy::too_many_arguments)]
    pub fn record_textured_region(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        texture: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        texture_transform: u32,
    ) -> Result<(), VulkanDrawPassError> {
        Self::record_inner(
            command_buffer,
            view,
            pipeline,
            previous_state,
            final_state,
            load,
            3,
            1,
            Some((texture, set_index)),
            Some((viewport, scissor)),
            Some(&texture_transform.to_ne_bytes()),
        )
    }

    /// Records a textured region with caller-defined push-constant bytes.
    ///
    /// This supports dedicated pipelines whose shader interface differs from
    /// the four-byte surface texture transform.
    #[allow(clippy::too_many_arguments)]
    pub fn record_textured_region_with_push_constants(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        texture: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        push_constants: &[u8],
    ) -> Result<(), VulkanDrawPassError> {
        Self::record_inner(
            command_buffer,
            view,
            pipeline,
            previous_state,
            final_state,
            load,
            3,
            1,
            Some((texture, set_index)),
            Some((viewport, scissor)),
            Some(push_constants),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn record_inner(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        vertex_count: u32,
        instance_count: u32,
        texture: Option<(&VulkanTextureBinding<'_, '_, '_, '_>, u32)>,
        region: Option<(vk::Viewport, vk::Rect2D)>,
        push_constants: Option<&[u8]>,
    ) -> Result<(), VulkanDrawPassError> {
        Self::validate(vertex_count, instance_count, final_state, load)?;
        if let Some(bytes) = push_constants {
            Self::validate_push_constants(bytes)?;
        }
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDrawPassError::MissingColorAttachmentUsage);
        }
        if !pipeline.is_compatible_with(view.device(), view.format()) {
            return Err(VulkanDrawPassError::IncompatiblePipeline);
        }
        if let Some((viewport, scissor)) = region {
            Self::validate_region(view.width(), view.height(), viewport, scissor)?;
        }

        VulkanImageTransition::record_color(
            view.device(),
            command_buffer,
            view.image_handle(),
            previous_state,
            VulkanImageState::ColorAttachment,
        );

        let scope = VulkanDynamicRenderingScope::begin_color(command_buffer, view, load);
        pipeline.bind(command_buffer);
        if let Some((binding, set_index)) = texture {
            binding.bind(command_buffer, pipeline.layout(), set_index);
            if let Some(data) = push_constants {
                pipeline.push_constants(command_buffer, vk::ShaderStageFlags::VERTEX, 0, data);
            }
        }
        if let Some((viewport, scissor)) = region {
            pipeline.set_viewport_and_scissor_region(command_buffer, viewport, scissor);
        } else {
            pipeline.set_viewport_and_scissor(command_buffer, view.width(), view.height());
        }
        pipeline.draw(command_buffer, vertex_count, instance_count);
        scope.end();

        VulkanImageTransition::record_color(
            view.device(),
            command_buffer,
            view.image_handle(),
            VulkanImageState::ColorAttachment,
            final_state,
        );

        Ok(())
    }

    fn validate_push_constants(bytes: &[u8]) -> Result<(), VulkanDrawPassError> {
        if bytes.is_empty() || bytes.len() > 128 || bytes.len() % 4 != 0 {
            return Err(VulkanDrawPassError::InvalidPushConstants);
        }
        Ok(())
    }

    fn validate_region(
        target_width: u32,
        target_height: u32,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) -> Result<(), VulkanDrawPassError> {
        let viewport_values = [
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
            viewport.min_depth,
            viewport.max_depth,
        ];
        if !viewport_values.iter().all(|value| value.is_finite())
            || viewport.width <= 0.0
            || viewport.height <= 0.0
            || viewport.min_depth < 0.0
            || viewport.max_depth > 1.0
            || viewport.min_depth > viewport.max_depth
        {
            return Err(VulkanDrawPassError::InvalidViewport);
        }

        let x = u32::try_from(scissor.offset.x).map_err(|_| VulkanDrawPassError::InvalidScissor)?;
        let y = u32::try_from(scissor.offset.y).map_err(|_| VulkanDrawPassError::InvalidScissor)?;
        let right = x
            .checked_add(scissor.extent.width)
            .ok_or(VulkanDrawPassError::InvalidScissor)?;
        let bottom = y
            .checked_add(scissor.extent.height)
            .ok_or(VulkanDrawPassError::InvalidScissor)?;

        if scissor.extent.width == 0
            || scissor.extent.height == 0
            || right > target_width
            || bottom > target_height
        {
            return Err(VulkanDrawPassError::InvalidScissor);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> vk::Viewport {
        vk::Viewport {
            x: -12.0,
            y: 4.0,
            width: 32.0,
            height: 24.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    fn scissor() -> vk::Rect2D {
        vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 4 },
            extent: vk::Extent2D {
                width: 20,
                height: 24,
            },
        }
    }

    #[test]
    fn accepts_four_float_wallpaper_push_constants() {
        assert_eq!(VulkanDrawPass::validate_push_constants(&[0; 16]), Ok(()));
    }

    #[test]
    fn rejects_empty_or_unaligned_push_constants() {
        assert_eq!(
            VulkanDrawPass::validate_push_constants(&[]),
            Err(VulkanDrawPassError::InvalidPushConstants)
        );
        assert_eq!(
            VulkanDrawPass::validate_push_constants(&[0; 6]),
            Err(VulkanDrawPassError::InvalidPushConstants)
        );
    }

    #[test]
    fn rejects_push_constants_larger_than_vulkan_minimum_limit() {
        assert_eq!(
            VulkanDrawPass::validate_push_constants(&[0; 132]),
            Err(VulkanDrawPassError::InvalidPushConstants)
        );
    }

    #[test]
    fn accepts_negative_viewport_origin_with_output_clipped_scissor() {
        assert_eq!(
            VulkanDrawPass::validate_region(1920, 1080, viewport(), scissor()),
            Ok(())
        );
    }

    #[test]
    fn rejects_scissor_outside_attachment_bounds() {
        let invalid = vk::Rect2D {
            offset: vk::Offset2D { x: 1900, y: 0 },
            extent: vk::Extent2D {
                width: 40,
                height: 10,
            },
        };

        assert_eq!(
            VulkanDrawPass::validate_region(1920, 1080, viewport(), invalid),
            Err(VulkanDrawPassError::InvalidScissor)
        );
    }

    #[test]
    fn rejects_non_finite_viewport_coordinates() {
        let invalid = vk::Viewport {
            x: f32::NAN,
            ..viewport()
        };

        assert_eq!(
            VulkanDrawPass::validate_region(1920, 1080, invalid, scissor()),
            Err(VulkanDrawPassError::InvalidViewport)
        );
    }
}
