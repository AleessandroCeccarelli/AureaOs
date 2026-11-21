// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! SHM surface composition into a Vulkan DRM scanout image.

mod error;
mod release;
mod readback;
mod visible_surfaces;

pub(crate) use error::VulkanSurfaceCompositionError;
use release::{clear_and_release, clear_display_and_release};
use readback::clear_and_readback;
use visible_surfaces::visible_surfaces;

use std::sync::Arc;

use smithay::{reexports::ash::vk, utils::Transform};

use crate::{
    render::{
        RenderScene,
        cursor::scaled_cursor_origin,
        geometry::{
            compose_surface_transforms, surface_geometry, surface_geometry_with_fractional_scale,
            surface_transform_index, transform_geometry_for_output,
        },
    },
    renderer::RenderTarget,
};

use super::{
    VulkanClearPass, VulkanColorLoad, VulkanCommandBuffer, VulkanDisplayColorPass, VulkanDrawPass,
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanLogicalDevice,
    VulkanReadbackBuffer, VulkanSurfacePipelineCache, VulkanTextureBinding,
};

pub(super) const BACKGROUND_COLOR: [f32; 4] = [0.08, 0.18, 0.32, 1.0];
pub(super) const MAX_COMPOSITED_SURFACES: usize = 128;
pub(super) const MAX_COMPOSITED_SHM_BYTES: usize = 256 * 1024 * 1024;

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
        device, view, target, scene, surface_cache, previous_image_state,
        display_color, None, on_submitted,
    )
}

/// Composes the same Wayland scene as the direct path, then copies RGBA8 pixels
/// to host-visible memory instead of transferring image ownership to KMS.
pub(crate) fn render_and_readback(
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
    readback: &VulkanReadbackBuffer<'_>,
) -> Result<(), VulkanSurfaceCompositionError> {
    render_frame(
        device, view, target, scene, surface_cache, previous_image_state,
        display_color, Some(readback), || {},
    )
}

fn render_frame(
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
    let active_surface_ids = scene.surface_ids();
    let snapshots = scene.shm_snapshots();
    let visible = visible_surfaces(snapshots, target)?;
    let cursor = scene.native_cursor_location().and_then(|location| {
        let scale = scene.native_cursor_scale();
        let scaled_width =
            (crate::render::cursor::DEFAULT_CURSOR_WIDTH as f64 * scale).round() as u32;
        let scaled_height =
            (crate::render::cursor::DEFAULT_CURSOR_HEIGHT as f64 * scale).round() as u32;
        let scaled_location = scaled_cursor_origin(location, scale).into();
        let geometry = if let Some(scale) = target.fractional_scale() {
            surface_geometry_with_fractional_scale(
                scaled_location,
                (scaled_width, scaled_height),
                1,
                Transform::Normal,
                target.logical_origin(),
                scale,
            )?
        } else {
            surface_geometry(
                scaled_location,
                (scaled_width, scaled_height),
                1,
                Transform::Normal,
                target.logical_origin(),
                target.scale(),
            )?
        };
        let geometry = transform_geometry_for_output(
            geometry,
            target.width(),
            target.height(),
            target.transform(),
        )?;
        let scissor = geometry.scissor(target.width(), target.height())?;
        let texture_transform = surface_transform_index(compose_surface_transforms(
            Transform::Normal,
            target.transform(),
        )?)?;
        Some((geometry, scissor, texture_transform))
    });
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
        .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))?;

    if visible.is_empty() && cursor.is_none() {
        surface_cache
            .complete_surface_texture_uploads()
            .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))?;
        if let Some(readback) = readback {
            if display_color.is_some() {
                return Err(VulkanSurfaceCompositionError::Command(
                    "Readback clear with display-color pass is not supported yet".into(),
                ));
            }
            return clear_and_readback(
                device, view, previous_image_state, readback,
            );
        }
        return match display_color {
            Some((pass, warmth, intermediate_view, intermediate_state)) => {
                clear_display_and_release(
                    device, view, previous_image_state, pass, warmth,
                    intermediate_view, intermediate_state, on_submitted,
                )
            }
            None => clear_and_release(device, view, previous_image_state, on_submitted),
        };
    }
    // Finish asynchronous texture uploads before borrowing cache-owned GPU resources.
    // This keeps upload retirement separate from the immutable descriptor/pipeline borrows.
    surface_cache
        .complete_surface_texture_uploads()
        .map_err(|error| VulkanSurfaceCompositionError::Texture(error.to_string()))?;

    let (composition_view, composition_previous_state) = match display_color {
        Some((_, _, intermediate_view, intermediate_state)) => {
            (intermediate_view, intermediate_state)
        }
        None => (view, previous_image_state),
    };

    if cursor.is_some() {
        surface_cache
            .ensure_native_cursor_texture()
            .map_err(VulkanSurfaceCompositionError::Texture)?;
    }

    surface_cache
        .ensure_pipeline(composition_view.format())
        .map_err(|error| VulkanSurfaceCompositionError::Pipeline(error.to_string()))?;
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
        .create_texture_binding_pool(visible.len() as u32 + u32::from(cursor.is_some()))
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

    let cursor_binding = if cursor.is_some() {
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
        BACKGROUND_COLOR,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

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
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}
