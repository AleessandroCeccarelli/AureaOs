// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Initial placement policy for newly mapped toplevel windows.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::Transform,
    wayland::compositor::{SurfaceAttributes, with_states},
};

use crate::render::shm_surface::surface_snapshot;

use super::AureaCompositor;

/// Centers the first mapped window in the first enabled output.
///
/// Later windows keep their compositor-assigned position until a full
/// placement and tiling policy is introduced. Buffer scale and transform are
/// respected so the calculation uses logical, not physical, dimensions.
pub(crate) fn center_first_mapped_window(compositor: &mut AureaCompositor, surface: &WlSurface) {
    let Some(first_mapped_surface) = compositor
        .window_manager
        .mapped_toplevels()
        .next()
        .map(|toplevel| toplevel.wl_surface().clone())
    else {
        return;
    };

    if first_mapped_surface != *surface {
        return;
    }

    let Some(snapshot) = surface_snapshot(surface) else {
        return;
    };

    let (buffer_scale, transform): (i32, Transform) = with_states(surface, |states| {
        let mut attributes = states.cached_state.get::<SurfaceAttributes>();
        let current = attributes.current();
        (current.buffer_scale, current.buffer_transform.into())
    });

    if buffer_scale <= 0 {
        return;
    }

    let (buffer_width, buffer_height) = if matches!(
        transform,
        Transform::_90 | Transform::_270 | Transform::Flipped90 | Transform::Flipped270
    ) {
        (snapshot.height(), snapshot.width())
    } else {
        (snapshot.width(), snapshot.height())
    };

    let window_size = (
        f64::from(buffer_width) / f64::from(buffer_scale),
        f64::from(buffer_height) / f64::from(buffer_scale),
    );

    let Some(output) = compositor
        .output_manager
        .registry()
        .iter()
        .find(|output| output.is_enabled())
    else {
        return;
    };
    let Some(bounds) = compositor
        .output_manager
        .logical_bounds(output.id())
        .ok()
        .flatten()
    else {
        return;
    };

    let origin = bounds.origin();
    let size = bounds.size();
    let Some(location) = centered_window_origin(
        (origin.x as f64, origin.y as f64),
        (f64::from(size.width), f64::from(size.height)),
        window_size,
    ) else {
        return;
    };

    // No popup tree can exist before the first toplevel is mapped. Keep this
    // initial placement on the manager path; interactive moves use the
    // compositor-level API so reactive popup trees are reconstrained.
    compositor
        .window_manager
        .set_toplevel_location(surface, location);
}

fn centered_window_origin(
    output_origin: (f64, f64),
    output_size: (f64, f64),
    window_size: (f64, f64),
) -> Option<(i32, i32)> {
    let x = output_origin.0 + ((output_size.0 - window_size.0) / 2.0).max(0.0);
    let y = output_origin.1 + ((output_size.1 - window_size.1) / 2.0).max(0.0);

    Some((finite_logical_coordinate(x)?, finite_logical_coordinate(y)?))
}

fn finite_logical_coordinate(value: f64) -> Option<i32> {
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return None;
    }

    Some(value.round() as i32)
}

#[cfg(test)]
mod tests {
    use super::centered_window_origin;

    #[test]
    fn centers_window_in_primary_output_logical_bounds() {
        assert_eq!(
            centered_window_origin((0.0, 0.0), (1920.0, 1080.0), (720.0, 440.0)),
            Some((600, 320))
        );
    }

    #[test]
    fn preserves_nonzero_monitor_origin() {
        assert_eq!(
            centered_window_origin((1920.0, 120.0), (1280.0, 720.0), (720.0, 440.0)),
            Some((2200, 260))
        );
    }

    #[test]
    fn keeps_oversized_window_origin_inside_output() {
        assert_eq!(
            centered_window_origin((0.0, 0.0), (640.0, 480.0), (800.0, 600.0)),
            Some((0, 0))
        );
    }

    #[test]
    fn rejects_non_finite_coordinates() {
        assert_eq!(
            centered_window_origin((f64::NAN, 0.0), (1920.0, 1080.0), (720.0, 440.0)),
            None
        );
    }
}
