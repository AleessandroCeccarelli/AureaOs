// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Ordered SHM snapshots for complete Wayland surface trees.

use std::sync::Arc;

use smithay::{
    utils::{Logical, Point},
    wayland::compositor::{
        SubsurfaceCachedState, SurfaceData, TraversalAction, with_surface_tree_downward,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

use super::{
    shm_buffer::ShmBufferSnapshot,
    shm_surface::surface_snapshot_with_revision,
};

/// One owned surface image and its position in the toplevel's logical space.
pub struct SurfaceTreeSnapshot {
    pub location: Point<i32, Logical>,
    pub revision: u64,
    pub snapshot: Arc<ShmBufferSnapshot>,
}

#[derive(Clone, Copy)]
struct TraversalLocation {
    origin: Point<i32, Logical>,
    is_root: bool,
}

/// Collects all available SHM snapshots in Smithay's surface-tree render order.
///
/// Child locations are accumulated from each committed subsurface position.
/// The traversal order matches Smithay's own surface-tree render-element
/// traversal, preserving the stacking order of subsurfaces.
pub fn collect_surface_tree_snapshots(
    root: &WlSurface,
    root_location: Point<i32, Logical>,
) -> Vec<SurfaceTreeSnapshot> {
    let mut snapshots = Vec::new();

    with_surface_tree_downward(
        root,
        TraversalLocation {
            origin: root_location,
            is_root: true,
        },
        |_, states, parent| {
            let location = surface_location(states, *parent);
            TraversalAction::DoChildren(TraversalLocation {
                origin: location,
                is_root: false,
            })
        },
        |surface, states, parent| {
            let location = surface_location(states, *parent);
            if let Some((revision, snapshot)) = surface_snapshot_with_revision(surface) {
                snapshots.push(SurfaceTreeSnapshot {
                    location,
                    revision,
                    snapshot,
                });
            }
        },
        |_, _, _| true,
    );

    snapshots
}

fn surface_location(
    states: &SurfaceData,
    parent: TraversalLocation,
) -> Point<i32, Logical> {
    let offset = if parent.is_root {
        Point::from((0, 0))
    } else {
        let mut subsurface = states.cached_state.get::<SubsurfaceCachedState>();
        subsurface.current().location
    };

    resolve_surface_location(parent.origin, parent.is_root, offset)
}

fn resolve_surface_location(
    origin: Point<i32, Logical>,
    is_root: bool,
    offset: Point<i32, Logical>,
) -> Point<i32, Logical> {
    if is_root {
        origin
    } else {
        origin + offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_surface_keeps_toplevel_location() {
        let location = Point::<i32, Logical>::from((120, 80));
        let parent = TraversalLocation {
            origin: location,
            is_root: true,
        };

        assert_eq!(resolve_surface_location(parent.origin, parent.is_root, Point::from((9, 9))), Point::from((120, 80)));
    }

    #[test]
    fn child_locations_accumulate_parent_origin_and_subsurface_offset() {
        let parent = Point::<i32, Logical>::from((120, 80));
        let offset = Point::<i32, Logical>::from((24, -12));

        assert_eq!(resolve_surface_location(parent, false, offset), Point::from((144, 68)));
    }

    #[test]
    fn nested_subsurface_offsets_accumulate_in_parent_space() {
        let root = Point::<i32, Logical>::from((100, 50));
        let child = Point::<i32, Logical>::from((20, 10));
        let grandchild = Point::<i32, Logical>::from((-5, 8));

        let child_origin = resolve_surface_location(root, false, child);
        assert_eq!(
            resolve_surface_location(child_origin, false, grandchild),
            Point::from((115, 68))
        );
    }

}
