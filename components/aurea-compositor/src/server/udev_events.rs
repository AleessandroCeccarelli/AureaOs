// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Udev device discovery and event registration.

use std::{io, path::PathBuf};

use smithay::{
    backend::udev::{UdevBackend, UdevEvent},
    reexports::calloop::LoopHandle,
};

use super::state::WaylandState;

/// Registers the Udev backend and returns the initial DRM device snapshot.
///
/// The initial device list is collected before inserting the backend
/// into Calloop. Subsequent device changes are queued in `WaylandState`
/// for processing by the DRM device manager.
pub(super) fn register_udev_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    seat: &str,
) -> io::Result<Vec<PathBuf>> {
    let backend = UdevBackend::new(seat).map_err(io::Error::other)?;

    let initial_devices = backend
        .device_list()
        .map(|(_, path)| path.to_path_buf())
        .collect();

    loop_handle
        .insert_source(backend, |event: UdevEvent, _, state| {
            state.udev_events.push(event);
        })
        .map_err(|_| io::Error::other("Failed to register the Udev event backend"))?;

    Ok(initial_devices)
}
