// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! SHM surface composition into a Vulkan DRM scanout image.

mod cursor_geometry;
mod error;
mod readback;
mod readback_entry;
mod release;
mod visible_surfaces;

pub(crate) use error::VulkanSurfaceCompositionError;
use readback::{clear_and_readback, clear_display_and_readback};
pub(crate) use readback_entry::render_and_readback;
use release::{clear_and_release, clear_display_and_release};
use visible_surfaces::visible_surfaces;

use std::sync::Arc;

use smithay::reexports::ash::vk;

use crate::{
    desktop::{DesktopContextMenuLayout, WallpaperDrawGeometry, WallpaperFitMode, WallpaperLayout},
    render::RenderScene,
    renderer::RenderTarget,
};

use cursor_geometry::native_cursor_geometry;

use super::{
    VulkanClearPass, VulkanColorLoad, VulkanCommandBuffer, VulkanDisplayColorPass, VulkanDrawPass,
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanLogicalDevice,
    VulkanReadbackBuffer, VulkanSurfacePipelineCache, VulkanTextureBinding,
    VulkanNativeCursorTextureError,
};

pub(super) const MAX_COMPOSITED_SURFACES: usize = 128;
pub(super) const MAX_COMPOSITED_SHM_BYTES: usize = 256 * 1024 * 1024;

/// Converts global logical menu bounds to an output-local physical viewport
/// and clips the 256px artwork canvas to this output.
fn context_menu_geometry(
    menu: &DesktopContextMenuLayout,
    target: &RenderTarget,
) -> Option<(vk::Viewport, vk::Rect2D)> {
    let origin = menu.placement().bounds().origin();
    let (output_x, output_y) = target.logical_origin();
    let scale = target.effective_scale();
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }

    let x = (i64::from(origin.x) - i64::from(output_x)) as f64 * scale;
    let y = (i64::from(origin.y) - i64::from(output_y)) as f64 * scale;
    let width = 256.0 * scale;
    let height = 256.0 * scale;
    let left = x.floor().max(0.0) as u32;
    let top = y.floor().max(0.0) as u32;
    let right = (x + width).ceil().min(f64::from(target.width())) as u32;
    let bottom = (y + height).ceil().min(f64::from(target.height())) as u32;
    if left >= right || top >= bottom {
        return None;
    }

    Some((
        vk::Viewport {
            x: x as f32,
            y: y as f32,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        },
        vk::Rect2D {
            offset: vk::Offset2D {
                x: i32::try_from(left).ok()?,
                y: i32::try_from(top).ok()?,
            },
            extent: vk::Extent2D {
                width: right - left,
                height: bottom - top,
            },
        },
    ))
}

/// Records and submits one complete SHM-composited frame, releasing the
/// scanout image to the foreign KMS queue only after all sampled resources
/// have remained alive through GPU completion.
pub(crate) fn render_and_release(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    target: &RenderTarget,
    scene: &RenderScene<'_>,
    surface_cache: &mut VulkanSurfacePipelineCache,
    previous_image_state: VulkanImageState,
    display_color: Option<(
        &VulkanDisplayColorPass<'_>,
        u8,
        &VulkanImageView<'_, '_>,
        VulkanImageState,
    )>,
    on_submitted: impl FnOnce(),
) -> Result<(), VulkanSurfaceCompositionError> {
    render_frame(
        device,
        view,
        target,
        scene,
        surface_cache,
        previous_image_state,
        display_color,
        None,
        on_submitted,
    )
}

pub(super) fn render_frame(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    target: &RenderTarget,
    scene: &RenderScene<'_>,
    surface_cache: &mut VulkanSurfacePipelineCache,
    previous_image_state: VulkanImageState,
    display_color: Option<(
        &VulkanDisplayColorPass<'_>,
        u8,
        &VulkanImageView<'_, '_>,
        VulkanImageState,
    )>,
    readback: Option<&VulkanReadbackBuffer<'_>>,
    on_submitted: impl FnOnce(),
) -> Result<(), VulkanSurfaceCompositionError> {
    let desktop_snapshot = scene.desktop();
    let background_color = desktop_snapshot.surface().background_color();
    let wallpaper = desktop_snapshot.wallpaper();
    surface_cache
        .prepare_wallpaper_decode(wallpaper.map(|wallpaper| wallpaper.asset().clone()))
        .map_err(|error| match error {
            super::wallpaper_texture_cache::VulkanWallpaperTextureCacheError::Command(
                smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            ) => VulkanSurfaceCompositionError::DeviceLost(
                smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            ),
            other => VulkanSurfaceCompositionError::Texture(other.to_string()),
        })?;
    surface_cache
        .poll_wallpaper_texture_uploads()
        .map_err(|error| match error {
            super::wallpaper_texture_cache::VulkanWallpaperTextureCacheError::Command(
                smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            ) => VulkanSurfaceCompositionError::DeviceLost(
                smithay::reexports::ash::vk::Result::ERROR_DEVICE_LOST,
            ),
            other => VulkanSurfaceCompositionError::Texture(other.to_string()),
        })?;

    let context_menu_key = if scene.context_menu().is_some() {
        surface_cache
            .prepare_context_menu_texture(scene.context_menu_theme())
            .map_err(VulkanSurfaceCompositionError::Texture)?
    } else {
        None
    };
    surface_cache
        .poll_desktop_icon_texture_uploads()
        .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))?;
    let context_menu_draw = scene
        .context_menu()
        .and_then(|menu| context_menu_geometry(menu, target));
    let active_surface_ids = scene.surface_ids();
    let snapshots = scene.shm_snapshots();
    let visible = visible_surfaces(snapshots, target)?;
    let cursor = native_cursor_geometry(scene, target);
    let wallpaper_asset_id = scene.desktop().wallpaper().map(|wallpaper| wallpaper.asset().id());
    if wallpaper_asset_id.is_none() {
        surface_cache.clear_wallpaper_texture();
    }
    let texture_requests = visible
        .iter()
        .map(|surface| {
            (
                surface.surface_id.clone(),
                surface.revision,
                Arc::clone(&surface.snapshot),
            )
        })
        .collect::<Vec<_>>();

    surface_cache
        .prepare_surface_textures(&active_surface_ids, &texture_requests)
        .map_err(VulkanSurfaceCompositionError::from_surface_texture_cache_error)?;

    if visible.is_empty()
        && cursor.is_none()
        && context_menu_draw.is_none()
        && wallpaper_asset_id.is_none_or(|asset_id| {
            surface_cache.wallpaper_texture_for_asset(asset_id).is_none()
        })
    {
        surface_cache
            .complete_surface_texture_uploads()
            .map_err(VulkanSurfaceCompositionError::from_surface_texture_cache_error)?;
        if let Some(readback) = readback {
            return match display_color {
                Some((pass, warmth, intermediate_view, intermediate_state)) => {
                    clear_display_and_readback(
                        device,
                        view,
                        previous_image_state,
                        pass,
                        warmth,
                        intermediate_view,
                        intermediate_state,
                        readback,
                        background_color,
                    )
                }
                None => clear_and_readback(
                    device,
                    view,
                    previous_image_state,
                    readback,
                    background_color,
                ),
            };
        }
        return match display_color {
            Some((pass, warmth, intermediate_view, intermediate_state)) => {
                clear_display_and_release(
                    device,
                    view,
                    previous_image_state,
                    pass,
                    warmth,
                    intermediate_view,
                    intermediate_state,
                    on_submitted,
                    background_color,
                )
            }
            None => clear_and_release(
                device,
                view,
                previous_image_state,
                on_submitted,
                background_color,
            ),
        };
    }
    // Finish asynchronous texture uploads before borrowing cache-owned GPU resources.
    // This keeps upload retirement separate from the immutable descriptor/pipeline borrows.
    surface_cache
        .complete_surface_texture_uploads()
        .map_err(VulkanSurfaceCompositionError::from_surface_texture_cache_error)?;
    surface_cache
        .complete_desktop_icon_texture_uploads()
        .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))?;

    let (composition_view, composition_previous_state) = match display_color {
        Some((_, _, intermediate_view, intermediate_state)) => {
            (intermediate_view, intermediate_state)
        }
        None => (view, previous_image_state),
    };

    let cursor_texture_ready = if cursor.is_some() {
        surface_cache
            .ensure_native_cursor_texture()
            .map_err(|error| match error {
                super::surface_pipeline_cache::VulkanSurfacePipelineCacheError::NativeCursor(
                    VulkanNativeCursorTextureError::Texture(error),
                ) => VulkanSurfaceCompositionError::from_shm_texture_error(error),
                super::surface_pipeline_cache::VulkanSurfacePipelineCacheError::NativeCursor(
                    other,
                ) => VulkanSurfaceCompositionError::Texture(other.to_string()),
                other => VulkanSurfaceCompositionError::Texture(other.to_string()),
            })?
    } else {
        false
    };

    surface_cache
        .ensure_pipeline(composition_view.format())
        .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;
    let wallpaper_ready = wallpaper_asset_id.is_some_and(|asset_id| {
        surface_cache.wallpaper_texture_for_asset(asset_id).is_some()
    });
    let context_menu_ready = context_menu_key.is_some_and(|key| {
        surface_cache.desktop_icon_texture(key).is_some()
    });
    if wallpaper_ready {
        surface_cache
            .ensure_wallpaper_pipeline(composition_view.format())
            .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;
    }
    let descriptor_layout = surface_cache.descriptor_layout();
    let sampler = surface_cache.sampler();
    let pipeline = surface_cache
        .pipeline(composition_view.format())
        .ok_or_else(|| {
            VulkanSurfaceCompositionError::Pipeline(
                "The requested surface pipeline was not cached".to_owned(),
            )
        })?;

    // Cached images survive across frames. Views and descriptor sets remain
    // frame-scoped until submit_and_wait confirms GPU completion.
    let binding_pool = surface_cache
        .create_texture_binding_pool(
            visible.len() as u32
                + u32::from(cursor_texture_ready)
                + u32::from(wallpaper_ready)
                + u32::from(context_menu_ready),
        )
        .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?;
    let views = visible
        .iter()
        .map(|surface| {
            let texture = surface_cache
                .surface_texture(&surface.surface_id)
                .ok_or_else(|| {
                    VulkanSurfaceCompositionError::Texture(
                        "A visible surface texture was not prepared in the cache".to_owned(),
                    )
                })?;
            Ok(texture.view())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = views
        .iter()
        .map(|texture_view| {
            VulkanTextureBinding::new_in_pool(
                device,
                &binding_pool,
                &descriptor_layout,
                texture_view,
                &sampler,
            )
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let wallpaper_binding = if wallpaper_ready {
        let texture = wallpaper_asset_id
            .and_then(|asset_id| surface_cache.wallpaper_texture_for_asset(asset_id))
            .ok_or_else(|| {
            VulkanSurfaceCompositionError::Texture(
                "The selected wallpaper texture was not ready after upload polling".to_owned(),
            )
        })?;
        Some(
            VulkanTextureBinding::new_in_pool(
                device,
                &binding_pool,
                &descriptor_layout,
                texture.view(),
                &sampler,
            )
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?,
        )
    } else {
        None
    };

    let context_menu_binding = if context_menu_ready {
        let key = context_menu_key.expect("ready context menu has a texture key");
        let texture = surface_cache.desktop_icon_texture(key).ok_or_else(|| {
            VulkanSurfaceCompositionError::Texture(
                "The context-menu texture disappeared before binding".to_owned(),
            )
        })?;
        Some(
            VulkanTextureBinding::new_in_pool(
                device,
                &binding_pool,
                &descriptor_layout,
                texture.view(),
                &sampler,
            )
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?,
        )
    } else {
        None
    };

    let cursor_binding = if cursor_texture_ready {
        let texture = surface_cache.native_cursor_texture().ok_or_else(|| {
            VulkanSurfaceCompositionError::Texture(
                "The native cursor texture was not initialized".to_owned(),
            )
        })?;
        Some(
            VulkanTextureBinding::new_in_pool(
                device,
                &binding_pool,
                &descriptor_layout,
                texture.view(),
                &sampler,
            )
            .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?,
        )
    } else {
        None
    };

    let display_binding = if let Some((pass, _, intermediate_view, _)) = display_color {
        Some(
            pass.bind_source(intermediate_view)
                .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?,
        )
    } else {
        None
    };

    let mut command_buffer = VulkanCommandBuffer::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    VulkanClearPass::record_color(
        command_buffer.handle(),
        composition_view,
        composition_previous_state,
        VulkanImageState::General,
        background_color,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    if let (Some(texture), Some(binding)) =
        (
            wallpaper_asset_id.and_then(|asset_id| {
                surface_cache.wallpaper_texture_for_asset(asset_id)
            }),
            wallpaper_binding.as_ref(),
        )
    {
        let layout = WallpaperLayout::calculate(
            texture.width(),
            texture.height(),
            target.width(),
            target.height(),
            WallpaperFitMode::Fill,
        )
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;
        if let Some(geometry) = WallpaperDrawGeometry::for_output(
            layout,
            texture.width(),
            texture.height(),
            target.width(),
            target.height(),
        ) {
            let destination = geometry.destination();
            let viewport = vk::Viewport {
                x: destination.x() as f32,
                y: destination.y() as f32,
                width: destination.width() as f32,
                height: destination.height() as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            let scissor = vk::Rect2D {
                offset: vk::Offset2D {
                    x: i32::try_from(destination.x())
                        .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
                    y: i32::try_from(destination.y())
                        .map_err(|_| VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
                },
                extent: vk::Extent2D {
                    width: destination.width(),
                    height: destination.height(),
                },
            };
            let pipeline = surface_cache
                .wallpaper_pipeline(composition_view.format())
                .ok_or_else(|| {
                    VulkanSurfaceCompositionError::Pipeline(
                        "The requested wallpaper pipeline was not cached".to_owned(),
                    )
                })?;
            VulkanDrawPass::record_textured_region_with_push_constants(
                command_buffer.handle(),
                composition_view,
                pipeline,
                binding,
                0,
                VulkanImageState::General,
                VulkanImageState::General,
                VulkanColorLoad::Load,
                viewport,
                scissor,
                &geometry.uv().to_push_constant_bytes(),
            )
            .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
        }
    }

    for (surface, binding) in visible.iter().zip(bindings.iter()) {
        let geometry = surface.geometry;
        let scissor = surface.scissor;
        let viewport = vk::Viewport {
            x: geometry.x() as f32,
            y: geometry.y() as f32,
            width: geometry.width() as f32,
            height: geometry.height() as f32,
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

        VulkanDrawPass::record_textured_region(
            command_buffer.handle(),
            composition_view,
            &pipeline,
            binding,
            0,
            VulkanImageState::General,
            VulkanImageState::General,
            VulkanColorLoad::Load,
            viewport,
            scissor,
            surface.texture_transform,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    if let (Some((viewport, scissor)), Some(binding)) =
        (context_menu_draw, context_menu_binding.as_ref())
    {
        VulkanDrawPass::record_textured_region(
            command_buffer.handle(),
            composition_view,
            &pipeline,
            binding,
            0,
            VulkanImageState::General,
            VulkanImageState::General,
            VulkanColorLoad::Load,
            viewport,
            scissor,
            0,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    if let (Some((geometry, scissor, texture_transform)), Some(binding)) =
        (cursor, cursor_binding.as_ref())
    {
        let viewport = vk::Viewport {
            x: geometry.x() as f32,
            y: geometry.y() as f32,
            width: geometry.width() as f32,
            height: geometry.height() as f32,
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

        VulkanDrawPass::record_textured_region(
            command_buffer.handle(),
            composition_view,
            &pipeline,
            binding,
            0,
            VulkanImageState::General,
            VulkanImageState::General,
            VulkanColorLoad::Load,
            viewport,
            scissor,
            texture_transform,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    if let Some((pass, warmth, _, _)) = display_color {
        pass.record(
            device,
            command_buffer.handle(),
            composition_view,
            view,
            display_binding
                .as_ref()
                .expect("display binding created with pass"),
            VulkanImageState::General,
            previous_image_state,
            VulkanImageState::General,
            warmth,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    if let Some(readback) = readback {
        readback
            .record_transition_and_copy_rgba8(
                device,
                command_buffer.handle(),
                view,
                VulkanImageState::General,
            )
            .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    } else {
        VulkanImageTransition::record_release_to_foreign(
            device,
            command_buffer.handle(),
            view.image_handle(),
            VulkanImageState::General,
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    // All sampled texture uploads are complete before the draw submission.
    command_buffer
        .submit_and_wait_with(on_submitted)
        .map_err(VulkanSurfaceCompositionError::from_queue_submit_error)?;

    Ok(())
}
