// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Logical hit-testing primitives for Wayland surface snapshots.

use smithay::utils::{Logical, Point, Transform};

use super::surface_tree::SurfaceTreeSnapshot;

/// Returns whether a logical point falls inside a surface snapshot.
///
/// Buffer dimensions are transformed before conversion to logical units.
/// Invalid scales and zero-sized buffers are never considered hittable.
pub fn contains_point(snapshot: &SurfaceTreeSnapshot, point: Point<f64, Logical>) -> bool {
    contains_surface_point(
        snapshot.location,
        (snapshot.snapshot.width(), snapshot.snapshot.height()),
        snapshot.buffer_scale,
        snapshot.buffer_transform,
        point,
    )
}

fn contains_surface_point(
    location: Point<i32, Logical>,
    buffer_size: (u32, u32),
    buffer_scale: i32,
    buffer_transform: Transform,
    point: Point<f64, Logical>,
) -> bool {
    if buffer_scale <= 0 || buffer_size.0 == 0 || buffer_size.1 == 0 {
        return false;
    }

    let swaps_dimensions = matches!(
        buffer_transform,
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
    );
    let (width, height) = if swaps_dimensions {
        (buffer_size.1, buffer_size.0)
    } else {
        buffer_size
    };

    let scale = f64::from(buffer_scale);
    let left = f64::from(location.x);
    let top = f64::from(location.y);
    let right = left + f64::from(width) / scale;
    let bottom = top + f64::from(height) / scale;

    point.x >= left && point.y >= top && point.x < right && point.y < bottom
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> Point<f64, Logical> {
        Point::from((x, y))
    }

    fn location(x: i32, y: i32) -> Point<i32, Logical> {
        Point::from((x, y))
    }

    #[test]
    fn hit_test_uses_logical_dimensions_and_excludes_right_bottom_edges() {
        let origin = location(10, 20);
        let size = (100, 60);

        assert!(contains_surface_point(
            origin,
            size,
            2,
            Transform::Normal,
            point(20.0, 30.0)
        ));
        assert!(contains_surface_point(
            origin,
            size,
            2,
            Transform::Normal,
            point(59.9, 49.9)
        ));
        assert!(!contains_surface_point(
            origin,
            size,
            2,
            Transform::Normal,
            point(60.0, 30.0)
        ));
        assert!(!contains_surface_point(
            origin,
            size,
            2,
            Transform::Normal,
            point(20.0, 50.0)
        ));
    }

    #[test]
    fn quarter_turns_swap_buffer_dimensions() {
        assert!(contains_surface_point(
            location(0, 0),
            (40, 80),
            1,
            Transform::_90,
            point(79.0, 39.0),
        ));
        assert!(!contains_surface_point(
            location(0, 0),
            (40, 80),
            1,
            Transform::_90,
            point(80.0, 10.0),
        ));
    }

    #[test]
    fn invalid_scale_and_empty_buffers_are_not_hittable() {
        assert!(!contains_surface_point(
            location(0, 0),
            (10, 10),
            0,
            Transform::Normal,
            point(1.0, 1.0),
        ));
        assert!(!contains_surface_point(
            location(0, 0),
            (0, 10),
            1,
            Transform::Normal,
            point(1.0, 1.0),
        ));
    }
}
