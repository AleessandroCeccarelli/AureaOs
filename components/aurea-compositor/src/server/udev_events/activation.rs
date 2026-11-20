// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Restores already discovered DRM devices when the compositor session resumes.

use std::collections::HashSet;

use smithay::backend::drm::DrmNode;

use super::{
    super::state::WaylandState,
    device::{initialize_drm_surfaces, initialize_vulkan_backend, open_and_integrate},
    outputs::{refresh_device_outputs, remove_device_outputs, restore_before_publish},
    tracking::{forget_changed_device, retry_pending_changes},
};

/// Restores graphics backends for existing devices, integrates devices
/// discovered while libseat was inactive and processes deferred output changes.
pub(super) fn activate_discovered_devices(state: &mut WaylandState) {
    if !state.session_active {
        return;
    }

    // Restore GBM for devices that remained registered across session pause.
    super::super::gbm_lifecycle::initialize_gbm_devices(state);

    // Restore Vulkan for existing DRM devices. The pause handler releases
    // these backends, so they must be recreated after session activation.
    let existing_nodes = state
        .drm_devices
        .iter()
        .map(|(node, _)| *node)
        .collect::<Vec<_>>();

    let mut restored_nodes = HashSet::new();

    for node in existing_nodes {
        initialize_vulkan_backend(state, node);
        match initialize_drm_surfaces(state, node) {
            Ok(()) => {
                restored_nodes.insert(node);
            }
            Err(error) => {
                eprintln!("AureaOS: unable to restore KMS surfaces for {node}: {error}");
                // The old output globals are no longer backed by active KMS
                // surfaces; withdraw them until the device becomes ready again.
                remove_device_outputs(state, node);
            }
        }
    }

    // Integrate devices discovered while the session was inactive.
    let discovered = state
        .udev_device_ids
        .iter()
        .map(|(node, _)| *node)
        .collect::<Vec<_>>();

    for node in discovered {
        if !state.drm_devices.contains(node) {
            match open_and_integrate(state, node) {
                Ok(()) if state.drm_devices.contains(node) => {
                    // open_and_integrate already restores KMS and publishes
                    // outputs, so no deferred connector refresh is needed.
                    forget_changed_device(&mut state.udev_changed_devices, node);
                }
                Ok(()) => {}
                Err(error) => {
                    eprintln!(
                        "AureaOS: deferred DRM device integration failed for {node}: {error}"
                    );
                }
            }
        }
    }

    // Retry connector changes that arrived while the session was inactive.
    let managed = state.drm_devices.iter().map(|(node, _)| *node).collect();

    let pending = std::mem::take(&mut state.udev_changed_devices);

    let (pending, failures) = retry_pending_changes(pending, &managed, |node| {
        if restored_nodes.contains(&node) {
            // Existing devices were already restored above; only reconcile
            // their logical outputs now.
            refresh_device_outputs(state, node)
        } else {
            // A device whose initial restore failed must establish KMS before
            // its refreshed logical outputs can be published.
            restore_before_publish(
                state,
                |state| initialize_drm_surfaces(state, node),
                |state| refresh_device_outputs(state, node),
            )
        }
    });

    state.udev_changed_devices = pending;

    for (node, error) in failures {
        // A failed KMS restore or output reconciliation must not leave stale
        // Wayland globals visible to clients.
        remove_device_outputs(state, node);
        eprintln!("AureaOS: deferred DRM output refresh failed for {node}: {error}");
    }
}
