// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Minimal non-indexed graphics draw pass for Vulkan dynamic rendering.

use std::{error::Error, fmt};

use smithay::reexports::ash::vk;

use super::{
    VulkanColorLoad, VulkanDynamicRenderingScope, VulkanGraphicsPipeline, VulkanImageState,
    VulkanImageTransition, VulkanImageView, VulkanLegacyGraphicsPipeline, VulkanLegacyRenderTarget,
    VulkanSharedLegacyGraphicsPipeline, VulkanTextureBinding,
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
    IncompatibleRenderPassLoad,
    MissingSampledImageUsage,
    TextureSourceMismatch,
    IncompatibleTextureSource,
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
            Self::IncompatibleRenderPassLoad => {
                formatter.write_str("The legacy draw load operation does not match its render pass")
            }
            Self::MissingSampledImageUsage => {
                formatter.write_str("The legacy texture source lacks sampled-image usage")
            }
            Self::TextureSourceMismatch => formatter
                .write_str("The legacy descriptor binding references a different source image"),
            Self::IncompatibleTextureSource => formatter.write_str(
                "The legacy texture source and target must be distinct images on the same device",
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

    /// Records a basic draw through the traditional render-pass backend.
    ///
    /// The target must use a render pass created with the same clear/load
    /// behavior intended by the caller. This first compatibility recorder
    /// deliberately supports the basic full-target draw path only.
    pub fn record_legacy(
        command_buffer: vk::CommandBuffer,
        target: &VulkanLegacyRenderTarget<'_, '_, '_, '_, '_>,
        pipeline: &VulkanLegacyGraphicsPipeline<'_, '_>,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        clear_color: [f32; 4],
        vertex_count: u32,
        instance_count: u32,
    ) -> Result<(), VulkanDrawPassError> {
        Self::validate(
            vertex_count,
            instance_count,
            final_state,
            VulkanColorLoad::Clear(clear_color),
        )?;
        if target.render_pass_ref().load_op() != vk::AttachmentLoadOp::CLEAR {
            return Err(VulkanDrawPassError::IncompatibleRenderPassLoad);
        }
        let view = target.view();
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDrawPassError::MissingColorAttachmentUsage);
        }
        if !pipeline.is_compatible_with(view.device(), view.format(), target.render_pass_ref()) {
            return Err(VulkanDrawPassError::IncompatiblePipeline);
        }

        VulkanImageTransition::record_color(
            view.device(),
            command_buffer,
            view.image_handle(),
            previous_state,
            VulkanImageState::ColorAttachment,
        );

        let scope = target.begin(command_buffer, clear_color);
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

    /// Records a textured region through a traditional render pass.
    ///
    /// The descriptor binding, legacy pipeline and render target must remain
    /// alive until the submitted command buffer has completed. The render
    /// pass load operation must match `load`; use `Load` when compositing
    /// over an already-rendered wallpaper or surface.
    #[allow(clippy::too_many_arguments)]
    pub fn record_legacy_textured_region(
        command_buffer: vk::CommandBuffer,
        target: &VulkanLegacyRenderTarget<'_, '_, '_, '_, '_>,
        pipeline: &VulkanLegacyGraphicsPipeline<'_, '_>,
        source: &VulkanImageView<'_, '_>,
        texture: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        push_constants: &[u8],
        push_constant_stages: vk::ShaderStageFlags,
    ) -> Result<(), VulkanDrawPassError> {
        Self::validate(3, 1, final_state, load)?;
        Self::validate_push_constants(push_constants)?;
        let view = target.view();
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDrawPassError::MissingSampledImageUsage);
        }
        if !texture.references_view(source) {
            return Err(VulkanDrawPassError::TextureSourceMismatch);
        }
        if source.image_handle() == view.image_handle()
            || source.device().handle().handle() != view.device().handle().handle()
        {
            return Err(VulkanDrawPassError::IncompatibleTextureSource);
        }
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDrawPassError::MissingColorAttachmentUsage);
        }
        if !pipeline.is_compatible_with(view.device(), view.format(), target.render_pass_ref()) {
            return Err(VulkanDrawPassError::IncompatiblePipeline);
        }
        if !Self::legacy_load_matches(target, load) {
            return Err(VulkanDrawPassError::IncompatibleRenderPassLoad);
        }
        Self::validate_region(view.width(), view.height(), viewport, scissor)?;

        VulkanImageTransition::record_color(
            source.device(),
            command_buffer,
            source.image_handle(),
            previous_source_state,
            VulkanImageState::ShaderRead,
        );
        VulkanImageTransition::record_color(
            view.device(),
            command_buffer,
            view.image_handle(),
            previous_target_state,
            VulkanImageState::ColorAttachment,
        );

        let scope = target.begin(command_buffer, [0.0; 4]);
        pipeline.bind(command_buffer);
        texture.bind(command_buffer, pipeline.layout(), set_index);
        pipeline.push_constants(command_buffer, push_constant_stages, 0, push_constants);
        pipeline.set_viewport_and_scissor_region(command_buffer, viewport, scissor);
        pipeline.draw(command_buffer, 3, 1);
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

    fn legacy_load_matches(
        target: &VulkanLegacyRenderTarget<'_, '_, '_, '_, '_>,
        load: VulkanColorLoad,
    ) -> bool {
        let expected = match load {
            VulkanColorLoad::Clear(_) => vk::AttachmentLoadOp::CLEAR,
            VulkanColorLoad::Load => vk::AttachmentLoadOp::LOAD,
            VulkanColorLoad::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        };
        target.render_pass_ref().load_op() == expected
    }

    /// Records a textured region through a cached shared-ownership legacy pipeline.
    ///
    /// This is the cache-friendly Vulkan 1.0 counterpart to
    /// `record_legacy_textured_region`. The pipeline owns its render pass,
    /// while the target and texture binding remain scoped to the current frame.
    #[allow(clippy::too_many_arguments)]
    pub fn record_shared_legacy_textured_region(
        command_buffer: vk::CommandBuffer,
        target: &VulkanLegacyRenderTarget<'_, '_, '_, '_, '_>,
        pipeline: &VulkanSharedLegacyGraphicsPipeline,
        source: &VulkanImageView<'_, '_>,
        texture: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        previous_source_state: VulkanImageState,
        previous_target_state: VulkanImageState,
        final_state: VulkanImageState,
        load: VulkanColorLoad,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        push_constants: &[u8],
        push_constant_stages: vk::ShaderStageFlags,
    ) -> Result<(), VulkanDrawPassError> {
        Self::validate(3, 1, final_state, load)?;
        Self::validate_push_constants(push_constants)?;
        let view = target.view();
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDrawPassError::MissingSampledImageUsage);
        }
        if !texture.references_view(source) {
            return Err(VulkanDrawPassError::TextureSourceMismatch);
        }
        if source.image_handle() == view.image_handle()
            || source.device().handle().handle() != view.device().handle().handle()
        {
            return Err(VulkanDrawPassError::IncompatibleTextureSource);
        }
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDrawPassError::MissingColorAttachmentUsage);
        }
        if !pipeline.is_compatible_with_target(
            view.device(),
            view.format(),
            target.render_pass_ref(),
        ) {
            return Err(VulkanDrawPassError::IncompatiblePipeline);
        }
        if !Self::legacy_load_matches(target, load) {
            return Err(VulkanDrawPassError::IncompatibleRenderPassLoad);
        }
        Self::validate_region(view.width(), view.height(), viewport, scissor)?;

        VulkanImageTransition::record_color(
            source.device(),
            command_buffer,
            source.image_handle(),
            previous_source_state,
            VulkanImageState::ShaderRead,
        );
        VulkanImageTransition::record_color(
            view.device(),
            command_buffer,
            view.image_handle(),
            previous_target_state,
            VulkanImageState::ColorAttachment,
        );

        let scope = target.begin(command_buffer, [0.0; 4]);
        pipeline.bind(command_buffer);
        texture.bind(command_buffer, pipeline.layout(), set_index);
        pipeline.push_constants(command_buffer, push_constant_stages, 0, push_constants);
        pipeline.set_viewport_and_scissor_region(command_buffer, viewport, scissor);
        pipeline.draw(command_buffer, 3, 1);
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

    /// Records a full-target clear draw with a cached shared legacy pipeline.
    pub fn record_shared_legacy_clear(
        command_buffer: vk::CommandBuffer,
        target: &VulkanLegacyRenderTarget<'_, '_, '_, '_, '_>,
        pipeline: &VulkanSharedLegacyGraphicsPipeline,
        previous_state: VulkanImageState,
        final_state: VulkanImageState,
        clear_color: [f32; 4],
        vertex_count: u32,
        instance_count: u32,
    ) -> Result<(), VulkanDrawPassError> {
        Self::validate(
            vertex_count,
            instance_count,
            final_state,
            VulkanColorLoad::Clear(clear_color),
        )?;
        if target.render_pass_ref().load_op() != vk::AttachmentLoadOp::CLEAR {
            return Err(VulkanDrawPassError::IncompatibleRenderPassLoad);
        }
        let view = target.view();
        if !view.usage().contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            return Err(VulkanDrawPassError::MissingColorAttachmentUsage);
        }
        if !pipeline.is_compatible_with_target(
            view.device(),
            view.format(),
            target.render_pass_ref(),
        ) {
            return Err(VulkanDrawPassError::IncompatiblePipeline);
        }
        VulkanImageTransition::record_color(
            view.device(),
            command_buffer,
            view.image_handle(),
            previous_state,
            VulkanImageState::ColorAttachment,
        );
        let scope = target.begin(command_buffer, clear_color);
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
            vk::ShaderStageFlags::VERTEX,
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
            vk::ShaderStageFlags::VERTEX,
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
            vk::ShaderStageFlags::VERTEX,
        )
    }

    /// Records one client DMA-BUF surface with explicit foreign queue ownership.
    ///
    /// The caller must have imported and retained the producer SYNC_FD semaphore
    /// in the submission wait list. The source image is acquired from the foreign
    /// queue family, sampled in SHADER_READ_ONLY_OPTIMAL, then released back in
    /// GENERAL so the Wayland client can reuse the buffer after submission ends.
    #[cfg(feature = "drm")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_foreign_textured_region(
        command_buffer: vk::CommandBuffer,
        target: &VulkanImageView<'_, '_>,
        source: &VulkanImageView<'_, '_>,
        pipeline: &VulkanGraphicsPipeline<'_>,
        texture: &VulkanTextureBinding<'_, '_, '_, '_>,
        set_index: u32,
        target_previous_state: VulkanImageState,
        target_final_state: VulkanImageState,
        load: VulkanColorLoad,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        texture_transform: u32,
    ) -> Result<(), VulkanDrawPassError> {
        if !source.usage().contains(vk::ImageUsageFlags::SAMPLED) {
            return Err(VulkanDrawPassError::MissingSampledImageUsage);
        }
        if !texture.references_view(source) {
            return Err(VulkanDrawPassError::TextureSourceMismatch);
        }
        if source.image_handle() == target.image_handle()
            || source.device().handle().handle() != target.device().handle().handle()
        {
            return Err(VulkanDrawPassError::IncompatibleTextureSource);
        }

        VulkanImageTransition::record_acquire_from_foreign(
            source.device(),
            command_buffer,
            source.image_handle(),
            VulkanImageState::ShaderRead,
        )
        .map_err(|_| VulkanDrawPassError::IncompatibleTextureSource)?;

        let result = Self::record_inner(
            command_buffer,
            target,
            pipeline,
            target_previous_state,
            target_final_state,
            load,
            3,
            1,
            Some((texture, set_index)),
            Some((viewport, scissor)),
            Some(&texture_transform.to_ne_bytes()),
            vk::ShaderStageFlags::VERTEX,
        );

        if result.is_ok() {
            VulkanImageTransition::record_release_to_foreign(
                source.device(),
                command_buffer,
                source.image_handle(),
                VulkanImageState::ShaderRead,
            )
            .map_err(|_| VulkanDrawPassError::IncompatibleTextureSource)?;
        }
        result
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
            vk::ShaderStageFlags::VERTEX,
        )
    }

    /// Records a textured region while exposing push constants to both shader stages.
    #[allow(clippy::too_many_arguments)]
    pub fn record_textured_region_with_stage_push_constants(
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
        stages: vk::ShaderStageFlags,
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
            stages,
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
        push_constant_stages: vk::ShaderStageFlags,
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
                pipeline.push_constants(command_buffer, push_constant_stages, 0, data);
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
