
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! GBM device lifecycle coordination for the Wayland server.

use std::io;

use smithay::backend::drm::DrmNode;

use crate::drm::gbm::GbmDeviceManager;

use super::state::WaylandState;

/// Initializes GBM for every currently opened DRM device.
///
/// A GBM initialization failure is isolated to its device and does not
/// prevent other GPUs from being initialized.
pub(super) fn initialize_gbm_devices(state: &mut WaylandState) {
    let nodes = state
        .drm_devices
        .iter()
        .map(|(node, _)| *node)
        .collect::<Vec<_>>();

    for node in nodes {
        if let Err(error) = register_gbm_device(state, node) {
            eprintln!(
                "AureaOS: GBM initialization unavailable for DRM device {node}: {error}"
            );
        }
    }
}

/// Initializes GBM for one opened DRM device.
pub(super) fn register_gbm_device(
    state: &mut WaylandState,
    node: DrmNode,
) -> io::Result<()> {
    if state.gbm_devices.contains(node) {
        return Ok(());
    }

    let drm_fd = state
        .drm_devices
        .get(node)
        .map(|device| device.fd().clone())
        .ok_or_else(|| {
            io::Error::other(format!(
                "Cannot initialize GBM: DRM device {node} is not managed"
            ))
        })?;

    state
        .gbm_devices
        .register(node, &drm_fd)
        .map_err(io::Error::other)
}

/// Removes GBM resources associated with one DRM device.
pub(super) fn remove_gbm_device(state: &mut WaylandState, node: DrmNode) {
    state.gbm_devices.remove(node);
}

/// Releases every GBM device.
///
/// This must happen before the DRM session is paused.
pub(super) fn clear_gbm_devices(state: &mut WaylandState) {
    state.gbm_devices.clear();
}

/// Creates an empty GBM manager.
///
/// Kept here to centralize server-side GBM initialization.
pub(super) fn new_gbm_device_manager() -> GbmDeviceManager {
    GbmDeviceManager::new()
}
