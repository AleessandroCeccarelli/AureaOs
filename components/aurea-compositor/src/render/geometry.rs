// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-relative geometry for Wayland surface buffers.

use smithay::utils::Transform;

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

/// Scales a destination rectangle around a global logical anchor point.
pub fn scale_surface_geometry(
    geometry: SurfaceGeometry,
    anchor: (i32, i32),
    output_origin: (i32, i32),
    output_scale: u32,
    scale: f64,
) -> Option<SurfaceGeometry> {
    if output_scale == 0 || !scale.is_finite() || !(1.0..=3.0).contains(&scale) {
        return None;
    }

    let anchor_x =
        i64::from(anchor.0.checked_sub(output_origin.0)?).checked_mul(i64::from(output_scale))?;
    let anchor_y =
        i64::from(anchor.1.checked_sub(output_origin.1)?).checked_mul(i64::from(output_scale))?;

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
    if buffer_scale <= 0 || output_scale == 0 || buffer_size.0 == 0 || buffer_size.1 == 0 {
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

    let width = scaled_buffer_dimension(transformed_width, buffer_scale, output_scale)?;
    let height = scaled_buffer_dimension(transformed_height, buffer_scale, output_scale)?;
    if width == 0 || height == 0 {
        return None;
    }

    let relative_x = i64::from(location.0).checked_sub(i64::from(output_origin.0))?;
    let relative_y = i64::from(location.1).checked_sub(i64::from(output_origin.1))?;
    let x = relative_x.checked_mul(i64::from(output_scale))?;
    let y = relative_y.checked_mul(i64::from(output_scale))?;

    Some(SurfaceGeometry {
        x,
        y,
        width,
        height,
    })
}

fn scaled_buffer_dimension(dimension: i32, buffer_scale: i32, output_scale: u32) -> Option<u32> {
    if dimension <= 0 {
        return None;
    }

    let numerator = u128::try_from(dimension)
        .ok()?
        .checked_mul(u128::from(output_scale))?;
    let denominator = u128::try_from(buffer_scale).ok()?;
    let rounded = numerator.checked_add(denominator / 2)? / denominator;
    u32::try_from(rounded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_scale_keeps_global_hotspot_anchor_fixed() {
        let geometry =
            surface_geometry((90, 90), (20, 20), 1, Transform::Normal, (0, 0), 1).unwrap();
        let scaled = scale_surface_geometry(geometry, (100, 100), (0, 0), 1, 2.0).unwrap();

        assert_eq!((scaled.x(), scaled.y()), (80, 80));
        assert_eq!((scaled.width(), scaled.height()), (40, 40));
    }

    #[test]
    fn scaling_rejects_invalid_factors() {
        let geometry = surface_geometry((0, 0), (10, 10), 1, Transform::Normal, (0, 0), 1).unwrap();

        assert!(scale_surface_geometry(geometry, (0, 0), (0, 0), 1, f64::NAN).is_none());
        assert!(scale_surface_geometry(geometry, (0, 0), (0, 0), 1, 3.5).is_none());
    }

    #[test]
    fn translates_global_position_and_applies_both_scales() {
        let geometry =
            surface_geometry((110, 70), (80, 40), 2, Transform::Normal, (100, 50), 2).unwrap();

        assert_eq!(
            geometry,
            SurfaceGeometry {
                x: 20,
                y: 40,
                width: 80,
                height: 40,
            }
        );
    }

    #[test]
    fn quarter_turn_transform_swaps_destination_dimensions() {
        let geometry = surface_geometry((0, 0), (80, 40), 1, Transform::_90, (0, 0), 1).unwrap();

        assert_eq!((geometry.width(), geometry.height()), (40, 80));
    }

    #[test]
    fn rounds_scaled_buffer_dimensions_to_nearest_physical_pixel() {
        let geometry = surface_geometry((0, 0), (5, 3), 2, Transform::Normal, (0, 0), 1).unwrap();

        assert_eq!((geometry.width(), geometry.height()), (3, 2));
    }

    #[test]
    fn clips_negative_surface_origin_without_changing_destination() {
        let geometry =
            surface_geometry((-10, 5), (20, 10), 1, Transform::Normal, (0, 0), 1).unwrap();

        assert_eq!(
            geometry.scissor(100, 100),
            Some(SurfaceScissor {
                x: 0,
                y: 5,
                width: 10,
                height: 10,
            })
        );
        assert_eq!(geometry.x(), -10);
    }

    #[test]
    fn skips_surfaces_outside_output_bounds() {
        let geometry =
            surface_geometry((120, 10), (20, 20), 1, Transform::Normal, (0, 0), 1).unwrap();

        assert_eq!(geometry.scissor(100, 100), None);
    }

    #[test]
    fn rejects_invalid_scale_and_zero_sized_buffers() {
        assert!(surface_geometry((0, 0), (10, 10), 0, Transform::Normal, (0, 0), 1).is_none());
        assert!(surface_geometry((0, 0), (10, 10), 1, Transform::Normal, (0, 0), 0).is_none());
        assert!(surface_geometry((0, 0), (0, 10), 1, Transform::Normal, (0, 0), 1).is_none());
    }
}
