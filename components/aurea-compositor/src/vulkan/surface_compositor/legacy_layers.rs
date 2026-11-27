// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Traditional-render-pass recording for Wayland surface layers.

use std::sync::Arc;

use smithay::reexports::ash::vk;

use super::{VulkanSurfaceCompositionError, visible_surfaces::VisibleSurface, cursor_trail_geometry::CursorTrailDraw};
use super::super::{
    VulkanImageState, VulkanImageView, VulkanLegacyFrameRecorder,
    VulkanLegacyFrameRecorderError, VulkanLegacyRenderPass,
    VulkanSharedLegacyGraphicsPipeline, VulkanTextureBinding,
};

/// Records one class of Wayland surfaces while preserving their scene order.
fn record_surface_class(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    surfaces: &[VisibleSurface],
    views: &[VulkanImageView<'_, '_>],
    bindings: &[VulkanTextureBinding<'_, '_, '_, '_>],
    cursor_surfaces: bool,
 ) -> Result<(), VulkanSurfaceCompositionError> {
    if surfaces.len() != views.len() || views.len() != bindings.len() {
        return Err(VulkanSurfaceCompositionError::FrameResourceCountMismatch);
    }

    for ((surface, source), binding) in surfaces.iter().zip(views).zip(bindings) {
        if surface.is_cursor != cursor_surfaces {
            continue;
        }

        let scissor = surface.scissor;
        let viewport = vk::Viewport {
            x: surface.geometry.x() as f32,
            y: surface.geometry.y() as f32,
            width: surface.geometry.width() as f32,
            height: surface.geometry.height() as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: i32::try_from(scissor.x)
                    .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
                y: i32::try_from(scissor.y)
                    .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
            },
            extent: vk::Extent2D {
                width: scissor.width,
                height: scissor.height,
            },
        };

        VulkanLegacyFrameRecorder::surface_region(
            command_buffer,
            render_pass,
            target,
            pipeline,
            source,
            binding,
            VulkanImageState::General,
            VulkanImageState::General,
            VulkanImageState::General,
            viewport,
            scissor,
            surface.texture_transform,
        )
        .map_err(|error: VulkanLegacyFrameRecorderError| {
            VulkanSurfaceCompositionError::Command(error.to_string())
        })?;
    }

    Ok(())
}


/// Records the desktop context-menu artwork between normal surfaces and client cursors.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_context_menu(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    texture_transform: u32,
) -> Result<(), VulkanSurfaceCompositionError> {
    VulkanLegacyFrameRecorder::surface_region(
        command_buffer,
        render_pass,
        target,
        pipeline,
        source,
        binding,
        VulkanImageState::General,
        VulkanImageState::General,
        VulkanImageState::General,
        viewport,
        scissor,
        texture_transform,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))
}

/// Records the desktop wallpaper using its dedicated four-float UV transform ABI.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_wallpaper(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    uv_transform: [f32; 4],
) -> Result<(), VulkanSurfaceCompositionError> {
    VulkanLegacyFrameRecorder::wallpaper_region(
        command_buffer,
        render_pass,
        target,
        pipeline,
        source,
        binding,
        VulkanImageState::ShaderRead,
        VulkanImageState::General,
        VulkanImageState::General,
        viewport,
        scissor,
        uv_transform,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))
}


/// Records the compositor-owned native cursor above the cursor-trail layer.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_native_cursor(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    texture_transform: u32,
) -> Result<(), VulkanSurfaceCompositionError> {
    VulkanLegacyFrameRecorder::surface_region(
        command_buffer,
        render_pass,
        target,
        pipeline,
        source,
        binding,
        VulkanImageState::General,
        VulkanImageState::General,
        VulkanImageState::General,
        viewport,
        scissor,
        texture_transform,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))
}

/// Records cursor-trail impressions in their supplied back-to-front order.
pub(super) fn record_cursor_trail(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    impressions: &[CursorTrailDraw],
) -> Result<(), VulkanSurfaceCompositionError> {
    for impression in impressions {
        VulkanLegacyFrameRecorder::cursor_trail_region(
            command_buffer,
            render_pass,
            target,
            pipeline,
            source,
            binding,
            VulkanImageState::General,
            VulkanImageState::General,
            VulkanImageState::General,
            impression.viewport,
            impression.scissor,
            impression.texture_transform,
            impression.opacity,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }
    Ok(())
}

/// Records ordinary client surfaces first and client cursor surfaces afterwards.
///
/// The caller places the context menu between the two calls when preserving the
/// complete compositor layer order.
pub(super) fn record_normal_surfaces(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    surfaces: &[VisibleSurface],
    views: &[VulkanImageView<'_, '_>],
    bindings: &[VulkanTextureBinding<'_, '_, '_, '_>],
) -> Result<(), VulkanSurfaceCompositionError> {
    record_surface_class(
        command_buffer, render_pass, target, pipeline, surfaces, views, bindings, false,
    )
}

/// Records client-provided cursor surfaces above the context menu.
pub(super) fn record_client_cursor_surfaces(
    command_buffer: vk::CommandBuffer,
    render_pass: &Arc<VulkanLegacyRenderPass<'static>>,
    target: &VulkanImageView<'_, '_>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    surfaces: &[VisibleSurface],
    views: &[VulkanImageView<'_, '_>],
    bindings: &[VulkanTextureBinding<'_, '_, '_, '_>],
) -> Result<(), VulkanSurfaceCompositionError> {
    record_surface_class(
        command_buffer, render_pass, target, pipeline, surfaces, views, bindings, true,
    )
}
