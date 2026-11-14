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
}

impl fmt::Display for VulkanDrawPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDraw => formatter.write_str("A Vulkan draw requires at least one vertex and instance"),
            Self::UndefinedFinalState => formatter.write_str("A draw pass cannot leave its image undefined"),
            Self::MissingColorAttachmentUsage => formatter.write_str("The Vulkan image lacks color-attachment usage"),
            Self::IncompatiblePipeline => formatter.write_str("The graphics pipeline is incompatible with the attachment"),
            Self::NonFiniteClearColor => formatter.write_str("Draw-pass clear color components must all be finite"),
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
            command_buffer, view, pipeline, previous_state, final_state, load,
            vertex_count, instance_count, None,
        )
    }

    /// Records a draw with one sampled texture descriptor set.
    ///
    /// The pipeline must have been created with the matching descriptor-set
    /// layout at `set_index`. The texture binding keeps its image and sampler
    /// alive for the duration of this recording call.
    pub fn record_textured(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        texture: &VulkanTextureBinding<'_, '_, '_>,
        set_index: u32,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        vertex_count: u32,
        instance_count: u32,
    ) -> Result<(), VulkanDrawPassError> {
        Self::record_inner(
            command_buffer, view, pipeline, previous_state, final_state, load,
            vertex_count, instance_count, Some((texture, set_index)),
        )
    }

    fn record_inner(
        command_buffer: vk::CommandBuffer,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        vertex_count: u32,
        instance_count: u32,
        texture: Option<(&VulkanTextureBinding<'_, '_, '_>, u32)>,
    ) -> Result<(), VulkanDrawPassError> {
        Self::validate(vertex_count, instance_count, final_state, load)?;
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDrawPassError::MissingColorAttachmentUsage);
        }
        if !pipeline.is_compatible_with(view.device(), view.format()) {
            return Err(VulkanDrawPassError::IncompatiblePipeline);
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
        }
        pipeline.set_viewport_and_scissor(command_buffer, view.width(), view.height());
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
}
