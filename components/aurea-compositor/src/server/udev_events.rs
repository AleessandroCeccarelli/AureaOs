// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Udev device discovery and GPU inventory synchronization.

use std::{io, path::PathBuf};

use smithay::{
    backend::udev::{UdevBackend, UdevEvent},
    reexports::calloop::LoopHandle,
};

use super::state::WaylandState;

/// Registers Udev monitoring and returns the initial GPU inventory.
///
/// Device identifiers are preserved alongside their DRM paths so that
/// subsequent removal events can identify the corresponding GPU.
pub(super) fn register_udev_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    seat: &str,
) -> io::Result<Vec<(String, PathBuf)>> {
    let backend = UdevBackend::new(seat).map_err(io::Error::other)?;

    let initial_devices = backend
        .device_list()
        .map(|(device_id, path)| (device_id.to_string(), path.to_path_buf()))
        .collect();

    loop_handle
        .insert_source(backend, |event: UdevEvent, _, state| match event {
            UdevEvent::Added { device_id, path } => {
                let device_id = device_id.to_string();

                if state.gpu_manager.get_device(&path).is_none() {
                    if let Err(error) = state.gpu_manager.register_device(path.clone()) {
                        state.error = Some(io::Error::other(error));
                        return;
                    }
                }

                if !state.udev_device_ids.iter().any(|(id, _)| id == &device_id) {
                    state.udev_device_ids.push((device_id, path));
                }
            }

            UdevEvent::Changed { device_id } => {
                let device_id = device_id.to_string();

                if state.udev_device_ids.iter().any(|(id, _)| id == &device_id)
                    && !state.udev_changed_devices.contains(&device_id)
                {
                    state.udev_changed_devices.push(device_id);
                }
            }

            UdevEvent::Removed { device_id } => {
                let device_id = device_id.to_string();

                if let Some(index) = state
                    .udev_device_ids
                    .iter()
                    .position(|(id, _)| id == &device_id)
                {
                    let (_, path) = state.udev_device_ids.remove(index);
                    state.gpu_manager.remove_device(&path);
                }

                state.udev_changed_devices.retain(|id| id != &device_id);
            }
        })
        .map_err(|_| io::Error::other("Failed to register the Udev event backend"))?;

    Ok(initial_devices)
}
