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
pub(super) fn center_first_mapped_window(
    compositor: &mut AureaCompositor,
    surface: &WlSurface,
) {
    if compositor.window_manager.mapped_window_count() != 1 {
        return;
    }

    let Some(snapshot) = surface_snapshot(surface) else {
        return;
    };

    let (buffer_scale, transform) = with_states(surface, |states| {
        let mut attributes = states.cached_state.get::<SurfaceAttributes>();
        let current = attributes.current();
        (current.buffer_scale, current.buffer_transform)
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

    let scale = f64::from(buffer_scale);
    let window_width = f64::from(buffer_width) / scale;
    let window_height = f64::from(buffer_height) / scale;

    let Some(output) = compositor
        .output_manager
        .registry()
        .iter()
        .find(|output| output.is_enabled())
    else {
        return;
    };
    let Some(position) = compositor.output_manager.layout().get(output.id()) else {
        return;
    };

    let output_scale = output.effective_scale();
    if !output_scale.is_finite() || output_scale <= 0.0 {
        return;
    }

    let output_width = f64::from(output.current_mode().width()) / output_scale;
    let output_height = f64::from(output.current_mode().height()) / output_scale;
    let x = f64::from(position.x()) + ((output_width - window_width) / 2.0).max(0.0);
    let y = f64::from(position.y()) + ((output_height - window_height) / 2.0).max(0.0);

    let Some(x) = finite_logical_coordinate(x) else {
        return;
    };
    let Some(y) = finite_logical_coordinate(y) else {
        return;
    };

    compositor
        .window_manager
        .set_toplevel_location(surface, (x, y));
}

fn finite_logical_coordinate(value: f64) -> Option<i32> {
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return None;
    }

    Some(value.round() as i32)
}
