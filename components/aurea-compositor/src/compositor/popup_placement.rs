// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Output-aware placement for XDG popup surfaces.

use smithay::{
    desktop::{PopupKind, PopupManager, Window, find_popup_root_surface},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Rectangle, Size},
    wayland::shell::xdg::{PopupSurface, PositionerState},
};

use crate::{
    output::coordinates::{LogicalPoint, OutputLogicalBounds},
    AureaCompositor,
};

/// Applies the XDG constraint-adjustment policy against the output containing
/// the popup anchor. Nested popup parents are resolved in toplevel coordinates.
pub(crate) fn constrain_geometry(
    compositor: &AureaCompositor,
    popup: &PopupSurface,
    positioner: PositionerState,
) -> Rectangle<i32, Logical> {
    let fallback = positioner.get_geometry();
    let popup_kind = PopupKind::Xdg(popup.clone());
    let Ok(root) = find_popup_root_surface(&popup_kind) else {
        return fallback;
    };
    let Some((toplevel, toplevel_location)) = compositor
        .window_manager
        .mapped_toplevels_with_location()
        .find(|(toplevel, _)| toplevel.wl_surface() == &root)
    else {
        return fallback;
    };

    let window_geometry = Window::new_wayland_window(toplevel.clone()).geometry();
    let root_window_origin = Point::<i32, Logical>::from(toplevel_location) + window_geometry.loc;
    let parent = match popup.get_parent_surface() {
        Some(parent) => parent,
        None => return fallback,
    };

    let (parent_origin, actual_parent_size) = if parent == root {
        (root_window_origin, window_geometry.size)
    } else {
        let Some((parent_popup, parent_offset)) = PopupManager::popups_for_surface(&root)
            .find(|(candidate, _)| candidate.wl_surface() == &parent)
        else {
            return fallback;
        };
        let parent_geometry = parent_popup.geometry();
        (
            root_window_origin + parent_offset + parent_geometry.loc,
            parent_geometry.size,
        )
    };

    let parent_size = positioner.parent_size.unwrap_or(actual_parent_size);
    let anchor = positioner.anchor_rect;
    let anchor_point = LogicalPoint {
        x: i64::from(parent_origin.x)
            + i64::from(anchor.loc.x)
            + i64::from(anchor.size.w / 2),
        y: i64::from(parent_origin.y)
            + i64::from(anchor.loc.y)
            + i64::from(anchor.size.h / 2),
    };
    let parent_center = LogicalPoint {
        x: i64::from(parent_origin.x) + i64::from(parent_size.w / 2),
        y: i64::from(parent_origin.y) + i64::from(parent_size.h / 2),
    };

    let Some(bounds) = select_constraint_output(compositor, anchor_point, parent_center) else {
        return fallback;
    };
    target_relative_to_parent(bounds, parent_origin).unwrap_or(fallback)
        .pipe(|target| positioner.get_unconstrained_geometry(target))
}

fn select_constraint_output(
    compositor: &AureaCompositor,
    anchor: LogicalPoint,
    parent_center: LogicalPoint,
) -> Option<OutputLogicalBounds> {
    let outputs = compositor
        .output_manager
        .registry()
        .iter()
        .filter(|output| output.is_enabled())
        .filter_map(|output| compositor.output_manager.logical_bounds(output.id()).ok().flatten())
        .collect::<Vec<_>>();

    outputs
        .iter()
        .copied()
        .find(|bounds| bounds.contains(anchor))
        .or_else(|| outputs.iter().copied().find(|bounds| bounds.contains(parent_center)))
}

fn target_relative_to_parent(
    bounds: OutputLogicalBounds,
    parent_origin: Point<i32, Logical>,
) -> Option<Rectangle<i32, Logical>> {
    let origin = bounds.origin();
    let size = bounds.size();
    let x = i32::try_from(origin.x.checked_sub(i64::from(parent_origin.x))?).ok()?;
    let y = i32::try_from(origin.y.checked_sub(i64::from(parent_origin.y))?).ok()?;
    let width = i32::try_from(size.width).ok()?;
    let height = i32::try_from(size.height).ok()?;

    Some(Rectangle::new(
        Point::from((x, y)),
        Size::from((width, height)),
    ))
}

#[cfg(test)]
mod tests {
    use super::target_relative_to_parent;
    use crate::output::coordinates::{LogicalPoint, LogicalSize, OutputLogicalBounds};

    #[test]
    fn converts_global_output_bounds_to_parent_window_coordinates() {
        let bounds = OutputLogicalBounds::from_logical_size(
            LogicalPoint { x: 1920, y: -120 },
            LogicalSize {
                width: 1280,
                height: 720,
            },
        )
        .unwrap();

        let target = target_relative_to_parent(bounds, (2000, 80).into()).unwrap();
        assert_eq!(target.loc, (-80, -200).into());
        assert_eq!(target.size, (1280, 720).into());
    }

    #[test]
    fn rejects_relative_coordinates_that_overflow_protocol_geometry() {
        let bounds = OutputLogicalBounds::from_logical_size(
            LogicalPoint { x: i64::MAX - 100, y: 0 },
            LogicalSize {
                width: 50,
                height: 50,
            },
        )
        .unwrap();

        assert!(target_relative_to_parent(bounds, (i32::MIN, 0).into()).is_none());
    }
}
