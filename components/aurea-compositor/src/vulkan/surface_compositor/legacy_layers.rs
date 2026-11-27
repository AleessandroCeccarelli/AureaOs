// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Traditional-render-pass recording for Wayland surface layers.

use smithay::reexports::ash::vk;

use super::{VulkanSurfaceCompositionError, visible_surfaces::VisibleSurface, cursor_trail_geometry::CursorTrailDraw};
use super::super::{
    VulkanImageState, VulkanImageView, VulkanLegacyFrameRecorder,
    VulkanLegacyFrameRecorderError, VulkanLegacyRenderTarget,
    VulkanSharedLegacyGraphicsPipeline, VulkanTextureBinding,
};

/// Records one class of Wayland surfaces while preserving their scene order.
fn record_surface_class<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
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

        let source_scissor = surface.scissor;
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
                x: i32::try_from(source_scissor.x)
                    .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
                y: i32::try_from(source_scissor.y)
                    .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
            },
            extent: vk::Extent2D {
                width: source_scissor.width,
                height: source_scissor.height,
            },
        };

        VulkanLegacyFrameRecorder::surface_region(
            command_buffer,
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
pub(super) fn record_context_menu<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    texture_transform: u32,
) -> Result<(), VulkanSurfaceCompositionError> {
    VulkanLegacyFrameRecorder::surface_region(
        command_buffer, target, pipeline, source, binding,
        VulkanImageState::ShaderRead, VulkanImageState::General,
        VulkanImageState::General, viewport, scissor, texture_transform,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))
}

/// Records the desktop wallpaper using its dedicated four-float UV transform ABI.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_wallpaper<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    uv_transform: [f32; 4],
) -> Result<(), VulkanSurfaceCompositionError> {
    VulkanLegacyFrameRecorder::wallpaper_region(
        command_buffer, target, pipeline, source, binding,
        VulkanImageState::ShaderRead, VulkanImageState::General,
        VulkanImageState::General, viewport, scissor, uv_transform,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))
}

/// Records the compositor-owned native cursor above the cursor-trail layer.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_native_cursor<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    texture_transform: u32,
) -> Result<(), VulkanSurfaceCompositionError> {
    VulkanLegacyFrameRecorder::surface_region(
        command_buffer, target, pipeline, source, binding,
        VulkanImageState::ShaderRead, VulkanImageState::General,
        VulkanImageState::General, viewport, scissor, texture_transform,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))
}

/// Records cursor-trail impressions in their supplied back-to-front order.
pub(super) fn record_cursor_trail<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    source: &VulkanImageView<'_, '_>,
    binding: &VulkanTextureBinding<'_, '_, '_, '_>,
    impressions: &[CursorTrailDraw],
) -> Result<(), VulkanSurfaceCompositionError> {
    for impression in impressions {
        VulkanLegacyFrameRecorder::cursor_trail_region(
            command_buffer, target, pipeline, source, binding,
            VulkanImageState::ShaderRead, VulkanImageState::General,
            VulkanImageState::General, impression.viewport, impression.scissor,
            impression.texture_transform, impression.opacity,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }
    Ok(())
}

/// Records ordinary client surfaces.
pub(super) fn record_normal_surfaces<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    surfaces: &[VisibleSurface],
    views: &[VulkanImageView<'_, '_>],
    bindings: &[VulkanTextureBinding<'_, '_, '_, '_>],
) -> Result<(), VulkanSurfaceCompositionError> {
    record_surface_class(command_buffer, target, pipeline, surfaces, views, bindings, false)
}

/// Records client-provided cursor surfaces above the context menu.
pub(super) fn record_client_cursor_surfaces<'pass, 'view, 'image, 'pass_device, 'view_device>(
    command_buffer: vk::CommandBuffer,
    target: &VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pipeline: &VulkanSharedLegacyGraphicsPipeline,
    surfaces: &[VisibleSurface],
    views: &[VulkanImageView<'_, '_>],
    bindings: &[VulkanTextureBinding<'_, '_, '_, '_>],
) -> Result<(), VulkanSurfaceCompositionError> {
    record_surface_class(command_buffer, target, pipeline, surfaces, views, bindings, true)
}

/// Optional wallpaper draw resources and output-local geometry.
pub(super) struct LegacyWallpaperLayer<'r, 'i, 'id, 'rd> {
    pub source: &'r VulkanImageView<'i, 'id>,
    pub binding: &'r VulkanTextureBinding<'r, 'i, 'id, 'rd>,
    pub pipeline: &'r VulkanSharedLegacyGraphicsPipeline,
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
pub(super) struct LegacyFrameLayers<'pass, 'view, 'image, 'pass_device, 'view_device, 'r, 'i, 'id, 'rd> {
    pub clear_target: &'r VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
    pub load_target: &'r VulkanLegacyRenderTarget<'pass, 'view, 'image, 'pass_device, 'view_device>,
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
    layers: LegacyFrameLayers<'_, '_, '_, '_, '_, '_, '_, '_, '_>,
    previous_target_state: VulkanImageState,
) -> Result<(), VulkanSurfaceCompositionError> {
    if layers.visible.len() != layers.surface_views.len()
        || layers.surface_views.len() != layers.surface_bindings.len()
    {
        return Err(VulkanSurfaceCompositionError::FrameResourceCountMismatch);
    }

    if !std::ptr::eq(
        layers.surface_pipeline.render_pass().as_ref(),
        layers.load_target.render_pass_ref(),
    ) {
        return Err(VulkanSurfaceCompositionError::Command(
            "Legacy surface pipeline does not use the frame load render pass".to_owned(),
        ));
    }
    if let Some(wallpaper) = layers.wallpaper.as_ref() {
        if !std::ptr::eq(
            wallpaper.pipeline.render_pass().as_ref(),
            layers.load_target.render_pass_ref(),
        ) {
            return Err(VulkanSurfaceCompositionError::Command(
                "Legacy wallpaper pipeline does not use the frame load render pass".to_owned(),
            ));
        }
    }
    if let Some(trail) = layers.cursor_trail.as_ref() {
        if !std::ptr::eq(
            trail.pipeline.render_pass().as_ref(),
            layers.load_target.render_pass_ref(),
        ) {
            return Err(VulkanSurfaceCompositionError::Command(
                "Legacy cursor-trail pipeline does not use the frame load render pass".to_owned(),
            ));
        }
    }

    VulkanLegacyFrameRecorder::clear(
        command_buffer, layers.clear_target, previous_target_state,
        VulkanImageState::General, layers.clear_color,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    if let Some(wallpaper) = layers.wallpaper {
        record_wallpaper(
            command_buffer, layers.load_target, wallpaper.pipeline,
            wallpaper.source, wallpaper.binding, wallpaper.viewport,
            wallpaper.scissor, wallpaper.uv_transform,
        )?;
    }

    record_normal_surfaces(
        command_buffer, layers.load_target, layers.surface_pipeline,
        layers.visible, layers.surface_views, layers.surface_bindings,
    )?;

    if let Some(menu) = layers.context_menu {
        record_context_menu(
            command_buffer, layers.load_target, layers.surface_pipeline,
            menu.source, menu.binding, menu.viewport, menu.scissor,
            menu.texture_transform,
        )?;
    }

    record_client_cursor_surfaces(
        command_buffer, layers.load_target, layers.surface_pipeline,
        layers.visible, layers.surface_views, layers.surface_bindings,
    )?;

    if let Some(trail) = layers.cursor_trail {
        record_cursor_trail(
            command_buffer, layers.load_target, trail.pipeline,
            trail.source, trail.binding, trail.impressions,
        )?;
    }

    if let Some(cursor) = layers.native_cursor {
        record_native_cursor(
            command_buffer, layers.load_target, layers.surface_pipeline,
            cursor.source, cursor.binding, cursor.viewport, cursor.scissor,
            cursor.texture_transform,
        )?;
    }

    Ok(())
}
