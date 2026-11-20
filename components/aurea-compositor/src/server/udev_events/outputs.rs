// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Logical output publication and KMS-before-Wayland restoration ordering.

use std::io;

use smithay::backend::drm::DrmNode;

use crate::{drm::outputs::reconcile_drm_outputs_with_identity, output::OutputId};

use super::super::state::WaylandState;

/// Restores native scanout surfaces before publishing their logical outputs.
///
/// Keeping this order in one helper prevents hotplug/session recovery from
/// advertising outputs before a working KMS path exists.
pub(in crate::server) fn restore_before_publish<C, E>(
    state: &mut C,
    restore_surfaces: impl FnOnce(&mut C) -> Result<(), E>,
    publish_outputs: impl FnOnce(&mut C) -> Result<(), E>,
) -> Result<(), E> {
    restore_surfaces(state)?;
    publish_outputs(state)
}

pub(in crate::server) fn output_ids_for_device(
    state: &mut WaylandState,
    node: DrmNode,
) -> Vec<OutputId> {
    let device_id = node.dev_id() as u64;

    state
        .compositor
        .output_manager()
        .registry()
        .iter()
        .filter(|output| output_belongs_to_device(output.id(), device_id))
        .map(|output| output.id())
        .collect()
}

pub(in crate::server) fn remove_device_outputs(state: &mut WaylandState, node: DrmNode) {
    let ids = output_ids_for_device(state, node);

    for id in ids {
        state.drm_display_identities.remove(id);
        state.compositor.unpublish_wayland_output(id);
        state.compositor.forget_output_frame_state(id);
        state.compositor.output_manager().unregister(id);
    }

    // A whole DRM device may disappear while the pointer is on one of its
    // outputs. Reposition it only after every output from that device is gone.
    state.compositor.center_pointer_on_primary_output();
}

pub(in crate::server) fn refresh_device_outputs(
    state: &mut WaylandState,
    node: DrmNode,
) -> io::Result<()> {
    let owned_ids = output_ids_for_device(state, node);
    let before = state.compositor.output_manager().clone();

    let (changes, identity_evidence) = {
        let Some(device) = state.drm_devices.get(node) else {
            return Ok(());
        };

        reconcile_drm_outputs_with_identity(device, state.compositor.output_manager(), &owned_ids)
            .map_err(io::Error::other)?
    };

    state.drm_display_identities.replace_device(node.dev_id() as u64, identity_evidence);

    let display = state.display.handle();

    state
        .compositor
        .publish_output_reconciliation(&display, before, &changes)
        .map_err(io::Error::other)?;
    state.compositor.center_pointer_on_primary_output();
    Ok(())
}

pub(in crate::server) fn output_belongs_to_device(output_id: OutputId, device_id: u64) -> bool {
    (output_id >> 32) as u64 == device_id
}
