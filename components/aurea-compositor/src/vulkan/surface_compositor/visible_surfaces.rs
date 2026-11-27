// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Collects visible Wayland surfaces and prepares their output geometry.

use std::sync::Arc;

#[cfg(feature = "drm")]
use std::collections::HashSet;

use crate::{
    render::{
        geometry::{
            SurfaceGeometry, SurfaceScissor, compose_surface_transforms, scale_surface_geometry,
            scale_surface_geometry_with_fractional_scale, surface_geometry,
            surface_geometry_with_fractional_scale, surface_transform_index,
            transform_geometry_for_output,
        },
        shm_buffer::ShmBufferSnapshot,
        surface_tree::SurfaceTreeSnapshot,
    },
    renderer::RenderTarget,
};

use super::{MAX_COMPOSITED_BUFFER_BYTES, MAX_COMPOSITED_SURFACES, VulkanSurfaceCompositionError};

#[cfg(feature = "drm")]
use crate::render::surface_tree::SurfaceTreeDmabufSnapshot;

pub(super) struct VisibleSurface {
    pub(super) surface_id: smithay::reexports::wayland_server::backend::ObjectId,
    pub(super) revision: u64,
    pub(super) snapshot: Arc<ShmBufferSnapshot>,
    pub(super) is_cursor: bool,
    pub(super) geometry: SurfaceGeometry,
    pub(super) scissor: SurfaceScissor,
    pub(super) texture_transform: u32,
}

pub(super) fn visible_surfaces(
    snapshots: Vec<SurfaceTreeSnapshot>,
    target: &RenderTarget,
) -> Result<Vec<VisibleSurface>, VulkanSurfaceCompositionError> {
    let mut visible = Vec::new();
    let mut total_bytes = 0usize;

    for snapshot in snapshots {
        let mut geometry = if let Some(scale) = target.fractional_scale() {
            surface_geometry_with_fractional_scale(
                (snapshot.location.x, snapshot.location.y),
                (snapshot.snapshot.width(), snapshot.snapshot.height()),
                snapshot.buffer_scale,
                snapshot.buffer_transform,
                target.logical_origin(),
                scale,
            )
        } else {
            surface_geometry(
                (snapshot.location.x, snapshot.location.y),
                (snapshot.snapshot.width(), snapshot.snapshot.height()),
                snapshot.buffer_scale,
                snapshot.buffer_transform,
                target.logical_origin(),
                target.scale(),
            )
        }
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;

        if let Some(anchor) = snapshot.cursor_anchor {
            geometry = if let Some(scale) = target.fractional_scale() {
                scale_surface_geometry_with_fractional_scale(
                    geometry,
                    (anchor.x, anchor.y),
                    target.logical_origin(),
                    scale,
                    snapshot.cursor_scale,
                )
            } else {
                scale_surface_geometry(
                    geometry,
                    (anchor.x, anchor.y),
                    target.logical_origin(),
                    target.scale(),
                    snapshot.cursor_scale,
                )
            }
            .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;
        }

        geometry = transform_geometry_for_output(
            geometry,
            target.width(),
            target.height(),
            target.transform(),
        )
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;

        let Some(scissor) = geometry.scissor(target.width(), target.height()) else {
            continue;
        };

        if visible.len() >= MAX_COMPOSITED_SURFACES {
            return Err(VulkanSurfaceCompositionError::TooManySurfaces);
        }
        total_bytes = total_bytes
            .checked_add(snapshot.snapshot.pixels().len())
            .ok_or(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded)?;
        if total_bytes > MAX_COMPOSITED_BUFFER_BYTES {
            return Err(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded);
        }

        visible.push(VisibleSurface {
            surface_id: snapshot.surface_id,
            revision: snapshot.revision,
            snapshot: Arc::clone(&snapshot.snapshot),
            is_cursor: snapshot.cursor_anchor.is_some(),
            geometry,
            scissor,
            texture_transform: surface_transform_index(
                compose_surface_transforms(snapshot.buffer_transform, target.transform())
                    .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
            )
            .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
        });
    }

    Ok(visible)
}


/// Geometry for one visible DMA-BUF surface, independent of its Vulkan image import.
#[cfg(feature = "drm")]
pub(super) struct VisibleDmabufSurface {
    /// Index into the frame-local unique imported-image array.
    pub(super) image_index: usize,
    pub(super) snapshot: SurfaceTreeDmabufSnapshot,
    pub(super) geometry: SurfaceGeometry,
    pub(super) scissor: SurfaceScissor,
    pub(super) texture_transform: u32,
    pub(super) is_cursor: bool,
}

/// Resolves output clipping and Wayland buffer transforms for DMA-BUF snapshots.
#[cfg(feature = "drm")]
pub(super) fn visible_dmabuf_surfaces(
    snapshots: Vec<SurfaceTreeDmabufSnapshot>,
    target: &RenderTarget,
) -> Result<Vec<VisibleDmabufSurface>, VulkanSurfaceCompositionError> {
    let mut visible = Vec::new();
    let mut total_bytes = 0usize;
    let mut buffer_ids = HashSet::new();
    let mut image_indices = std::collections::HashMap::new();
    for snapshot in snapshots {
        // The first importer supports one packed memory plane. Bound the
        // retained/imported working set just as we do for copied SHM surfaces.
        let plane = snapshot
            .layout
            .planes()
            .first()
            .ok_or(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded)?;
        let plane_bytes = usize::try_from(plane.stride)
            .ok()
            .and_then(|stride| stride.checked_mul(snapshot.layout.height() as usize))
            .and_then(|bytes| bytes.checked_add(plane.offset as usize))
            .ok_or(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded)?;
        let buffer_id = snapshot.buffer.id();
        if buffer_ids.insert(buffer_id.clone()) {
            total_bytes = total_bytes
                .checked_add(plane_bytes)
                .ok_or(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded)?;
            if total_bytes > MAX_COMPOSITED_BUFFER_BYTES {
                return Err(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded);
            }
            image_indices.insert(buffer_id.clone(), image_indices.len());
        }
        let image_index = *image_indices
            .get(&buffer_id)
            .ok_or(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded)?;

        let geometry = if let Some(scale) = target.fractional_scale() {
            surface_geometry_with_fractional_scale(
                (snapshot.location.x, snapshot.location.y),
                (snapshot.layout.width(), snapshot.layout.height()),
                snapshot.buffer_scale,
                snapshot.buffer_transform,
                target.logical_origin(),
                scale,
            )
        } else {
            surface_geometry(
                (snapshot.location.x, snapshot.location.y),
                (snapshot.layout.width(), snapshot.layout.height()),
                snapshot.buffer_scale,
                snapshot.buffer_transform,
                target.logical_origin(),
                target.scale(),
            )
        }
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;
        let geometry = transform_geometry_for_output(
            geometry,
            target.width(),
            target.height(),
            target.transform(),
        )
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;
        let Some(scissor) = geometry.scissor(target.width(), target.height()) else {
            continue;
        };
        if visible.len() >= MAX_COMPOSITED_SURFACES {
            return Err(VulkanSurfaceCompositionError::TooManySurfaces);
        }
        // linux-dmabuf's y_inverted flag describes the pixel orientation
        // independently from wl_surface.buffer_transform. Convert it to the
        // equivalent vertical-flip transform before composing output rotation.
        let buffer_transform = if snapshot.layout.y_inverted() {
            compose_surface_transforms(
                snapshot.buffer_transform,
                smithay::utils::Transform::Flipped180,
            )
            .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?
        } else {
            snapshot.buffer_transform
        };
        let texture_transform = surface_transform_index(
            compose_surface_transforms(buffer_transform, target.transform())
                .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?,
        )
        .ok_or(VulkanSurfaceCompositionError::InvalidSurfaceGeometry)?;
        let is_cursor = snapshot.is_cursor;
        visible.push(VisibleDmabufSurface {
            image_index,
            snapshot,
            geometry,
            scissor,
            texture_transform,
            is_cursor,
        });
    }
    Ok(visible)
}
