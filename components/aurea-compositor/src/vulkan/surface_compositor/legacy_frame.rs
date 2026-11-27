// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Vulkan 1.0-compatible frame preparation, recording and submission.

use std::{sync::Arc, time::Instant};

use smithay::reexports::ash::vk;

use crate::{
    desktop::{WallpaperDrawGeometry, WallpaperFitMode, WallpaperLayout},
    render::{RenderScene, geometry::{SurfaceGeometry, SurfaceScissor}},
    renderer::RenderTarget,
};

use super::{
    VulkanCommandBuffer, VulkanDisplayColorPassError, VulkanImageState, VulkanImageView,
    VulkanLegacyFrameRecorder, VulkanReadbackBuffer, VulkanSharedLegacyDisplayColorPass,
    VulkanSurfaceCompositionError, VulkanSurfacePipelineCache, VulkanTextureBinding,
    cursor_trail_geometry::CursorTrailDraw,
    legacy_layers::{
        self, LegacyContextMenuLayer, LegacyCursorTrailLayer, LegacyFrameLayers,
        LegacyNativeCursorLayer, LegacyWallpaperLayer,
    },
    surface_pipeline_cache::LegacyPipelineLoad,
    visible_surfaces::VisibleSurface,
};

/// Records a complete compatibility frame using the resources already prepared
/// by the common scene/cache path. No Vulkan 1.3 dynamic-rendering object is used.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_frame_legacy<K: Copy>(
    device: &super::super::VulkanLogicalDevice,
    output_view: &VulkanImageView<'_, '_>,
    composition_view: &VulkanImageView<'_, '_>,
    target: &RenderTarget,
    scene: &RenderScene<'_>,
    surface_cache: &mut VulkanSurfacePipelineCache,
    previous_composition_state: VulkanImageState,
    visible: &[VisibleSurface],
    wallpaper_asset_id: Option<&str>,
    context_menu_key: Option<K>,
    context_menu_draw: Option<(vk::Viewport, vk::Rect2D, u32)>,
    cursor: Option<(SurfaceGeometry, SurfaceScissor, u32)>,
    cursor_trail: &[CursorTrailDraw],
    cursor_texture_ready: bool,
    cursor_name: &str,
    requested_cursor_size: u32,
    cursor_frame_time: Instant,
    display_warmth: Option<u8>,
    readback: Option<&VulkanReadbackBuffer<'_>>,
    on_submitted: impl FnOnce(),
) -> Result<(), VulkanSurfaceCompositionError> {
    let format = composition_view.format();
    let wallpaper_ready = wallpaper_asset_id.is_some_and(|asset_id| {
        surface_cache.wallpaper_texture_for_asset(asset_id).is_some()
    });
    let context_menu_ready = context_menu_key
        .is_some_and(|key| surface_cache.desktop_icon_texture(key).is_some());

    surface_cache
        .prepare_shared_legacy_frame_resources_without_display_color(
            format,
            wallpaper_ready,
            !cursor_trail.is_empty(),
        )
        .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;

    let clear_render_pass = surface_cache
        .legacy_clear_render_pass(format)
        .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;
    let load_render_pass = surface_cache
        .legacy_load_render_pass(format)
        .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;

    let surface_pipeline = surface_cache
        .shared_legacy_surface_pipeline(format, LegacyPipelineLoad::Load)
        .ok_or_else(|| VulkanSurfaceCompositionError::Pipeline(
            "The compatibility surface pipeline was not cached".to_owned(),
        ))?;
    let wallpaper_pipeline = if wallpaper_ready {
        Some(surface_cache.shared_legacy_wallpaper_pipeline(format, LegacyPipelineLoad::Load)
            .ok_or_else(|| VulkanSurfaceCompositionError::Pipeline(
                "The compatibility wallpaper pipeline was not cached".to_owned(),
            ))?)
    } else {
        None
    };
    let trail_pipeline = if cursor_trail.is_empty() {
        None
    } else {
        Some(surface_cache.shared_legacy_cursor_trail_pipeline(format, LegacyPipelineLoad::Load)
            .ok_or_else(|| VulkanSurfaceCompositionError::Pipeline(
                "The compatibility cursor-trail pipeline was not cached".to_owned(),
            ))?)
    };

    // Cache-owned textures are borrowed only after all asynchronous uploads have
    // completed. The frame pool and every binding stay alive through submission.
    let descriptor_layout = surface_cache.descriptor_layout();
    let sampler = surface_cache.sampler();
    let surface_views = visible
        .iter()
        .map(|surface| {
            surface_cache.surface_texture(&surface.surface_id)
                .map(|texture| texture.view())
                .ok_or_else(|| VulkanSurfaceCompositionError::Texture(
                    "A visible surface texture was not present in the shared cache".to_owned(),
                ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let wallpaper_view = if wallpaper_ready {
        Some(wallpaper_asset_id
            .and_then(|asset_id| surface_cache.wallpaper_texture_for_asset(asset_id))
            .ok_or_else(|| VulkanSurfaceCompositionError::Texture(
                "The selected wallpaper texture disappeared before legacy binding".to_owned(),
            ))?
            .view())
    } else {
        None
    };
    let context_menu_view = if context_menu_ready {
        let key = context_menu_key.expect("ready context menu has a cache key");
        Some(surface_cache.desktop_icon_texture(key)
            .ok_or_else(|| VulkanSurfaceCompositionError::Texture(
                "The context-menu texture disappeared before legacy binding".to_owned(),
            ))?
            .view())
    } else {
        None
    };
    let cursor_view = if cursor_texture_ready {
        Some(surface_cache.native_cursor_asset(
            cursor_name, scene.context_menu_theme(), requested_cursor_size,
        ).map(|asset| asset.frame_at(cursor_frame_time).texture().view())
            .ok_or_else(|| VulkanSurfaceCompositionError::Texture(
                "The selected native cursor texture was not ready for legacy binding".to_owned(),
            ))?)
    } else {
        None
    };

    let binding_count = visible.len()
        + usize::from(wallpaper_view.is_some())
        + usize::from(context_menu_view.is_some())
        + usize::from(cursor_view.is_some());
    let binding_pool = if binding_count > 0 {
        Some(surface_cache.create_texture_binding_pool(binding_count as u32)
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?)
    } else {
        None
    };
    let pool = binding_pool.as_ref();

    let surface_bindings = surface_views
        .iter()
        .map(|source| {
            VulkanTextureBinding::new_in_pool(
                device,
                pool.expect("visible surfaces require a descriptor pool"),
                descriptor_layout,
                source,
                sampler,
            )
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let wallpaper_binding = wallpaper_view.as_ref().map(|source| {
        VulkanTextureBinding::new_in_pool(
            device, pool.expect("wallpaper requires a descriptor pool"),
            descriptor_layout, source, sampler,
        ).map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
    }).transpose()?;
    let context_menu_binding = context_menu_view.as_ref().map(|source| {
        VulkanTextureBinding::new_in_pool(
            device, pool.expect("context menu requires a descriptor pool"),
            descriptor_layout, source, sampler,
        ).map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
    }).transpose()?;
    let cursor_binding = cursor_view.as_ref().map(|source| {
        VulkanTextureBinding::new_in_pool(
            device, pool.expect("native cursor requires a descriptor pool"),
            descriptor_layout, source, sampler,
        ).map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
    }).transpose()?;

    let wallpaper_layer = if let (Some(source), Some(binding), Some(pipeline)) = (
        wallpaper_view.as_ref(), wallpaper_binding.as_ref(), wallpaper_pipeline,
    ) {
        let texture = wallpaper_asset_id
            .and_then(|asset_id| surface_cache.wallpaper_texture_for_asset(asset_id))
            .ok_or_else(|| VulkanSurfaceCompositionError::Texture(
                "Wallpaper texture was lost while resolving its geometry".to_owned(),
            ))?;
        let layout = WallpaperLayout::calculate(
            texture.width(), texture.height(), target.width(), target.height(),
            WallpaperFitMode::Fill,
        ).ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;
        WallpaperDrawGeometry::for_output(
            layout, texture.width(), texture.height(), target.width(), target.height(),
        ).map(|geometry| {
            let destination = geometry.destination();
            let uv = geometry.uv();
            LegacyWallpaperLayer {
                source,
                binding,
                pipeline,
                render_pass: &load_render_pass,
                viewport: vk::Viewport {
                    x: destination.x() as f32,
                    y: destination.y() as f32,
                    width: destination.width() as f32,
                    height: destination.height() as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                },
                scissor: vk::Rect2D {
                    offset: vk::Offset2D {
                        x: i32::try_from(destination.x()).unwrap_or(0),
                        y: i32::try_from(destination.y()).unwrap_or(0),
                    },
                    extent: vk::Extent2D {
                        width: destination.width(),
                        height: destination.height(),
                    },
                },
                uv_transform: [uv.min()[0], uv.min()[1], uv.max()[0], uv.max()[1]],
            }
        })
    } else {
        None
    };

    let context_menu_layer = match (
        context_menu_view.as_ref(), context_menu_binding.as_ref(), context_menu_draw,
    ) {
        (Some(source), Some(binding), Some((viewport, scissor, texture_transform))) => {
            Some(LegacyContextMenuLayer {
                source, binding, viewport, scissor, texture_transform,
            })
        }
        _ => None,
    };
    let cursor_trail_layer = match (
        cursor_view.as_ref(), cursor_binding.as_ref(), trail_pipeline,
    ) {
        (Some(source), Some(binding), Some(pipeline)) if !cursor_trail.is_empty() => {
            Some(LegacyCursorTrailLayer {
                source, binding, pipeline, impressions: cursor_trail,
            })
        }
        _ => None,
    };
    let native_cursor_layer = match (
        cursor_view.as_ref(), cursor_binding.as_ref(), cursor,
    ) {
        (Some(source), Some(binding), Some((geometry, scissor, texture_transform))) => {
            Some(LegacyNativeCursorLayer {
                source, binding,
                viewport: vk::Viewport {
                    x: geometry.x() as f32,
                    y: geometry.y() as f32,
                    width: geometry.width() as f32,
                    height: geometry.height() as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                },
                scissor: vk::Rect2D {
                    offset: vk::Offset2D {
                        x: i32::try_from(scissor.x)
                            .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
                        y: i32::try_from(scissor.y)
                            .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
                    },
                    extent: vk::Extent2D { width: scissor.width, height: scissor.height },
                },
                texture_transform,
            })
        }
        _ => None,
    };

    let layers = LegacyFrameLayers {
        clear_render_pass: &clear_render_pass,
        load_render_pass: &load_render_pass,
        target: composition_view,
        clear_color: scene.desktop().surface().background_color(),
        visible,
        surface_views: &surface_views,
        surface_bindings: &surface_bindings,
        surface_pipeline,
        wallpaper: wallpaper_layer,
        context_menu: context_menu_layer,
        cursor_trail: cursor_trail_layer,
        native_cursor: native_cursor_layer,
    };

    let legacy_display_pass: Option<Arc<VulkanSharedLegacyDisplayColorPass>> =
        if display_warmth.is_some() {
            Some(surface_cache.legacy_display_color_pass(output_view.format())
                .map_err(|error: VulkanDisplayColorPassError| {
                    VulkanSurfaceCompositionError::Pipeline(error.to_string())
                })?)
        } else {
            None
        };
    let display_binding = legacy_display_pass.as_ref().map(|pass| {
        pass.bind_source(composition_view)
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
    }).transpose()?;

    let mut command_buffer = VulkanCommandBuffer::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    legacy_layers::record_legacy_frame_layers(
        command_buffer.handle(), layers, previous_composition_state,
    )?;

    if let (Some(pass), Some(binding), Some(warmth)) = (
        legacy_display_pass.as_ref(), display_binding.as_ref(), display_warmth,
    ) {
        VulkanLegacyFrameRecorder::display_color(
            pass,
            device,
            command_buffer.handle(),
            composition_view,
            output_view,
            binding,
            VulkanImageState::General,
            previous_composition_state,
            VulkanImageState::General,
            warmth,
        ).map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    if let Some(readback) = readback {
        readback.record_transition_and_copy_rgba8(
            device, command_buffer.handle(), output_view, VulkanImageState::General,
        ).map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
        VulkanLegacyFrameRecorder::submit_readback(
            &mut command_buffer, device, output_view, on_submitted,
        ).map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    } else {
        VulkanLegacyFrameRecorder::release_and_submit(
            &mut command_buffer, device, output_view, VulkanImageState::General, on_submitted,
        ).map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    Ok(())
}
