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
            VulkanImageState::ShaderRead,
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
        VulkanImageState::ShaderRead,
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
        VulkanImageState::ShaderRead,
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
            VulkanImageState::ShaderRead,
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

/// Optional wallpaper draw resources and output-local geometry.
pub(super) struct LegacyWallpaperLayer<'r, 'i, 'id, 'rd> {
    pub source: &'r VulkanImageView<'i, 'id>,
    pub binding: &'r VulkanTextureBinding<'r, 'i, 'id, 'rd>,
    pub pipeline: &'r VulkanSharedLegacyGraphicsPipeline,
    pub render_pass: &'r Arc<VulkanLegacyRenderPass<'static>>,
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    pub uv_transform: [f32; 4],
}

/// Optional context-menu artwork and its already-resolved geometry.
pub(super) struct LegacyContextMenuLayer<'r, 'i, 'id, 'rd> {
    pub source: &'r VulkanImageView<'i, 'id>,
    pub binding: &'r VulkanTextureBinding<'r, 'i, 'id, 'rd>,
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    pub texture_transform: u32,
}

/// Optional compositor-owned cursor draw.
pub(super) struct LegacyNativeCursorLayer<'r, 'i, 'id, 'rd> {
    pub source: &'r VulkanImageView<'i, 'id>,
    pub binding: &'r VulkanTextureBinding<'r, 'i, 'id, 'rd>,
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    pub texture_transform: u32,
}

/// Optional cursor-trail draw resources.
pub(super) struct LegacyCursorTrailLayer<'r, 'i, 'id, 'rd> {
    pub source: &'r VulkanImageView<'i, 'id>,
    pub binding: &'r VulkanTextureBinding<'r, 'i, 'id, 'rd>,
    pub pipeline: &'r VulkanSharedLegacyGraphicsPipeline,
    pub impressions: &'r [CursorTrailDraw],
}

/// All resources required to record the legacy scene layers for one frame.
pub(super) struct LegacyFrameLayers<'r, 'i, 'id, 'rd> {
    pub clear_render_pass: &'r Arc<VulkanLegacyRenderPass<'static>>,
    pub load_render_pass: &'r Arc<VulkanLegacyRenderPass<'static>>,
    pub target: &'r VulkanImageView<'i, 'id>,
    pub clear_color: [f32; 4],
    pub visible: &'r [VisibleSurface],
    pub surface_views: &'r [VulkanImageView<'i, 'id>],
    pub surface_bindings: &'r [VulkanTextureBinding<'r, 'i, 'id, 'rd>],
    pub surface_pipeline: &'r VulkanSharedLegacyGraphicsPipeline,
    pub wallpaper: Option<LegacyWallpaperLayer<'r, 'i, 'id, 'rd>>,
    pub context_menu: Option<LegacyContextMenuLayer<'r, 'i, 'id, 'rd>>,
    pub cursor_trail: Option<LegacyCursorTrailLayer<'r, 'i, 'id, 'rd>>,
    pub native_cursor: Option<LegacyNativeCursorLayer<'r, 'i, 'id, 'rd>>,
}

/// Records the complete scene in the compositor's established back-to-front order.
///
/// Output display-color processing and the terminal DRM/readback operation are
/// intentionally left to the caller, after all scene layers have been recorded.
pub(super) fn record_legacy_frame_layers(
    command_buffer: vk::CommandBuffer,
    layers: LegacyFrameLayers<'_, '_, '_, '_>,
    previous_target_state: VulkanImageState,
) -> Result<(), VulkanSurfaceCompositionError> {
    // Validate the parallel frame arrays before recording even the clear pass.
    // A malformed frame must not leave a partially recorded command buffer.
    if layers.visible.len() != layers.surface_views.len()
        || layers.surface_views.len() != layers.surface_bindings.len()
    {
        return Err(VulkanSurfaceCompositionError::FrameResourceCountMismatch);
    }

    VulkanLegacyFrameRecorder::clear(
        command_buffer,
        layers.clear_render_pass,
        layers.target,
        previous_target_state,
        VulkanImageState::General,
        layers.clear_color,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    if let Some(wallpaper) = layers.wallpaper {
        record_wallpaper(
            command_buffer,
            wallpaper.render_pass,
            layers.target,
            wallpaper.pipeline,
            wallpaper.source,
            wallpaper.binding,
            wallpaper.viewport,
            wallpaper.scissor,
            wallpaper.uv_transform,
        )?;
    }

    record_normal_surfaces(
        command_buffer,
        layers.load_render_pass,
        layers.target,
        layers.surface_pipeline,
        layers.visible,
        layers.surface_views,
        layers.surface_bindings,
    )?;

    if let Some(menu) = layers.context_menu {
        record_context_menu(
            command_buffer,
            layers.load_render_pass,
            layers.target,
            layers.surface_pipeline,
            menu.source,
            menu.binding,
            menu.viewport,
            menu.scissor,
            menu.texture_transform,
        )?;
    }

    record_client_cursor_surfaces(
        command_buffer,
        layers.load_render_pass,
        layers.target,
        layers.surface_pipeline,
        layers.visible,
        layers.surface_views,
        layers.surface_bindings,
    )?;

    if let Some(trail) = layers.cursor_trail {
        record_cursor_trail(
            command_buffer,
            layers.load_render_pass,
            layers.target,
            trail.pipeline,
            trail.source,
            trail.binding,
            trail.impressions,
        )?;
    }

    if let Some(cursor) = layers.native_cursor {
        record_native_cursor(
            command_buffer,
            layers.load_render_pass,
            layers.target,
            layers.surface_pipeline,
            cursor.source,
            cursor.binding,
            cursor.viewport,
            cursor.scissor,
            cursor.texture_transform,
        )?;
    }

    Ok(())
}

