// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Frame-scoped Vulkan 1.0 recording helpers.
//!
//! Render passes and pipelines are persistent cache resources. Framebuffers,
//! image views, and descriptor bindings remain borrowed only for the duration
//! of each recording operation and are never retained by this module.

use std::sync::Arc;

use smithay::reexports::ash::vk;

use super::{
    VulkanClearPass, VulkanClearPassError, VulkanImageState, VulkanImageView,
    VulkanLegacyRenderPass, VulkanLegacyRenderPassError, VulkanSharedLegacyGraphicsPipeline,
    VulkanTextureBinding, VulkanDrawPass, VulkanDrawPassError,
};

/// Stateless recorder for operations that form one legacy-rendered frame.
///
/// The caller owns command-buffer lifecycle, submission, image-state tracking,
/// and all sampled resources until GPU completion. Each call creates and
/// destroys its framebuffer locally, preventing a framebuffer from escaping
/// the lifetime of the image view or render pass it references.
pub(crate) struct VulkanLegacyFrameRecorder;

impl VulkanLegacyFrameRecorder {
    /// Records the frame's initial clear into a legacy color attachment.
    pub(crate) fn clear(
        command_buffer: vk::CommandBuffer,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
        view: &VulkanImageView<'_, 'static>,
        previous_state: VulkanImageState,
        next_state: VulkanImageState,
        color: [f32; 4],
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        let target = render_pass
            .create_target(view)
            .map_err(VulkanLegacyFrameRecorderError::RenderPass)?;
        VulkanClearPass::record_legacy_color(
            command_buffer,
            &target,
            previous_state,
            next_state,
            color,
        )
        .map_err(VulkanLegacyFrameRecorderError::Clear)
    }

    /// Records one textured layer using a traditional render pass.
    ///
    /// This is intentionally one layer per call: it keeps framebuffer lifetime
    /// minimal and permits the compositor to interleave surfaces, menu artwork,
    /// cursor surfaces, and cursor-trail impressions in their established order.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn textured_region(
        command_buffer: vk::CommandBuffer,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
        view: &VulkanImageView<'_, 'static>,
        pipeline: &VulkanSharedLegacyGraphicsPipeline,
        source: &VulkanImageView<'_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        next_target_state: VulkanImageState,
        load: super::VulkanColorLoad,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        push_constants: &[u8],
        push_constant_stages: vk::ShaderStageFlags,
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        let target = render_pass
            .create_target(view)
            .map_err(VulkanLegacyFrameRecorderError::RenderPass)?;
        VulkanDrawPass::record_shared_legacy_textured_region(
            command_buffer,
            &target,
            pipeline,
            source,
            binding,
            set_index,
            previous_source_state,
            previous_target_state,
            next_target_state,
            load,
            viewport,
            scissor,
            push_constants,
            push_constant_stages,
        )
        .map_err(VulkanLegacyFrameRecorderError::Draw)
    }
}

/// Errors returned by the frame-scoped legacy recording helpers.
#[derive(Debug)]
pub(crate) enum VulkanLegacyFrameRecorderError {
    RenderPass(VulkanLegacyRenderPassError),
    Clear(VulkanClearPassError),
    Draw(VulkanDrawPassError),
}

impl std::fmt::Display for VulkanLegacyFrameRecorderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RenderPass(error) => write!(formatter, "Legacy frame target: {error}"),
            Self::Clear(error) => write!(formatter, "Legacy frame clear: {error}"),
            Self::Draw(error) => write!(formatter, "Legacy frame draw: {error}"),
        }
    }
}

impl std::error::Error for VulkanLegacyFrameRecorderError {}
