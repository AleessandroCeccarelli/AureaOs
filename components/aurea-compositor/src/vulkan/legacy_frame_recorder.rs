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
    VulkanSharedLegacyDisplayColorPass, VulkanDisplayColorPassError,
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
        view: &VulkanImageView<'_, '_>,
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

    /// Records a regular surface or cursor texture using the legacy surface ABI.
    ///
    /// The transform is a four-byte vertex-stage push constant, matching the
    /// normal surface shader. This named entry point avoids callers rebuilding
    /// the byte representation or accidentally selecting the wrong stage.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn surface_region(
        command_buffer: vk::CommandBuffer,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanSharedLegacyGraphicsPipeline,
        source: &VulkanImageView<'_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        next_target_state: VulkanImageState,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        texture_transform: u32,
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        Self::textured_region(
            command_buffer, render_pass, view, pipeline, source, binding, 0,
            previous_source_state, previous_target_state, next_target_state,
            super::VulkanColorLoad::Load, viewport, scissor,
            &texture_transform.to_ne_bytes(), vk::ShaderStageFlags::VERTEX,
        )
    }

    /// Records a wallpaper region with its four-float UV transform.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn wallpaper_region(
        command_buffer: vk::CommandBuffer,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanSharedLegacyGraphicsPipeline,
        source: &VulkanImageView<'_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        next_target_state: VulkanImageState,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        uv_transform: [f32; 4],
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        let mut bytes = [0_u8; 16];
        for (index, value) in uv_transform.iter().enumerate() {
            bytes[index * 4..(index + 1) * 4].copy_from_slice(&value.to_ne_bytes());
        }
        Self::textured_region(
            command_buffer, render_pass, view, pipeline, source, binding, 0,
            previous_source_state, previous_target_state, next_target_state,
            super::VulkanColorLoad::Load, viewport, scissor,
            &bytes, vk::ShaderStageFlags::VERTEX,
        )
    }

    /// Records one alpha-faded cursor-trail impression.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cursor_trail_region(
        command_buffer: vk::CommandBuffer,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
        view: &VulkanImageView<'_, '_>,
        pipeline: &VulkanSharedLegacyGraphicsPipeline,
        source: &VulkanImageView<'_, '_>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        next_target_state: VulkanImageState,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        texture_transform: u32,
        opacity: f32,
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        let mut bytes = [0_u8; 8];
        bytes[0..4].copy_from_slice(&texture_transform.to_ne_bytes());
        bytes[4..8].copy_from_slice(&opacity.to_ne_bytes());
        Self::textured_region(
            command_buffer, render_pass, view, pipeline, source, binding, 0,
            previous_source_state, previous_target_state, next_target_state,
            super::VulkanColorLoad::Load, viewport, scissor,
            &bytes, vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        )
    }

    /// Applies display-color processing as the final composition layer.
    ///
    /// The source is the completed composition image and the target is the
    /// scanout image. The target must use the discard render pass owned by the
    /// display-color pass; the method leaves it in the caller-selected state
    /// for either readback or foreign-queue release.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn display_color(
        pass: &VulkanSharedLegacyDisplayColorPass,
        device: &super::VulkanLogicalDevice,
        command_buffer: vk::CommandBuffer,
        source: &VulkanImageView<'_, '_>,
        target: &VulkanImageView<'_, '_>,
        render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
        binding: &VulkanTextureBinding<'_, '_, '_, '_>,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        final_target_state: VulkanImageState,
        warmth_percent: u8,
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        let target = render_pass
            .create_target(target)
            .map_err(VulkanLegacyFrameRecorderError::RenderPass)?;
        pass.record(
            device,
            command_buffer,
            source,
            &target,
            binding,
            previous_source_state,
            previous_target_state,
            final_target_state,
            warmth_percent,
        )
        .map_err(VulkanLegacyFrameRecorderError::DisplayColor)
    }

    /// Ends a legacy scanout frame, releases its output image to KMS, and submits it.
    ///
    /// This is the terminal path for DMA-BUF scanout frames. Readback frames must
    /// use their transfer-copy path instead and must not release ownership to KMS.
    pub(crate) fn release_and_submit(
        command_buffer: &mut super::VulkanCommandBuffer<'_>,
        device: &super::VulkanLogicalDevice,
        output: &VulkanImageView<'_, '_>,
        previous_output_state: VulkanImageState,
        on_submitted: impl FnOnce(),
    ) -> Result<(), VulkanLegacyFrameRecorderError> {
        if output.device().handle().handle() != device.handle().handle() {
            return Err(VulkanLegacyFrameRecorderError::DeviceMismatch);
        }

        super::VulkanImageTransition::record_release_to_foreign(
            device,
            command_buffer.handle(),
            output.image_handle(),
            previous_output_state,
        )
        .map_err(VulkanLegacyFrameRecorderError::Transition)?;

        command_buffer
            .end()
            .map_err(|error| VulkanLegacyFrameRecorderError::Command(error.to_string()))?;
        command_buffer
            .submit_and_wait_with(on_submitted)
            .map_err(|error| VulkanLegacyFrameRecorderError::Command(format!("{error:?}")))?;

        Ok(())
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
        view: &VulkanImageView<'_, '_>,
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
    DisplayColor(VulkanDisplayColorPassError),
    Transition(super::VulkanImageTransitionError),
    DeviceMismatch,
    Command(String),
}

impl std::fmt::Display for VulkanLegacyFrameRecorderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RenderPass(error) => write!(formatter, "Legacy frame target: {error}"),
            Self::Clear(error) => write!(formatter, "Legacy frame clear: {error}"),
            Self::Draw(error) => write!(formatter, "Legacy frame draw: {error}"),
            Self::DisplayColor(error) => write!(formatter, "Legacy frame display color: {error}"),
            Self::Transition(error) => write!(formatter, "Legacy frame output release: {error}"),
            Self::DeviceMismatch => formatter.write_str("Legacy frame output belongs to a different Vulkan device"),
            Self::Command(error) => write!(formatter, "Legacy frame command submission: {error}"),
        }
    }
}

impl std::error::Error for VulkanLegacyFrameRecorderError {}
