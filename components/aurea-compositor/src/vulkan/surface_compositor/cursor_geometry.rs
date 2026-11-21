// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Computes the native cursor's physical geometry for one output.

use smithay::utils::Transform;

use crate::{
    render::{
        RenderScene,
        cursor::scaled_cursor_origin,
        geometry::{
            SurfaceGeometry, SurfaceScissor, compose_surface_transforms, surface_geometry,
            surface_geometry_with_fractional_scale, surface_transform_index,
            transform_geometry_for_output,
        },
    },
    renderer::RenderTarget,
};

/// Resolves the cursor rectangle, clipping and texture transform for an output.
pub(super) fn native_cursor_geometry(
    scene: &RenderScene<'_>,
    target: &RenderTarget,
) -> Option<(SurfaceGeometry, SurfaceScissor, u32)> {
    let location = scene.native_cursor_location()?;
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
        )
    } else {
        surface_geometry(
            scaled_location,
            (scaled_width, scaled_height),
            1,
            Transform::Normal,
            target.logical_origin(),
            target.scale(),
        )
    }?;
    let geometry = transform_geometry_for_output(
        geometry,
        target.width(),
        target.height(),
        target.transform(),
    )?;
    let scissor = geometry.scissor(target.width(), target.height())?;
    let texture_transform = surface_transform_index(
        compose_surface_transforms(Transform::Normal, target.transform())?,
    )?;

    Some((geometry, scissor, texture_transform))
}
