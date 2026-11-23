// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Collects visible Wayland surfaces and prepares their output geometry.

use std::sync::Arc;

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

use super::{MAX_COMPOSITED_SHM_BYTES, MAX_COMPOSITED_SURFACES, VulkanSurfaceCompositionError};

pub(super) struct VisibleSurface {
    pub(super) surface_id: smithay::reexports::wayland_server::backend::ObjectId,
    pub(super) revision: u64,
    pub(super) snapshot: Arc<ShmBufferSnapshot>,
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
        if total_bytes > MAX_COMPOSITED_SHM_BYTES {
            return Err(VulkanSurfaceCompositionError::FrameMemoryLimitExceeded);
        }

        visible.push(VisibleSurface {
            surface_id: snapshot.surface_id,
            revision: snapshot.revision,
            snapshot: Arc::clone(&snapshot.snapshot),
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
