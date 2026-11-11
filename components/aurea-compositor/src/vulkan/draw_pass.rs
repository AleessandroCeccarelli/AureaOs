// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Minimal non-indexed graphics draw pass for Vulkan dynamic rendering.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanGraphicsPipeline,
    VulkanImageState, VulkanImageTransition, VulkanImageView,
};

/// Invalid draw requests rejected before commands are recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulkanDrawPassError {
    EmptyDraw,
    UndefinedFinalState,
    MissingColorAttachmentUsage,
    IncompatiblePipeline,
}

impl fmt::Display for VulkanDrawPassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDraw => formatter.write_str("A Vulkan draw requires at least one vertex and instance"),
            Self::UndefinedFinalState => formatter.write_str("A draw pass cannot leave its image undefined"),
            Self::MissingColorAttachmentUsage => formatter.write_str("The Vulkan image lacks color-attachment usage"),
            Self::IncompatiblePipeline => formatter.write_str("The graphics pipeline is incompatible with the attachment"),
        }
    }
}

impl Error for VulkanDrawPassError {}

/// Records one graphics pipeline draw into a single color attachment.
///
/// The pass owns no GPU resources and does not submit work. The caller supplies
/// the actual previous image state and remains responsible for command-buffer
/// submission and resource lifetime.
pub struct VulkanDrawPass;

impl VulkanDrawPass {
    /// Validates draw dimensions and the requested final image state.
    pub fn validate(
        vertex_count: u32,
        instance_count: u32,
        final_state: VulkanImageState,
    ) -> Result<(), VulkanDrawPassError> {
        if vertex_count == 0 || instance_count == 0 {
            return Err(VulkanDrawPassError::EmptyDraw);
        }
        if final_state == VulkanImageState::Undefined {
            return Err(VulkanDrawPassError::UndefinedFinalState);
        }
        Ok(())
    }

    /// Records transitions, dynamic rendering, pipeline binding and draw.
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
        Self::validate(vertex_count, instance_count, final_state)?;
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

        let scope = VulkanDynamicRenderingScope::begin_color(
            command_buffer,
            view,
            load,
        );
        pipeline.bind(command_buffer);
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
