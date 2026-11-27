// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Ordered SHM snapshots for complete Wayland surface trees.

use std::sync::Arc;

use smithay::{
    reexports::wayland_server::{
        Resource, backend::ObjectId,
        protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
    },
    utils::{Logical, Point, Transform},
    wayland::compositor::{
        SubsurfaceCachedState, SurfaceData, TraversalAction, with_surface_tree_downward,
    },
};

use super::{shm_buffer::ShmBufferSnapshot, shm_surface::surface_render_snapshot};

/// One owned surface image and its accumulated global logical position.
pub struct SurfaceTreeSnapshot {
    /// Wayland surface represented by this snapshot.
    pub surface: WlSurface,
    /// Full Wayland object identity; unlike protocol_id, this is not reused.
    pub surface_id: ObjectId,
    pub location: Point<i32, Logical>,
    pub revision: u64,
    pub snapshot: Arc<ShmBufferSnapshot>,
    pub buffer_scale: i32,
    pub buffer_transform: Transform,
    /// Scale applied only when this snapshot belongs to the pointer cursor tree.
    pub cursor_scale: f64,
    /// Global logical point around which cursor scaling is anchored.
    pub cursor_anchor: Option<Point<i32, Logical>>,
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
/// Collects every live Wayland surface identity in tree traversal order.
///
/// Surfaces without an attached SHM buffer are included. This lets resource
/// caches distinguish a live but currently non-renderable surface from one
/// that has been destroyed or unmapped.
pub fn collect_surface_tree_surfaces(root: &WlSurface) -> Vec<WlSurface> {
    let mut surfaces = Vec::new();

    with_surface_tree_downward(
        root,
        (),
        |_, _, _| TraversalAction::DoChildren(()),
        |surface, _, _| surfaces.push(surface.clone()),
        |_, _, _| true,
    );

    surfaces
}

pub fn collect_surface_tree_ids(root: &WlSurface) -> Vec<ObjectId> {
    let mut surface_ids = Vec::new();

    with_surface_tree_downward(
        root,
        (),
        |_, _, _| TraversalAction::DoChildren(()),
        |surface, _, _| surface_ids.push(surface.id()),
        |_, _, _| true,
    );

    surface_ids
}

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
            if let Some(render_snapshot) = surface_render_snapshot(surface) {
                snapshots.push(SurfaceTreeSnapshot {
                    surface: surface.clone(),
                    surface_id: surface.id(),
                    location,
                    revision: render_snapshot.revision,
                    snapshot: render_snapshot.snapshot,
                    buffer_scale: render_snapshot.buffer_scale,
                    buffer_transform: render_snapshot.buffer_transform,
                    cursor_scale: 1.0,
                    cursor_anchor: None,
                });
            }
        },
        |_, _, _| true,
    );

    snapshots
}

fn surface_location(states: &SurfaceData, parent: TraversalLocation) -> Point<i32, Logical> {
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
    if is_root { origin } else { origin + offset }
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

        assert_eq!(
            resolve_surface_location(parent.origin, parent.is_root, Point::from((9, 9))),
            Point::from((120, 80))
        );
    }

    #[test]
    fn child_locations_accumulate_parent_origin_and_subsurface_offset() {
        let parent = Point::<i32, Logical>::from((120, 80));
        let offset = Point::<i32, Logical>::from((24, -12));

        assert_eq!(
            resolve_surface_location(parent, false, offset),
            Point::from((144, 68))
        );
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


/// One retained DMA-BUF surface and its accumulated global logical position.
///
/// The import layout owns a strong reference to the client DMA-BUF, keeping
/// every plane descriptor alive while the renderer imports or samples it.
#[cfg(feature = "drm")]
pub struct SurfaceTreeDmabufSnapshot {
    /// Wayland surface represented by this snapshot.
    pub surface: WlSurface,
    /// Full Wayland object identity used by the renderer texture cache.
    pub surface_id: ObjectId,
    /// Strong Wayland resource reference retained until the renderer releases this snapshot.
    /// Keeping only the DMA-BUF file descriptors alive is insufficient: the client must not
    /// receive `wl_buffer.release` while the GPU may still be reading the buffer.
    pub buffer: WlBuffer,
    /// Global logical origin after accumulating subsurface positions.
    pub location: Point<i32, Logical>,
    /// Validated dimensions, DRM format, modifier and per-plane metadata.
    pub layout: crate::vulkan::DmabufImportLayout,
    /// Committed buffer scale.
    pub buffer_scale: i32,
    /// Committed Wayland buffer transform.
    pub buffer_transform: Transform,
    /// Whether this snapshot belongs to the client-provided cursor surface tree.
    pub is_cursor: bool,
}

/// Collects DMA-BUF-backed surfaces in the same stacking order as SHM snapshots.
///
/// This deliberately does not advertise the linux-dmabuf protocol or claim
/// that a buffer is importable by Vulkan. It only preserves validated buffer
/// ownership and surface placement for the later device-specific import step.
#[cfg(feature = "drm")]
pub fn collect_surface_tree_dmabuf_snapshots(
    root: &WlSurface,
    root_location: Point<i32, Logical>,
) -> Vec<SurfaceTreeDmabufSnapshot> {
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
            if let Some(snapshot) = surface_dmabuf_snapshot(surface) {
                snapshots.push(SurfaceTreeDmabufSnapshot {
                    surface: surface.clone(),
                    surface_id: surface.id(),
                    buffer: snapshot.buffer,
                    location,
                    layout: snapshot.layout,
                    buffer_scale: snapshot.buffer_scale,
                    buffer_transform: snapshot.buffer_transform,
                    is_cursor: false,
                });
            }
        },
        |_, _, _| true,
    );

    snapshots
}

#[cfg(feature = "drm")]
struct SurfaceDmabufSnapshot {
    buffer: WlBuffer,
    layout: crate::vulkan::DmabufImportLayout,
    buffer_scale: i32,
    buffer_transform: Transform,
}

#[cfg(feature = "drm")]
fn surface_dmabuf_snapshot(surface: &WlSurface) -> Option<SurfaceDmabufSnapshot> {
    use smithay::{
        reexports::wayland_server::protocol::wl_buffer::WlBuffer,
        wayland::{compositor::{BufferAssignment, SurfaceAttributes, with_states}, dmabuf::get_dmabuf},
    };

    with_states(surface, |states| {
        let (buffer, buffer_scale, buffer_transform) = {
            let mut attributes = states.cached_state.get::<SurfaceAttributes>();
            let current = attributes.current();
            let buffer = match current.buffer.as_ref()? {
                BufferAssignment::NewBuffer(buffer) => buffer.clone(),
                BufferAssignment::Removed => return None,
            };
            (buffer, current.buffer_scale, current.buffer_transform)
        };

        if buffer_scale <= 0 {
            return None;
        }

        let dmabuf = get_dmabuf(&buffer).ok()?;
        let layout = crate::vulkan::DmabufImportLayout::from_dmabuf(dmabuf).ok()?;
        Some(SurfaceDmabufSnapshot {
            buffer,
            layout,
            buffer_scale,
            buffer_transform: buffer_transform.into(),
        })
    })
}
