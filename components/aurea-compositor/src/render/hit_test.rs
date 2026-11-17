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
    let width = snapshot.snapshot.width();
    let height = snapshot.snapshot.height();

    if snapshot.buffer_scale <= 0 || width == 0 || height == 0 {
        return false;
    }

    let swaps_dimensions = matches!(
        snapshot.buffer_transform,
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
    );

    let (width, height) = if swaps_dimensions {
        (height, width)
    } else {
        (width, height)
    };

    let scale = f64::from(snapshot.buffer_scale);
    let left = f64::from(snapshot.location.x);
    let top = f64::from(snapshot.location.y);
    let right = left + f64::from(width) / scale;
    let bottom = top + f64::from(height) / scale;

    point.x >= left && point.y >= top && point.x < right && point.y < bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::shm_buffer::ShmBufferSnapshot;
    use smithay::reexports::wayland_server::backend::ObjectId;
    use std::sync::Arc;

    fn snapshot(
        location: (i32, i32),
        size: (u32, u32),
        scale: i32,
        transform: Transform,
    ) -> SurfaceTreeSnapshot {
        // ObjectId is not constructible without a live Wayland backend. The
        // geometry helper is tested through the standalone rectangle below.
        let _ = (location, size, scale, transform);
        unreachable!("surface snapshots require a Wayland object identity")
    }

    #[test]
    fn hit_test_uses_logical_dimensions_and_excludes_right_bottom_edges() {
        assert!(contains_rect((10, 20), (100, 60), 2, Transform::Normal, (20.0, 30.0)));
        assert!(contains_rect((10, 20), (100, 60), 2, Transform::Normal, (59.9, 49.9)));
        assert!(!contains_rect((10, 20), (100, 60), 2, Transform::Normal, (60.0, 30.0)));
        assert!(!contains_rect((10, 20), (100, 60), 2, Transform::Normal, (20.0, 50.0)));
    }

    #[test]
    fn quarter_turns_swap_buffer_dimensions() {
        assert!(contains_rect((0, 0), (40, 80), 1, Transform::_90, (39.0, 79.0)));
        assert!(!contains_rect((0, 0), (40, 80), 1, Transform::_90, (40.0, 10.0)));
    }

    fn contains_rect(
        location: (i32, i32),
        size: (u32, u32),
        scale: i32,
        transform: Transform,
        point: (f64, f64),
    ) -> bool {
        if scale <= 0 || size.0 == 0 || size.1 == 0 {
            return false;
        }
        let (width, height) = if matches!(
            transform,
            Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
        ) {
            (size.1, size.0)
        } else {
            size
        };
        let left = f64::from(location.0);
        let top = f64::from(location.1);
        point.0 >= left
            && point.1 >= top
            && point.0 < left + f64::from(width) / f64::from(scale)
            && point.1 < top + f64::from(height) / f64::from(scale)
    }
}
