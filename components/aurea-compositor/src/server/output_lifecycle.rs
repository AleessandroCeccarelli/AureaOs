// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Native DRM/KMS output initialization and validation.

use std::io;

use crate::{drm::surfaces::DrmSurfaceManager, server::state::WaylandState};

/// Initializes KMS surfaces and their matching logical Wayland outputs.
pub(super) fn initialize_outputs(state: &mut WaylandState) -> io::Result<()> {
    let nodes = state
        .drm_devices
        .iter()
        .map(|(node, _)| *node)
        .collect::<Vec<_>>();

    for node in nodes {
        if !super::gbm_lifecycle::output_presentation_ready(state.gbm_devices.contains(node)) {
            eprintln!("AureaOS: skipping outputs for {node}: GBM presentation is unavailable");
            continue;
        }

        let manager = {
            let Some(device) = state.drm_devices.get_mut(node) else {
                continue;
            };

            match DrmSurfaceManager::initialize(device) {
                Ok(manager) => manager,
                Err(failure) => {
                    let (partial_manager, cause) = failure.into_parts();
                    eprintln!("AureaOS: KMS surfaces unavailable for DRM device {node}: {cause}");
                    // Retain partially initialized surfaces for safe teardown.
                    state.drm_surfaces.insert(node, partial_manager);
                    continue;
                }
            }
        };

        let output_result = {
            let Some(device) = state.drm_devices.get(node) else {
                continue;
            };
            crate::drm::outputs::initialize_drm_outputs(device, state.compositor.output_manager())
        };

        match output_result {
            Ok(0) => {
                eprintln!("AureaOS: DRM device {node} has no connected, usable display connectors");
                state.drm_surfaces.insert(node, manager);
            }
            Ok(_) => {
                state.drm_surfaces.insert(node, manager);
            }
            Err(error) => {
                eprintln!("AureaOS: logical outputs unavailable for DRM device {node}: {error}");
                // Keep native surface ownership even when logical registration fails.
                state.drm_surfaces.insert(node, manager);
            }
        }
    }

    validate_active_outputs(state)
}

/// Rejects logical outputs that cannot be presented by a retained KMS surface.
fn validate_active_outputs(state: &mut WaylandState) -> io::Result<()> {
    if state.drm_session.is_none() || !state.session_active {
        return Ok(());
    }

    let output_manager = state.compositor.output_manager();
    if output_manager.active_output_count() == 0 {
        return Err(io::Error::other(
            "No usable display output was initialized: check DRM/KMS, GBM, Vulkan rendering support, and connected monitors",
        ));
    }

    let unbacked_outputs = output_manager
        .registry()
        .iter()
        .filter(|output| output.is_enabled())
        .filter(|output| {
            !state
                .drm_surfaces
                .values()
                .any(|surfaces| surfaces.get(output.id()).is_some())
        })
        .map(|output| output.id())
        .collect::<Vec<_>>();

    if unbacked_outputs.is_empty() {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "Enabled Wayland outputs have no native DRM/KMS surface: {}",
        unbacked_outputs
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )))
}
