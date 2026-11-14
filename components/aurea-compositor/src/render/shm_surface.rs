// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-surface owned snapshots of Wayland SHM content.

use std::sync::{Arc, Mutex};

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{
        BufferAssignment, SurfaceAttributes, SurfaceData, TraversalAction,
        with_surface_tree_downward, with_states,
    },
};

use super::shm_buffer::{ShmBufferError, ShmBufferSnapshot};

/// Snapshot retained independently from client-writable SHM memory.
#[derive(Default)]
pub struct SurfaceShmBufferState {
    snapshot: Mutex<Option<Arc<ShmBufferSnapshot>>>,
}

impl SurfaceShmBufferState {
    /// Returns a shared snapshot without copying its pixel allocation.
    pub fn snapshot(&self) -> Option<Arc<ShmBufferSnapshot>> {
        self.snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn replace(&self, snapshot: Option<ShmBufferSnapshot>) {
        *self
            .snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            snapshot.map(Arc::new);
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
    let update = match current.buffer.as_ref() {
        Some(BufferAssignment::NewBuffer(buffer)) => Some(Some(buffer.clone())),
        Some(BufferAssignment::Removed) => Some(None),
        None => None,
    };

    let state = states
        .data_map
        .get_or_insert_threadsafe(SurfaceShmBufferState::default);
    let Some(update) = update else {
        return Ok(());
    };

    match update {
        Some(None) => {
            state.replace(None);
            current.buffer = None;
        }
        Some(Some(buffer)) => match ShmBufferSnapshot::copy_from_wl_buffer(&buffer) {
            Ok(snapshot) => {
                state.replace(Some(snapshot));
                current.buffer = None;
                buffer.release();
            }
            Err(ShmBufferError::NotSharedMemory | ShmBufferError::UnsupportedFormat) => {
                state.replace(None);
            }
            Err(error) => {
                state.replace(None);
                return Err(error);
            }
        },
        None => {}
    }

    Ok(())
}

/// Returns the latest owned SHM snapshot associated with a surface.
pub fn surface_snapshot(surface: &WlSurface) -> Option<Arc<ShmBufferSnapshot>> {
    with_states(surface, |states| {
        states
            .data_map
            .get::<SurfaceShmBufferState>()
            .and_then(SurfaceShmBufferState::snapshot)
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
    fn replacing_with_none_keeps_state_empty() {
        let state = SurfaceShmBufferState::default();
        state.replace(None);
        assert!(state.snapshot().is_none());
    }
}
