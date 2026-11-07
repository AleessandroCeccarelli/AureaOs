// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Udev device discovery and DRM registry synchronization.

use std::{io, path::PathBuf};

use smithay::{
    backend::{
        drm::DrmNode,
        udev::{UdevBackend, UdevEvent},
    },
    reexports::calloop::LoopHandle,
};

use super::state::WaylandState;

/// Registers Udev monitoring and returns the initial DRM device inventory.
///
/// Smithay's device identifier is converted to a canonical `DrmNode`.
/// Device paths are retained for GPU inventory and device lifecycle work.
pub(super) fn register_udev_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    seat: &str,
) -> io::Result<Vec<(DrmNode, PathBuf)>> {
    let backend = UdevBackend::new(seat).map_err(io::Error::other)?;

    let initial_devices = backend
        .device_list()
        .map(|(device_id, path)| {
            DrmNode::from_dev_id(device_id as u64)
                .map(|node| (node, path.to_path_buf()))
                .map_err(io::Error::other)
        })
        .collect::<io::Result<Vec<_>>>()?;

    loop_handle
        .insert_source(backend, |event: UdevEvent, _, state| match event {
            UdevEvent::Added { device_id, path } => {
                let node = match DrmNode::from_dev_id(device_id as u64) {
                    Ok(node) => node,
                    Err(error) => {
                        state.error = Some(io::Error::other(error));
                        return;
                    }
                };

                if state.drm_device_registry.get(node).is_none() {
                    if let Err(error) = state.drm_device_registry.register(node, path.clone()) {
                        state.error = Some(io::Error::other(error));
                        return;
                    }
                }

                if state.gpu_manager.get_device(&path).is_none() {
                    if let Err(error) = state.gpu_manager.register_device(path.clone()) {
                        state.drm_device_registry.remove(node);
                        state.error = Some(io::Error::other(error));
                        return;
                    }
                }

                if !state.udev_device_ids.iter().any(|(id, _)| *id == node) {
                    state.udev_device_ids.push((node, path));
                }
            }

            UdevEvent::Changed { device_id } => {
                let node = match DrmNode::from_dev_id(device_id as u64) {
                    Ok(node) => node,
                    Err(error) => {
                        state.error = Some(io::Error::other(error));
                        return;
                    }
                };

                if state.udev_device_ids.iter().any(|(id, _)| *id == node)
                    && !state.udev_changed_devices.contains(&node)
                {
                    state.udev_changed_devices.push(node);
                }
            }

            UdevEvent::Removed { device_id } => {
                let node = match DrmNode::from_dev_id(device_id as u64) {
                    Ok(node) => node,
                    Err(error) => {
                        state.error = Some(io::Error::other(error));
                        return;
                    }
                };

                if let Some(index) = state.udev_device_ids.iter().position(|(id, _)| *id == node) {
                    let (_, path) = state.udev_device_ids.remove(index);

                    state.drm_device_registry.remove(node);
                    state.gpu_manager.remove_device(&path);
                }

                state
                    .udev_changed_devices
                    .retain(|changed_node| *changed_node != node);
            }
        })
        .map_err(|_| io::Error::other("Failed to register the Udev event backend"))?;

    Ok(initial_devices)
}
