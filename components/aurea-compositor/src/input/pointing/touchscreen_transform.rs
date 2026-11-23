// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Maps absolute touchscreen coordinates through an output orientation.

use smithay::utils::Transform;

/// Converts output-local coordinates into the oriented logical output space.
pub fn transform_touch_coordinates(
    transform: Transform,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<(f64, f64)> {
    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return None;
    }

    let u = (x / width).clamp(0.0, 1.0);
    let v = (y / height).clamp(0.0, 1.0);
    let (u, v) = match transform {
        Transform::Normal => (u, v),
        Transform::_90 => (1.0 - v, u),
        Transform::_180 => (1.0 - u, 1.0 - v),
        Transform::_270 => (v, 1.0 - u),
        Transform::Flipped => (1.0 - u, v),
        Transform::Flipped90 => (v, u),
        Transform::Flipped180 => (u, 1.0 - v),
        Transform::Flipped270 => (1.0 - v, 1.0 - u),
    };

    let swaps_dimensions = matches!(
        transform,
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
    );
    let (target_width, target_height) = if swaps_dimensions {
        (height, width)
    } else {
        (width, height)
    };
    Some((
        (u * target_width).clamp(0.0, target_width),
        (v * target_height).clamp(0.0, target_height),
    ))
}

#[cfg(test)]
mod tests {
    use super::transform_touch_coordinates;
    use smithay::utils::Transform;

    const WIDTH: f64 = 100.0;
    const HEIGHT: f64 = 60.0;

    #[test]
    fn normal_orientation_preserves_coordinates() {
        assert_eq!(
            transform_touch_coordinates(Transform::Normal, 20.0, 15.0, WIDTH, HEIGHT),
            Some((20.0, 15.0))
        );
    }

    #[test]
    fn all_eight_orientations_map_the_top_left_corner() {
        let cases = [
            (Transform::Normal, (0.0, 0.0)),
            (Transform::_90, (60.0, 0.0)),
            (Transform::_180, (100.0, 60.0)),
            (Transform::_270, (0.0, 100.0)),
            (Transform::Flipped, (100.0, 0.0)),
            (Transform::Flipped90, (0.0, 0.0)),
            (Transform::Flipped180, (0.0, 60.0)),
            (Transform::Flipped270, (60.0, 100.0)),
        ];
        for (transform, expected) in cases {
            assert_eq!(
                transform_touch_coordinates(transform, 0.0, 0.0, WIDTH, HEIGHT),
                Some(expected),
                "{transform:?}"
            );
        }
    }

    #[test]
    fn quarter_turns_swap_output_extents() {
        assert_eq!(
            transform_touch_coordinates(Transform::_90, 100.0, 60.0, WIDTH, HEIGHT),
            Some((0.0, 100.0))
        );
        assert_eq!(
            transform_touch_coordinates(Transform::_270, 100.0, 60.0, WIDTH, HEIGHT),
            Some((60.0, 0.0))
        );
    }

    #[test]
    fn rejects_non_finite_and_empty_geometry() {
        assert_eq!(
            transform_touch_coordinates(Transform::Normal, f64::NAN, 0.0, WIDTH, HEIGHT),
            None
        );
        assert_eq!(
            transform_touch_coordinates(Transform::Normal, 0.0, 0.0, 0.0, HEIGHT),
            None
        );
    }

    #[test]
    fn clamps_out_of_range_device_coordinates() {
        assert_eq!(
            transform_touch_coordinates(Transform::Normal, 150.0, -10.0, WIDTH, HEIGHT),
            Some((100.0, 0.0))
        );
    }
}
