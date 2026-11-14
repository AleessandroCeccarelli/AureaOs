// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-surface owned snapshots of Wayland SHM content.

use std::sync::{Arc, Mutex};

use smithay::{
    utils::Transform,
    reexports::wayland_server::protocol::{
        wl_buffer::WlBuffer,
        wl_surface::WlSurface,
    },
    wayland::compositor::{
        BufferAssignment, SurfaceAttributes, SurfaceData, TraversalAction,
        with_surface_tree_downward, with_states,
    },
};

use super::shm_buffer::{ShmBufferError, ShmBufferSnapshot};

/// Snapshot metadata required to place a Wayland buffer on an output.
pub(super) struct SurfaceRenderSnapshot {
    pub revision: u64,
    pub snapshot: Arc<ShmBufferSnapshot>,
    pub buffer_scale: i32,
    pub buffer_transform: Transform,
}

/// Snapshot retained independently from client-writable SHM memory.
#[derive(Default)]
pub struct SurfaceShmBufferState {
    state: Mutex<SurfaceSnapshotState>,
}

#[derive(Default)]
struct SurfaceSnapshotState {
    revision: u64,
    snapshot: Option<Arc<ShmBufferSnapshot>>,
    processed_buffer: Option<WlBuffer>,
}

impl SurfaceShmBufferState {
    /// Returns a shared snapshot without copying its pixel allocation.
    pub fn snapshot(&self) -> Option<Arc<ShmBufferSnapshot>> {
        self.snapshot_with_revision().map(|(_, snapshot)| snapshot)
    }

    /// Returns the update revision and shared pixel snapshot.
    pub fn snapshot_with_revision(&self) -> Option<(u64, Arc<ShmBufferSnapshot>)> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .snapshot
            .as_ref()
            .map(|snapshot| (state.revision, Arc::clone(snapshot)))
    }

    fn replace(&self, snapshot: Option<ShmBufferSnapshot>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.revision = state.revision.wrapping_add(1);
        state.snapshot = snapshot.map(Arc::new);
    }

    fn has_processed(&self, buffer: &WlBuffer) -> bool {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.processed_buffer.as_ref() == Some(buffer)
    }

    fn mark_processed(&self, buffer: WlBuffer) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.processed_buffer = Some(buffer);
    }

    fn clear_processed(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.processed_buffer = None;
    }
}

/// Copies new SHM buffers throughout a committed surface tree.
///
/// Synchronized subsurface state is applied by Smithay before the root
/// surface's commit callback. Traversing here therefore observes the coherent
/// tree state and also handles independently committed desynchronized children.
pub fn update_surface_tree_snapshots(root: &WlSurface) -> Result<(), ShmBufferError> {
    let mut first_error = None;

    with_surface_tree_downward(
        root,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_, states, &()| {
            if let Err(error) = update_surface_state(states)
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        },
        |_, _, &()| true,
    );

    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn update_surface_state(states: &SurfaceData) -> Result<(), ShmBufferError> {
    let mut cached_state = states.cached_state.get::<SurfaceAttributes>();
    let current = cached_state.current();
    let state = states
        .data_map
        .get_or_insert_threadsafe(SurfaceShmBufferState::default);

    match current.buffer.as_ref() {
        Some(BufferAssignment::NewBuffer(buffer)) => {
            if state.has_processed(buffer) {
                return Ok(());
            }

            let snapshot = ShmBufferSnapshot::copy_from_wl_buffer(buffer);
            state.mark_processed(buffer.clone());

            match snapshot {
                Ok(snapshot) => state.replace(Some(snapshot)),
                Err(ShmBufferError::NotSharedMemory | ShmBufferError::UnsupportedFormat) => {
                    state.replace(None);
                }
                Err(error) => {
                    state.replace(None);
                    return Err(error);
                }
            }

            // Keep current.buffer intact. Smithay's SurfaceAttributes merge
            // logic owns the attached-buffer lifetime and releases the previous
            // WlBuffer when a different buffer (or Removed) replaces it.
        }
        Some(BufferAssignment::Removed) => {
            state.replace(None);
            state.clear_processed();
            current.buffer = None;
        }
        None => {}
    }

    Ok(())
}

/// Returns the latest owned SHM snapshot associated with a surface.
pub fn surface_snapshot(surface: &WlSurface) -> Option<Arc<ShmBufferSnapshot>> {
    surface_snapshot_with_revision(surface).map(|(_, snapshot)| snapshot)
}

/// Returns pixel data and the committed buffer mapping attributes for rendering.
pub(super) fn surface_render_snapshot(surface: &WlSurface) -> Option<SurfaceRenderSnapshot> {
    with_states(surface, |states| {
        let (buffer_scale, buffer_transform) = {
            let mut attributes = states.cached_state.get::<SurfaceAttributes>();
            let current = attributes.current();
            (current.buffer_scale, current.buffer_transform)
        };

        if buffer_scale <= 0 {
            return None;
        }

        let (revision, snapshot) = states
            .data_map
            .get::<SurfaceShmBufferState>()?
            .snapshot_with_revision()?;

        Some(SurfaceRenderSnapshot {
            revision,
            snapshot,
            buffer_scale,
            buffer_transform,
        })
    })
}

/// Returns the latest snapshot together with its per-surface update revision.
pub fn surface_snapshot_with_revision(
    surface: &WlSurface,
) -> Option<(u64, Arc<ShmBufferSnapshot>)> {
    with_states(surface, |states| {
        states
            .data_map
            .get::<SurfaceShmBufferState>()
            .and_then(SurfaceShmBufferState::snapshot_with_revision)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_state_starts_empty() {
        let state = SurfaceShmBufferState::default();
        assert!(state.snapshot().is_none());
    }

    #[test]
    fn replacing_with_snapshot_advances_revision_without_copying_pixels() {
        let state = SurfaceShmBufferState::default();
        let pixels = vec![0x10, 0x20, 0x30, 0xff];
        let snapshot = ShmBufferSnapshot::from_rgba8(1, 1, pixels).unwrap();

        state.replace(Some(snapshot));
        let (revision, first) = state.snapshot_with_revision().unwrap();
        let second = state.snapshot().unwrap();

        assert_eq!(revision, 1);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn removing_snapshot_advances_revision_and_invalidates_pixels() {
        let state = SurfaceShmBufferState::default();
        state.replace(Some(
            ShmBufferSnapshot::from_rgba8(1, 1, vec![0, 0, 0, 0]).unwrap(),
        ));
        state.replace(None);

        assert!(state.snapshot().is_none());
        assert_eq!(state.state.lock().unwrap().revision, 2);
    }
}
