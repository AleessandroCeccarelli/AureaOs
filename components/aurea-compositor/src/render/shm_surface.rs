// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-surface owned snapshots of Wayland SHM content.

use std::sync::{Arc, Mutex};

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{BufferAssignment, SurfaceAttributes, with_states},
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

/// Copies a newly attached SHM buffer and releases it once the copy is owned.
///
/// Unsupported non-SHM buffers are left attached for a future import path.
/// A removed or unsupported replacement invalidates any previous SHM snapshot.
pub fn update_surface_snapshot(surface: &WlSurface) -> Result<(), ShmBufferError> {
    with_states(surface, |states| {
        let mut cached_state = states.cached_state.get::<SurfaceAttributes>();
        let current = cached_state.current();
        let update = match current.buffer.as_ref() {
            Some(BufferAssignment::NewBuffer(buffer)) => Some(Some(buffer.clone())),
            Some(BufferAssignment::Removed) => Some(None),
            None => None,
        };

        let state = states
            .data_map
            .get_or_insert(SurfaceShmBufferState::default);
        let Some(update) = update else {
            return Ok(());
        };

        match update {
            None => {
                state.replace(None);
                current.buffer = None;
            }
            Some(buffer) => match ShmBufferSnapshot::copy_from_wl_buffer(&buffer) {
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
        }

        Ok(())
    })
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
    fn replacing_with_none_invalidates_previous_snapshot() {
        let state = SurfaceShmBufferState::default();
        state.replace(None);
        assert!(state.snapshot().is_none());
    }
}
