// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Reconciles surface/output membership and client fractional scale.

use smithay::output::Output;

use crate::{
    output::coordinates::{LogicalPoint, LogicalRect, LogicalSize, OutputLogicalBounds},
    render::RenderScene,
};

use super::{AureaCompositor, fractional_scale::set_surface_preferred_scale};

struct ActiveOutput {
    id: crate::output::OutputId,
    output: Output,
    bounds: OutputLogicalBounds,
    scale: f64,
}

/// Recalculates output membership for every mapped surface in the scene.
pub(super) fn reconcile(compositor: &mut AureaCompositor) {
    let active_outputs = compositor
        .output_manager
        .registry()
        .iter()
        .filter(|description| description.is_enabled())
        .filter_map(|description| {
            let id = description.id();
            let bounds = compositor.output_manager.logical_bounds(id).ok()??;
            let output = compositor.wayland_outputs.get(&id)?.clone();
            Some(ActiveOutput {
                id,
                output,
                bounds,
                scale: description.effective_scale(),
            })
        })
        .collect::<Vec<_>>();

    let snapshots = RenderScene::from_window_manager(&compositor.window_manager).shm_snapshots();

    for snapshot in snapshots {
        let (width, height) = logical_dimensions(
            snapshot.snapshot.width(),
            snapshot.snapshot.height(),
            snapshot.buffer_scale,
            snapshot.buffer_transform,
        );
        let Some((width, height)) = width.zip(height) else {
            compositor.forget_surface_output_membership(&snapshot.surface);
            continue;
        };
        let size = LogicalSize { width, height };
        let origin = LogicalPoint {
            x: i64::from(snapshot.location.x),
            y: i64::from(snapshot.location.y),
        };
        let Some(rect) = valid_rect(origin, size) else {
            compositor.forget_surface_output_membership(&snapshot.surface);
            continue;
        };

        let intersecting = active_outputs
            .iter()
            .filter(|output| output.bounds.intersects(rect))
            .collect::<Vec<_>>();
        let preferred_scale = intersecting
            .iter()
            .map(|output| output.scale)
            .reduce(f64::max)
            .unwrap_or(1.0);

        compositor.update_surface_output_membership(
            &snapshot.surface,
            intersecting
                .iter()
                .map(|output| (output.id, output.output.clone())),
        );
        set_surface_preferred_scale(&snapshot.surface, preferred_scale);
    }

    compositor.prune_surface_output_membership();
}

fn valid_rect(origin: LogicalPoint, size: LogicalSize) -> Option<LogicalRect> {
    if size.width == 0 || size.height == 0 {
        return None;
    }
    origin.x.checked_add(i64::from(size.width))?;
    origin.y.checked_add(i64::from(size.height))?;
    Some(LogicalRect::new(origin, size))
}

fn logical_dimensions(
    width: u32,
    height: u32,
    buffer_scale: i32,
    transform: smithay::utils::Transform,
) -> (Option<u32>, Option<u32>) {
    if buffer_scale <= 0 {
        return (None, None);
    }

    let (width, height) = if matches!(
        transform,
        smithay::utils::Transform::_90
            | smithay::utils::Transform::_270
            | smithay::utils::Transform::Flipped90
            | smithay::utils::Transform::Flipped270
    ) {
        (height, width)
    } else {
        (width, height)
    };
    let scale = buffer_scale as u32;
    (Some(width.div_ceil(scale)), Some(height.div_ceil(scale)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_dimensions_apply_buffer_scale() {
        assert_eq!(
            logical_dimensions(1920, 1080, 2, smithay::utils::Transform::Normal),
            (Some(960), Some(540))
        );
    }

    #[test]
    fn rotated_buffers_swap_logical_dimensions() {
        assert_eq!(
            logical_dimensions(1920, 1080, 1, smithay::utils::Transform::_90),
            (Some(1080), Some(1920))
        );
    }

    #[test]
    fn invalid_buffer_scale_is_rejected() {
        assert_eq!(
            logical_dimensions(1920, 1080, 0, smithay::utils::Transform::Normal),
            (None, None)
        );
    }

    #[test]
    fn empty_surface_rect_is_rejected() {
        assert!(valid_rect(
            LogicalPoint { x: 10, y: 20 },
            LogicalSize {
                width: 0,
                height: 100,
            },
        )
        .is_none());
    }

    #[test]
    fn overflowing_surface_rect_is_rejected() {
        assert!(valid_rect(
            LogicalPoint { x: i64::MAX, y: 0 },
            LogicalSize {
                width: 1,
                height: 100,
            },
        )
        .is_none());
    }

    #[test]
    fn valid_surface_rect_preserves_global_geometry() {
        let origin = LogicalPoint { x: -1920, y: 120 };
        let size = LogicalSize {
            width: 800,
            height: 600,
        };

        let rect = valid_rect(origin, size).unwrap();
        assert_eq!(rect.origin(), origin);
        assert_eq!(rect.size(), size);
    }
}
