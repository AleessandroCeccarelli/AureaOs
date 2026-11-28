// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Udev discovery and DRM hotplug lifecycle coordination.

mod activation;
pub(super) mod device;
mod outputs;
mod tracking;

use std::{io, path::PathBuf};

use smithay::{
    backend::{
        drm::DrmNode,
        udev::{UdevBackend, UdevEvent},
    },
    reexports::calloop::LoopHandle,
};

use super::state::WaylandState;

use self::device::{add_device, changed_device, remove_device};

pub(super) use self::activation::activate_discovered_devices;
#[cfg(test)]
pub(super) use self::outputs::{output_belongs_to_device, restore_before_publish};
#[cfg(test)]
pub(super) use self::tracking::{
    forget_changed_device, forget_device, queue_changed_device, retry_pending_changes,
    take_notifier_token, track_device,
};
/// Registers Udev monitoring and returns the initial DRM device inventory.
pub(super) fn register_udev_events(
    loop_handle: &LoopHandle<'_, WaylandState>,
    seat: &str,
) -> io::Result<Vec<(DrmNode, PathBuf)>> {
    let backend = UdevBackend::new(seat).map_err(io::Error::other)?;

    let initial_devices = backend
        .device_list()
        .map(|(device_id, path)| {
            DrmNode::from_dev_id(device_id)
                .map(|node| (node, path.to_path_buf()))
                .map_err(io::Error::other)
        })
        .collect::<io::Result<Vec<_>>>()?;

    loop_handle
        .insert_source(backend, |event: UdevEvent, _, state| {
            let result = match event {
                UdevEvent::Added { device_id, path } => DrmNode::from_dev_id(device_id)
                    .map_err(io::Error::other)
                    .and_then(|node| add_device(state, node, path)),
                UdevEvent::Changed { device_id } => DrmNode::from_dev_id(device_id)
                    .map_err(io::Error::other)
                    .and_then(|node| changed_device(state, node)),
                UdevEvent::Removed { device_id } => {
                    // The sysfs identity check in DrmNode::from_dev_id can fail
                    // after unplug, so resolve removal against identities retained
                    // while the device was present.
                    let node = state
                        .udev_device_ids
                        .iter()
                        .map(|(node, _)| *node)
                        .find(|node| node.dev_id() == device_id);

                    match node {
                        Some(node) => remove_device(state, node),
                        None => Ok(()),
                    }
                }
            };

            if let Err(error) = result {
                eprintln!("AureaOS: Udev DRM lifecycle event failed: {error}");
            }
        })
        .map_err(|_| io::Error::other("Failed to register the Udev event backend"))?;

    Ok(initial_devices)
}

#[cfg(test)]
#[path = "../../tests/unit/server_udev_lifecycle.rs"]
mod tests;
