// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! SHM surface composition into a Vulkan DRM scanout image.

use std::{error::Error, fmt};

use smithay::{reexports::ash::vk, utils::Transform};

use std::sync::Arc;

use crate::{
    render::{
        RenderScene, geometry::surface_geometry, shm_buffer::ShmBufferSnapshot,
        surface_tree::SurfaceTreeSnapshot,
    },
    renderer::RenderTarget,
};

use super::{
    VulkanClearPass, VulkanColorLoad, VulkanCommandBuffer, VulkanDisplayColorPass, VulkanDrawPass,
    VulkanImageState, VulkanImageTransition, VulkanImageView, VulkanLogicalDevice,
    VulkanSurfacePipelineCache, VulkanTextureBinding,
};

const BACKGROUND_COLOR: [f32; 4] = [0.08, 0.18, 0.32, 1.0];
const MAX_COMPOSITED_SURFACES: usize = 128;
const MAX_COMPOSITED_SHM_BYTES: usize = 256 * 1024 * 1024;

/// Errors raised while preparing or recording a SHM-composited frame.
#[derive(Debug)]
pub(crate) enum VulkanSurfaceCompositionError {
    Pipeline(String),
    Texture(String),
    Descriptor(String),
    Command(String),
    UnsupportedTransform,
    InvalidSurfaceGeometry,
    TooManySurfaces,
    FrameMemoryLimitExceeded,
}

impl fmt::Display for VulkanSurfaceCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pipeline(error) => write!(formatter, "Surface pipeline initialization failed: {error}"),
            Self::Texture(error) => write!(formatter, "Wayland SHM texture upload failed: {error}"),
            Self::Descriptor(error) => write!(formatter, "Surface texture descriptor creation failed: {error}"),
            Self::Command(error) => write!(formatter, "Surface composition command failed: {error}"),
            Self::UnsupportedTransform => formatter.write_str("Non-normal Wayland buffer transforms are not supported by the initial SHM compositor"),
            Self::InvalidSurfaceGeometry => formatter.write_str("A visible Wayland surface has invalid physical geometry"),
            Self::TooManySurfaces => formatter.write_str("The frame exceeds the configured visible-surface limit"),
            Self::FrameMemoryLimitExceeded => formatter.write_str("The frame exceeds the configured aggregate SHM texture memory limit"),
        }
    }
}

impl Error for VulkanSurfaceCompositionError {}

struct VisibleSurface {
    surface_id: smithay::reexports::wayland_server::backend::ObjectId,
    revision: u64,
    snapshot: Arc<ShmBufferSnapshot>,
    geometry: crate::render::geometry::SurfaceGeometry,
    scissor: crate::render::geometry::SurfaceScissor,
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
    let active_surface_ids = scene.surface_ids();
    let snapshots = scene.shm_snapshots();
    let visible = visible_surfaces(snapshots, target)?;
    let cursor = scene.native_cursor_location().and_then(|location| {
        let scale = scene.native_cursor_scale();
        let hotspot = crate::render::cursor::DEFAULT_CURSOR_HOTSPOT;
        let scaled_width =
            (crate::render::cursor::DEFAULT_CURSOR_WIDTH as f64 * scale).round() as u32;
        let scaled_height =
            (crate::render::cursor::DEFAULT_CURSOR_HEIGHT as f64 * scale).round() as u32;
        let scaled_location = (
            (f64::from(location.x) + f64::from(hotspot.0) * (1.0 - scale)).round() as i32,
            (f64::from(location.y) + f64::from(hotspot.1) * (1.0 - scale)).round() as i32,
        );
        let geometry = surface_geometry(
            scaled_location,
            (scaled_width, scaled_height),
            1,
            Transform::Normal,
            target.logical_origin(),
            target.scale(),
        )?;
        let scissor = geometry.scissor(target.width(), target.height())?;
        Some((geometry, scissor))
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
        )
        .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    }

    if let (Some((geometry, scissor)), Some(binding)) = (cursor, cursor_binding.as_ref()) {
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

    VulkanImageTransition::record_release_to_foreign(
        device,
        command_buffer.handle(),
        view.image_handle(),
        VulkanImageState::General,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    // All sampled texture uploads are complete before the draw submission.
    command_buffer
        .submit_and_wait_with(on_submitted)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}

fn clear_display_and_release(
    device: &VulkanLogicalDevice,
    target_view: &VulkanImageView<'_, '_>,
    previous_target_state: VulkanImageState,
    pass: &VulkanDisplayColorPass<'_>,
    warmth: u8,
    intermediate_view: &VulkanImageView<'_, '_>,
    previous_intermediate_state: VulkanImageState,
    on_submitted: impl FnOnce(),
) -> Result<(), VulkanSurfaceCompositionError> {
    let binding = pass
        .bind_source(intermediate_view)
        .map_err(|error| VulkanSurfaceCompositionError::Descriptor(error.to_string()))?;
    let mut command_buffer = VulkanCommandBuffer::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    VulkanClearPass::record_color(
        command_buffer.handle(),
        intermediate_view,
        previous_intermediate_state,
        VulkanImageState::General,
        BACKGROUND_COLOR,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    pass.record(
        device,
        command_buffer.handle(),
        intermediate_view,
        target_view,
        &binding,
        VulkanImageState::General,
        previous_target_state,
        VulkanImageState::General,
        warmth,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    VulkanImageTransition::record_release_to_foreign(
        device,
        command_buffer.handle(),
        target_view.image_handle(),
        VulkanImageState::General,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .submit_and_wait_with(on_submitted)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}

fn clear_and_release(
    device: &VulkanLogicalDevice,
    view: &VulkanImageView<'_, '_>,
    previous_image_state: VulkanImageState,
    on_submitted: impl FnOnce(),
) -> Result<(), VulkanSurfaceCompositionError> {
    let mut command_buffer = VulkanCommandBuffer::new(device)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    VulkanClearPass::record_color(
        command_buffer.handle(),
        view,
        previous_image_state,
        VulkanImageState::General,
        BACKGROUND_COLOR,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;
    VulkanImageTransition::record_release_to_foreign(
        device,
        command_buffer.handle(),
        view.image_handle(),
        VulkanImageState::General,
    )
    .map_err(|error| VulkanSurfaceCompositionError::Command(error.to_string()))?;

    command_buffer
        .end()
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;
    command_buffer
        .submit_and_wait_with(on_submitted)
        .map_err(|error| VulkanSurfaceCompositionError::Command(format!("{error:?}")))?;

    Ok(())
}

fn visible_surfaces(
    snapshots: Vec<SurfaceTreeSnapshot>,
    target: &RenderTarget,
) -> Result<Vec<VisibleSurface>, VulkanSurfaceCompositionError> {
    let mut visible = Vec::new();
    let mut total_bytes = 0usize;

    for snapshot in snapshots {
        let geometry = surface_geometry(
            (snapshot.location.x, snapshot.location.y),
            (snapshot.snapshot.width(), snapshot.snapshot.height()),
            snapshot.buffer_scale,
            snapshot.buffer_transform,
            target.logical_origin(),
            target.scale(),
        )
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;

        let Some(scissor) = geometry.scissor(target.width(), target.height()) else {
            continue;
        };

        if snapshot.buffer_transform != Transform::Normal {
            return Err(VulkanSurfaceCompositionError::UnsupportedTransform);
        }

        if visible.len() >= MAX_COMPOSITED_SURFACES {
            return Err(VulkanSurfaceCompositionError::TooManySurfaces);
        }
        total_bytes = total_bytes
            .checked_add(snapshot.snapshot.pixels().len())
            .ok_or(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded)?;
        if total_bytes > MAX_COMPOSITED_SHM_BYTES {
            return Err(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded);
        }

        visible.push(VisibleSurface {
            surface_id: snapshot.surface_id,
            revision: snapshot.revision,
            snapshot: Arc::clone(&snapshot.snapshot),
            geometry,
            scissor,
        });
    }

    Ok(visible)
}
