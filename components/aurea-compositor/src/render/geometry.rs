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
mod tests {
    use super::*;

    #[test]
    fn quarter_turn_output_transform_maps_logical_extent_to_native_scanout() {
        let geometry = SurfaceGeometry {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        };

        assert_eq!(
            transform_geometry_for_output(geometry, 1920, 1080, Transform::_90),
            Some(SurfaceGeometry {
                x: 1020,
                y: 10,
                width: 40,
                height: 30,
            })
        );
        assert_eq!(
            transform_geometry_for_output(geometry, 1920, 1080, Transform::_270),
            Some(SurfaceGeometry {
                x: 20,
                y: 1040,
                width: 40,
                height: 30,
            })
        );
    }

    #[test]
    fn output_transform_preserves_native_extent_for_half_turns() {
        let geometry = SurfaceGeometry {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        };

        assert_eq!(
            transform_geometry_for_output(geometry, 1920, 1080, Transform::_180),
            Some(SurfaceGeometry {
                x: 1880,
                y: 1020,
                width: 30,
                height: 40,
            })
        );
    }

    #[test]
    fn output_rotation_maps_rectangles_without_changing_scanout_extent() {
        let geometry = SurfaceGeometry {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        };

        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::_90),
            Some(SurfaceGeometry {
                x: 40,
                y: 10,
                width: 40,
                height: 30,
            })
        );
        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::_180),
            Some(SurfaceGeometry {
                x: 160,
                y: 40,
                width: 30,
                height: 40,
            })
        );
        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::_270),
            Some(SurfaceGeometry {
                x: 20,
                y: 160,
                width: 40,
                height: 30,
            })
        );
    }

    #[test]
    fn output_reflections_map_surface_rectangles_inside_target() {
        let geometry = SurfaceGeometry {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        };

        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::Flipped),
            Some(SurfaceGeometry {
                x: 160,
                y: 20,
                width: 30,
                height: 40,
            })
        );
        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::Flipped90),
            Some(SurfaceGeometry {
                x: 20,
                y: 10,
                width: 40,
                height: 30,
            })
        );
        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::Flipped180),
            Some(SurfaceGeometry {
                x: 10,
                y: 40,
                width: 30,
                height: 40,
            })
        );
        assert_eq!(
            transform_surface_geometry(geometry, 200, 100, Transform::Flipped270),
            Some(SurfaceGeometry {
                x: 40,
                y: 160,
                width: 40,
                height: 30,
            })
        );
    }

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
    fn fractional_cursor_scale_keeps_hotspot_fixed_on_output() {
        let output_scale = ScaleFactor::from_percent(125).unwrap();
        let geometry = surface_geometry_with_fractional_scale(
            (90, 90),
            (20, 20),
            1,
            Transform::Normal,
            (0, 0),
            output_scale,
        )
        .unwrap();
        let scaled = scale_surface_geometry_with_fractional_scale(
            geometry,
            (100, 100),
            (0, 0),
            output_scale,
            2.0,
        )
        .unwrap();

        assert_eq!((scaled.x(), scaled.y()), (-24, -24));
        assert_eq!((scaled.width(), scaled.height()), (50, 50));
    }

    #[test]
    fn fractional_output_scale_controls_surface_origin_and_extent() {
        let scale = ScaleFactor::from_percent(125).unwrap();
        let geometry = surface_geometry_with_fractional_scale(
            (10, -10),
            (800, 600),
            1,
            Transform::Normal,
            (0, 0),
            scale,
        )
        .unwrap();

        assert_eq!(
            (
                geometry.x(),
                geometry.y(),
                geometry.width(),
                geometry.height(),
            ),
            (13, -13, 1000, 750)
        );
    }

    #[test]
    fn fractional_scale_keeps_multi_output_origin_local() {
        let scale = ScaleFactor::from_percent(150).unwrap();
        let geometry = surface_geometry_with_fractional_scale(
            (1700, 100),
            (800, 600),
            1,
            Transform::Normal,
            (1920, 0),
            scale,
        )
        .unwrap();

        assert_eq!((geometry.x(), geometry.y()), (-330, 150));
        assert_eq!((geometry.width(), geometry.height()), (1200, 900));
        assert_eq!(
            geometry.scissor(1920, 1080),
            Some(SurfaceScissor {
                x: 0,
                y: 150,
                width: 870,
                height: 900,
            })
        );
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
    fn clipping_each_output_edge_preserves_full_surface_geometry() {
        let cases = [
            (
                SurfaceGeometry {
                    x: -5,
                    y: 10,
                    width: 20,
                    height: 30,
                },
                (-5, 10, 20, 30),
                SurfaceScissor {
                    x: 0,
                    y: 10,
                    width: 15,
                    height: 30,
                },
            ),
            (
                SurfaceGeometry {
                    x: 90,
                    y: 10,
                    width: 20,
                    height: 30,
                },
                (90, 10, 20, 30),
                SurfaceScissor {
                    x: 90,
                    y: 10,
                    width: 10,
                    height: 30,
                },
            ),
            (
                SurfaceGeometry {
                    x: 10,
                    y: -5,
                    width: 20,
                    height: 30,
                },
                (10, -5, 20, 30),
                SurfaceScissor {
                    x: 10,
                    y: 0,
                    width: 20,
                    height: 25,
                },
            ),
            (
                SurfaceGeometry {
                    x: 10,
                    y: 90,
                    width: 20,
                    height: 30,
                },
                (10, 90, 20, 30),
                SurfaceScissor {
                    x: 10,
                    y: 90,
                    width: 20,
                    height: 10,
                },
            ),
        ];

        for (geometry, expected_geometry, expected_scissor) in cases {
            assert_eq!(
                (
                    geometry.x(),
                    geometry.y(),
                    geometry.width(),
                    geometry.height(),
                ),
                expected_geometry
            );
            assert_eq!(geometry.scissor(100, 100), Some(expected_scissor));
        }
    }

    #[test]
    fn one_surface_is_clipped_independently_for_outputs_with_different_scales() {
        let left_output =
            surface_geometry((1700, 100), (800, 600), 1, Transform::Normal, (0, 0), 1)
                .unwrap();
        let right_output =
            surface_geometry((1700, 100), (800, 600), 1, Transform::Normal, (1920, 0), 2)
                .unwrap();

        assert_eq!((left_output.x(), left_output.width()), (1700, 800));
        assert_eq!(
            left_output.scissor(1920, 1080),
            Some(SurfaceScissor {
                x: 1700,
                y: 100,
                width: 220,
                height: 600,
            })
        );

        assert_eq!((right_output.x(), right_output.width()), (-440, 1600));
        assert_eq!(
            right_output.scissor(2560, 2160),
            Some(SurfaceScissor {
                x: 0,
                y: 200,
                width: 1160,
                height: 1200,
            })
        );
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
    #[test]
    fn buffer_uv_mapping_applies_inverse_wayland_transform_for_all_eight_cases() {
        let surface_uv = (0.2, 0.3);
        let cases = [
            (Transform::Normal, (0.2, 0.3)),
            (Transform::_90, (0.3, 0.8)),
            (Transform::_180, (0.8, 0.7)),
            (Transform::_270, (0.7, 0.2)),
            (Transform::Flipped, (0.8, 0.3)),
            (Transform::Flipped90, (0.3, 0.2)),
            (Transform::Flipped180, (0.2, 0.7)),
            (Transform::Flipped270, (0.7, 0.8)),
        ];

        for (transform, expected) in cases {
            let actual = buffer_uv_from_surface_uv(transform, surface_uv).unwrap();
            assert!(
                (actual.0 - expected.0).abs() < f32::EPSILON
                    && (actual.1 - expected.1).abs() < f32::EPSILON,
                "unexpected UV mapping for {transform:?}: {actual:?}"
            );
        }
    }

    #[test]
    fn buffer_uv_mapping_rejects_non_finite_coordinates() {
        assert!(buffer_uv_from_surface_uv(Transform::Normal, (f32::NAN, 0.5)).is_none());
        assert!(buffer_uv_from_surface_uv(Transform::Normal, (0.5, f32::INFINITY)).is_none());
    }

}
