// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-relative geometry for Wayland surface buffers.

use smithay::utils::Transform;

use crate::output::scale::{SCALE_DENOMINATOR, ScaleFactor};

/// Physical destination rectangle for one surface on one output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceGeometry {
    x: i64,
    y: i64,
    width: u32,
    height: u32,
}

/// Integer scissor rectangle clipped to a physical output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceScissor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SurfaceGeometry {
    pub fn x(self) -> i64 {
        self.x
    }

    pub fn y(self) -> i64 {
        self.y
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }

    /// Intersects this destination rectangle with the physical output bounds.
    ///
    /// The original destination remains unchanged so a renderer can preserve
    /// the full texture mapping while using the returned rectangle as scissor.
    pub fn scissor(self, output_width: u32, output_height: u32) -> Option<SurfaceScissor> {
        if output_width == 0 || output_height == 0 {
            return None;
        }

        let left = self.x.max(0);
        let top = self.y.max(0);
        let right = self
            .x
            .checked_add(i64::from(self.width))?
            .min(i64::from(output_width));
        let bottom = self
            .y
            .checked_add(i64::from(self.height))?
            .min(i64::from(output_height));

        if right <= left || bottom <= top {
            return None;
        }

        Some(SurfaceScissor {
            x: u32::try_from(left).ok()?,
            y: u32::try_from(top).ok()?,
            width: u32::try_from(right - left).ok()?,
            height: u32::try_from(bottom - top).ok()?,
        })
    }
}


/// Maps surface-local normalized coordinates to buffer texture coordinates.
///
/// Wayland defines the buffer transform as a transform already applied by
/// the client. Sampling therefore applies its inverse. Coordinates use a
/// top-left origin and are normalized to the inclusive [0, 1] range.
pub fn buffer_uv_from_surface_uv(
    transform: Transform,
    uv: (f32, f32),
) -> Option<(f32, f32)> {
    let (u, v) = uv;
    if !u.is_finite() || !v.is_finite() {
        return None;
    }

    Some(match transform {
        Transform::Normal => (u, v),
        Transform::_90 => (v, 1.0 - u),
        Transform::_180 => (1.0 - u, 1.0 - v),
        Transform::_270 => (1.0 - v, u),
        Transform::Flipped => (1.0 - u, v),
        Transform::Flipped90 => (v, u),
        Transform::Flipped180 => (u, 1.0 - v),
        Transform::Flipped270 => (1.0 - v, 1.0 - u),
        _ => return None,
    })
}

/// Combines a client buffer transform with the compositor output transform.
pub fn compose_surface_transforms(
    buffer_transform: Transform,
    output_transform: Transform,
) -> Option<Transform> {
    let candidates = [
        Transform::Normal,
        Transform::_90,
        Transform::_180,
        Transform::_270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ];
    let probes = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];

    candidates.into_iter().find(|candidate| {
        probes.into_iter().all(|uv| {
            let expected = buffer_uv_from_surface_uv(output_transform, uv)
                .and_then(|uv| buffer_uv_from_surface_uv(buffer_transform, uv));
            let actual = buffer_uv_from_surface_uv(*candidate, uv);
            actual == expected
        })
    })
}

/// Converts a supported Wayland transform to the shader's compact index.
pub fn surface_transform_index(transform: Transform) -> Option<u32> {
    Some(match transform {
        Transform::Normal => 0,
        Transform::_90 => 1,
        Transform::_180 => 2,
        Transform::_270 => 3,
        Transform::Flipped => 4,
        Transform::Flipped90 => 5,
        Transform::Flipped180 => 6,
        Transform::Flipped270 => 7,
        _ => return None,
    })
}

/// Maps an output-local rectangle through the configured physical orientation.
pub fn transform_surface_geometry(
    geometry: SurfaceGeometry,
    output_width: u32,
    output_height: u32,
    transform: Transform,
) -> Option<SurfaceGeometry> {
    let x = geometry.x;
    let y = geometry.y;
    let width = i64::from(geometry.width);
    let height = i64::from(geometry.height);
    let output_width = i64::from(output_width);
    let output_height = i64::from(output_height);

    let (x, y, width, height) = match transform {
        Transform::Normal => (x, y, width, height),
        Transform::_90 => (output_height.checked_sub(y.checked_add(height)?)?, x, height, width),
        Transform::_180 => (
            output_width.checked_sub(x.checked_add(width)?)?,
            output_height.checked_sub(y.checked_add(height)?)?,
            width,
            height,
        ),
        Transform::_270 => (y, output_width.checked_sub(x.checked_add(width)?)?, height, width),
        Transform::Flipped => (
            output_width.checked_sub(x.checked_add(width)?)?,
            y,
            width,
            height,
        ),
        Transform::Flipped90 => (y, x, height, width),
        Transform::Flipped180 => (
            x,
            output_height.checked_sub(y.checked_add(height)?)?,
            width,
            height,
        ),
        Transform::Flipped270 => (
            output_height.checked_sub(y.checked_add(height)?)?,
            output_width.checked_sub(x.checked_add(width)?)?,
            height,
            width,
        ),
        _ => return None,
    };

    Some(SurfaceGeometry {
        x,
        y,
        width: u32::try_from(width).ok()?,
        height: u32::try_from(height).ok()?,
    })
}

/// Applies the output's configured orientation to a surface rectangle.
///
/// The Vulkan scanout image keeps the DRM mode's native extent. For quarter
/// turns, the pre-transform coordinate space therefore has swapped dimensions.
pub fn transform_geometry_for_output(
    geometry: SurfaceGeometry,
    target_width: u32,
    target_height: u32,
    transform: Transform,
) -> Option<SurfaceGeometry> {
    if target_width == 0 || target_height == 0 {
        return None;
    }

    let swaps_dimensions = matches!(
        transform,
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
    );
    let (source_width, source_height) = if swaps_dimensions {
        (target_height, target_width)
    } else {
        (target_width, target_height)
    };

    transform_surface_geometry(geometry, source_width, source_height, transform)
}

/// Scales a destination rectangle around a global logical anchor point.
pub fn scale_surface_geometry(
    geometry: SurfaceGeometry,
    anchor: (i32, i32),
    output_origin: (i32, i32),
    output_scale: u32,
    scale: f64,
) -> Option<SurfaceGeometry> {
    scale_surface_geometry_with_ratio(
        geometry,
        anchor,
        output_origin,
        u64::from(output_scale),
        1,
        scale,
    )
}

/// Scales a surface around a global logical anchor using the output's exact
/// fractional scale, keeping cursor hotspots aligned on mixed-DPI desktops.
pub fn scale_surface_geometry_with_fractional_scale(
    geometry: SurfaceGeometry,
    anchor: (i32, i32),
    output_origin: (i32, i32),
    output_scale: ScaleFactor,
    scale: f64,
) -> Option<SurfaceGeometry> {
    scale_surface_geometry_with_ratio(
        geometry,
        anchor,
        output_origin,
        u64::from(output_scale.numerator()),
        u64::from(SCALE_DENOMINATOR),
        scale,
    )
}

fn scale_surface_geometry_with_ratio(
    geometry: SurfaceGeometry,
    anchor: (i32, i32),
    output_origin: (i32, i32),
    scale_numerator: u64,
    scale_denominator: u64,
    scale: f64,
) -> Option<SurfaceGeometry> {
    if scale_numerator == 0
        || scale_denominator == 0
        || !scale.is_finite()
        || !(1.0..=3.0).contains(&scale)
    {
        return None;
    }

    let anchor_x = round_ratio(
        i128::from(anchor.0.checked_sub(output_origin.0)?)
            .checked_mul(i128::from(scale_numerator))?,
        scale_denominator,
    )?;
    let anchor_y = round_ratio(
        i128::from(anchor.1.checked_sub(output_origin.1)?)
            .checked_mul(i128::from(scale_numerator))?,
        scale_denominator,
    )?;

    let x = (anchor_x as f64 + (geometry.x as f64 - anchor_x as f64) * scale).round();
    let y = (anchor_y as f64 + (geometry.y as f64 - anchor_y as f64) * scale).round();
    let width = (f64::from(geometry.width) * scale).round();
    let height = (f64::from(geometry.height) * scale).round();

    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || x < i64::MIN as f64
        || x > i64::MAX as f64
        || y < i64::MIN as f64
        || y > i64::MAX as f64
        || width < 1.0
        || width > u32::MAX as f64
        || height < 1.0
        || height > u32::MAX as f64
    {
        return None;
    }

    Some(SurfaceGeometry {
        x: x as i64,
        y: y as i64,
        width: width as u32,
        height: height as u32,
    })
}

/// Calculates a surface's physical destination rectangle for a target output.
///
/// Buffer dimensions are transformed and divided by the Wayland buffer scale
/// before the output scale is applied. Position is global logical space and is
/// translated by the target output's logical origin first.
pub fn surface_geometry(
    location: (i32, i32),
    buffer_size: (u32, u32),
    buffer_scale: i32,
    buffer_transform: Transform,
    output_origin: (i32, i32),
    output_scale: u32,
) -> Option<SurfaceGeometry> {
    surface_geometry_with_ratio(
        location,
        buffer_size,
        buffer_scale,
        buffer_transform,
        output_origin,
        u64::from(output_scale),
        1,
    )
}

/// Calculates surface geometry using an exact Wayland fractional output scale.
pub fn surface_geometry_with_fractional_scale(
    location: (i32, i32),
    buffer_size: (u32, u32),
    buffer_scale: i32,
    buffer_transform: Transform,
    output_origin: (i32, i32),
    output_scale: ScaleFactor,
) -> Option<SurfaceGeometry> {
    surface_geometry_with_ratio(
        location,
        buffer_size,
        buffer_scale,
        buffer_transform,
        output_origin,
        u64::from(output_scale.numerator()),
        u64::from(SCALE_DENOMINATOR),
    )
}

fn surface_geometry_with_ratio(
    location: (i32, i32),
    buffer_size: (u32, u32),
    buffer_scale: i32,
    buffer_transform: Transform,
    output_origin: (i32, i32),
    scale_numerator: u64,
    scale_denominator: u64,
) -> Option<SurfaceGeometry> {
    if buffer_scale <= 0
        || scale_numerator == 0
        || scale_denominator == 0
        || buffer_size.0 == 0
        || buffer_size.1 == 0
    {
        return None;
    }

    let buffer_width = i32::try_from(buffer_size.0).ok()?;
    let buffer_height = i32::try_from(buffer_size.1).ok()?;
    let swaps_dimensions = matches!(
        buffer_transform,
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
    );
    let (transformed_width, transformed_height) = if swaps_dimensions {
        (buffer_height, buffer_width)
    } else {
        (buffer_width, buffer_height)
    };

    let width = scaled_buffer_dimension(
        transformed_width,
        buffer_scale,
        scale_numerator,
        scale_denominator,
    )?;
    let height = scaled_buffer_dimension(
        transformed_height,
        buffer_scale,
        scale_numerator,
        scale_denominator,
    )?;
    if width == 0 || height == 0 {
        return None;
    }

    let relative_x = i128::from(location.0).checked_sub(i128::from(output_origin.0))?;
    let relative_y = i128::from(location.1).checked_sub(i128::from(output_origin.1))?;
    let x = round_ratio(relative_x.checked_mul(i128::from(scale_numerator))?, scale_denominator)?;
    let y = round_ratio(relative_y.checked_mul(i128::from(scale_numerator))?, scale_denominator)?;

    Some(SurfaceGeometry {
        x: i64::try_from(x).ok()?,
        y: i64::try_from(y).ok()?,
        width,
        height,
    })
}

fn scaled_buffer_dimension(
    dimension: i32,
    buffer_scale: i32,
    scale_numerator: u64,
    scale_denominator: u64,
) -> Option<u32> {
    if dimension <= 0 {
        return None;
    }

    let numerator = u128::try_from(dimension)
        .ok()?
        .checked_mul(u128::from(scale_numerator))?;
    let denominator = u128::try_from(buffer_scale)
        .ok()?
        .checked_mul(u128::from(scale_denominator))?;
    let rounded = numerator.checked_add(denominator / 2)? / denominator;
    u32::try_from(rounded).ok()
}

fn round_ratio(value: i128, denominator: u64) -> Option<i128> {
    let denominator = i128::from(denominator);
    let half = denominator / 2;
    if value >= 0 {
        value.checked_add(half).map(|adjusted| adjusted / denominator)
    } else {
        value.checked_sub(half).map(|adjusted| adjusted / denominator)
    }
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
