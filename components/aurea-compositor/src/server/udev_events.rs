// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Udev discovery and DRM hotplug lifecycle coordination.

use std::{io, path::PathBuf};

use smithay::{
    backend::{
        drm::DrmNode,
        udev::{UdevBackend, UdevEvent},
    },
    reexports::calloop::LoopHandle,
};

use crate::{
    drm::{device::DrmDevice, outputs::reconcile_drm_outputs},
    output::OutputId,
    renderer::RenderPreference,
    vulkan::VulkanBackend,
};

use super::state::WaylandState;

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

/// Registers discovery metadata once, then integrates the opened device if the
/// seat is active. Open failures are non-fatal so unrelated GPUs keep running.
pub(super) fn add_device(state: &mut WaylandState, node: DrmNode, path: PathBuf) -> io::Result<()> {
    if !state
        .udev_device_ids
        .iter()
        .any(|(known, _)| *known == node)
    {
        let new_registry_entry = state.drm_device_registry.get(node).is_none();

        if new_registry_entry {
            state
                .drm_device_registry
                .register(node, path.clone())
                .map_err(io::Error::other)?;
        }

        if state.gpu_manager.get_device(&path).is_none() {
            if let Err(error) = state.gpu_manager.register_device(path.clone()) {
                if new_registry_entry {
                    state.drm_device_registry.remove(node);
                }

                return Err(io::Error::other(error));
            }
        }

        track_device(&mut state.udev_device_ids, node, path);
    }

    if state.session_active && !state.drm_devices.contains(node) {
        open_and_integrate(state, node)?;
    }

    Ok(())
}

/// Opens one discovered device, initializes its graphics resources,
/// installs its notifier and registers output globals.
fn open_and_integrate(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    let Some((_, path)) = state
        .udev_device_ids
        .iter()
        .find(|(known, _)| *known == node)
    else {
        return Ok(());
    };

    let path = path.clone();

    let Some(session) = state.drm_session.as_mut() else {
        return Ok(());
    };

    let device = match DrmDevice::open(session, &path) {
        Ok(device) => device,
        Err(error) => {
            eprintln!("AureaOS: unable to open DRM device {node}: {error}");
            return Ok(());
        }
    };

    if device.node().map_err(io::Error::other)? != node {
        return Err(io::Error::other(format!(
            "Opened DRM device identity does not match Udev node {node}"
        )));
    }

    state
        .drm_devices
        .register(device)
        .map_err(io::Error::other)?;

    if let Err(error) = super::gbm_lifecycle::register_gbm_device(state, node) {
        eprintln!("AureaOS: GBM unavailable for newly opened DRM device {node}: {error}");
    }

    initialize_vulkan_backend(state, node);

    let result = (|| {
        if !state.gbm_devices.contains(node) {
            return Err(io::Error::other(format!(
                "DRM device {node} has no usable GBM allocator"
            )));
        }
        if !state.vulkan_backends.contains_key(&node) {
            return Err(io::Error::other(format!(
                "DRM device {node} has no compatible Vulkan backend"
            )));
        }

        register_device_notifier(state, node)?;
        initialize_drm_surfaces(state, node)?;
        refresh_device_outputs(state, node)
    })();

    if let Err(error) = result {
        remove_device_outputs(state, node);

        // A failed KMS clear means the kernel may still reference buffers
        // owned by this surface manager. Do not drop the manager or any of
        // its GBM/Vulkan/DRM dependencies on this rollback path.
        if let Some(surfaces) = state.drm_surfaces.get_mut(&node) {
            if let Err(clear_error) = surfaces.clear() {
                return Err(io::Error::other(format!(
                    "{error}; rollback could not disable KMS surfaces for {node}: {clear_error}; graphics dependencies remain retained"
                )));
            }
        }

        state.drm_surfaces.remove(&node);
        remove_device_notifier(state, node);
        super::gbm_lifecycle::remove_gbm_device(state, node);
        state.vulkan_backends.remove(&node);
        state.drm_devices.remove(node);

        return Err(error);
    }

    Ok(())
}

/// Initializes the Vulkan backend for one DRM device.
///
/// Failure is isolated to the affected GPU.
fn initialize_vulkan_backend(state: &mut WaylandState, node: DrmNode) {
    if state.vulkan_backends.contains_key(&node) {
        return;
    }

    match VulkanBackend::new_for_drm_node(node, RenderPreference::Automatic) {
        Ok(backend) => {
            state.vulkan_backends.insert(node, backend);
        }
        Err(error) => {
            eprintln!("AureaOS: Vulkan backend unavailable for DRM device {node}: {error}");
        }
    }
}

fn initialize_drm_surfaces(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    if !super::gbm_lifecycle::graphics_dependencies_ready(
        state.gbm_devices.contains(node),
        state.vulkan_backends.contains_key(&node),
    ) {
        return Err(io::Error::other(format!(
            "DRM device {node} is not graphics-ready (GBM/Vulkan unavailable)"
        )));
    }

    // Never drop an existing manager until every KMS surface was disabled.
    if let Some(existing) = state.drm_surfaces.get_mut(&node) {
        existing.clear().map_err(|error| {
            io::Error::other(format!(
                "cannot replace DRM surfaces for {node}; previous surfaces remain owned: {error}"
            ))
        })?;
    }
    state.drm_surfaces.remove(&node);

    let manager = {
        let Some(device) = state.drm_devices.get_mut(node) else {
            return Err(io::Error::other(format!(
                "DRM device {node} disappeared during surface initialization"
            )));
        };
        crate::drm::surfaces::DrmSurfaceManager::initialize(device).map_err(io::Error::other)?
    };

    state.drm_surfaces.insert(node, manager);
    state.compositor.request_all_output_frames();
    Ok(())
}

fn register_device_notifier(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    if state.drm_notifiers.contains_key(&node) {
        return Ok(());
    }

    let handle = state
        .loop_handle
        .clone()
        .ok_or_else(|| io::Error::other("Calloop handle is unavailable"))?;

    let notifier = state
        .drm_devices
        .get_mut(node)
        .ok_or_else(|| io::Error::other(format!("DRM device {node} is not managed")))?
        .take_notifier()
        .map_err(io::Error::other)?;

    let token = handle
        .insert_source(notifier, move |event, _, state| match event {
            smithay::backend::drm::DrmEvent::VBlank(crtc) => {
                if let Some(surfaces) = state.drm_surfaces.get_mut(&node) {
                    match surfaces.on_vblank(node, crtc) {
                        Ok(Some(released)) => {
                            if let Some(Err(error)) = surfaces.retire_released_frame(crtc, released)
                            {
                                eprintln!(
                                    "AureaOS: failed to reacquire released scanout frame on {node}: {error}"
                                );
                            }
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!(
                                "AureaOS: DRM presentation completion failed for {node}: {error}"
                            );
                        }
                    }
                }
            }
            smithay::backend::drm::DrmEvent::Error(error) => {
                state.error = Some(io::Error::other(format!("DRM device {node}: {error}")));
            }
        })
        .map_err(io::Error::other)?;

    state.drm_notifiers.insert(node, token);

    Ok(())
}

fn output_ids_for_device(state: &mut WaylandState, node: DrmNode) -> Vec<OutputId> {
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

fn remove_device_outputs(state: &mut WaylandState, node: DrmNode) {
    let ids = output_ids_for_device(state, node);

    for id in ids {
        state.compositor.unpublish_wayland_output(id);
        state.compositor.output_manager().unregister(id);
    }
}

fn remove_device_notifier(state: &mut WaylandState, node: DrmNode) {
    if let (Some(token), Some(handle)) = (
        take_notifier_token(&mut state.drm_notifiers, &node),
        state.loop_handle.as_ref(),
    ) {
        handle.remove(token);
    }
}

fn take_notifier_token<K: Eq + std::hash::Hash, V>(
    map: &mut std::collections::HashMap<K, V>,
    key: &K,
) -> Option<V> {
    map.remove(key)
}

fn refresh_device_outputs(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    let owned_ids = output_ids_for_device(state, node);
    let before = state.compositor.output_manager().clone();

    let changes = {
        let Some(device) = state.drm_devices.get(node) else {
            return Ok(());
        };

        reconcile_drm_outputs(device, state.compositor.output_manager(), &owned_ids)
            .map_err(io::Error::other)?
    };

    let display = state.display.handle();

    state
        .compositor
        .publish_output_reconciliation(&display, before, &changes)
        .map_err(io::Error::other)
}

fn changed_device(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    if !state
        .udev_device_ids
        .iter()
        .any(|(known, _)| *known == node)
    {
        return Ok(());
    }

    queue_changed_device(&mut state.udev_changed_devices, node);

    if state.session_active {
        if state.drm_devices.contains(node) {
            let restore_result = initialize_drm_surfaces(state, node)
                .and_then(|()| refresh_device_outputs(state, node));

            if let Err(error) = restore_result {
                // Do not leave stale Wayland globals advertising outputs whose
                // KMS surfaces or logical output state could not be restored.
                remove_device_outputs(state, node);
                return Err(error);
            }

            forget_changed_device(&mut state.udev_changed_devices, node);
        } else {
            open_and_integrate(state, node)?;

            if state.drm_devices.contains(node) {
                forget_changed_device(&mut state.udev_changed_devices, node);
            }
        }
    }

    Ok(())
}

fn remove_device(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    remove_device_outputs(state, node);

    // A Udev removal event does not prove that every KMS disable operation
    // completed. Keep the surface manager and all resources it depends on
    // alive when clear() fails; dropping them here would discard userspace
    // ownership while the kernel may still reference a scanout framebuffer.
    if let Some(surfaces) = state.drm_surfaces.get_mut(&node) {
        surfaces.clear().map_err(|error| {
            io::Error::other(format!(
                "DRM device {node} was removed but KMS surfaces could not be cleared; graphics dependencies remain retained: {error}"
            ))
        })?;
    }

    state.drm_surfaces.remove(&node);
    remove_device_notifier(state, node);

    // Release GBM resources before removing the DRM device they depend on.
    super::gbm_lifecycle::remove_gbm_device(state, node);

    state.vulkan_backends.remove(&node);
    state.drm_devices.remove(node);
    state.drm_device_registry.remove(node);

    forget_changed_device(&mut state.udev_changed_devices, node);

    if let Some(path) = forget_device(&mut state.udev_device_ids, node) {
        if !state
            .udev_device_ids
            .iter()
            .any(|(_, remaining)| remaining == &path)
        {
            state.gpu_manager.remove_device(&path);
        }
    }

    Ok(())
}

fn track_device<N: Copy + Eq>(devices: &mut Vec<(N, PathBuf)>, node: N, path: PathBuf) -> bool {
    if devices.iter().any(|(known, _)| *known == node) {
        return false;
    }

    devices.push((node, path));

    true
}

fn forget_device<N: Copy + Eq>(devices: &mut Vec<(N, PathBuf)>, node: N) -> Option<PathBuf> {
    devices
        .iter()
        .position(|(known, _)| *known == node)
        .map(|index| devices.remove(index).1)
}

fn queue_changed_device<N: Copy + Eq>(changed: &mut Vec<N>, node: N) {
    if !changed.contains(&node) {
        changed.push(node);
    }
}

fn forget_changed_device<N: Copy + Eq>(changed: &mut Vec<N>, node: N) {
    changed.retain(|known| *known != node);
}

fn output_belongs_to_device(output_id: OutputId, device_id: u64) -> bool {
    (output_id >> 32) as u64 == device_id
}

fn retry_pending_changes<N, E, F>(
    pending: Vec<N>,
    managed: &std::collections::HashSet<N>,
    mut refresh: F,
) -> (Vec<N>, Vec<(N, E)>)
where
    N: Copy + Eq + std::hash::Hash,
    F: FnMut(N) -> Result<(), E>,
{
    let mut retained = Vec::new();
    let mut failures = Vec::new();

    for node in pending {
        if !managed.contains(&node) {
            retained.push(node);
            continue;
        }

        match refresh(node) {
            Ok(()) => {}
            Err(error) => {
                retained.push(node);
                failures.push((node, error));
            }
        }
    }

    (retained, failures)
}

#[cfg(test)]
#[path = "../../tests/unit/server_udev_lifecycle.rs"]
mod tests;

/// Restores graphics backends for existing devices, integrates devices
/// discovered while libseat was inactive and processes deferred output changes.
pub(super) fn activate_discovered_devices(state: &mut WaylandState) {
    if !state.session_active {
        return;
    }

    // Restore GBM for devices that remained registered across session pause.
    super::gbm_lifecycle::initialize_gbm_devices(state);

    // Restore Vulkan for existing DRM devices. The pause handler releases
    // these backends, so they must be recreated after session activation.
    let existing_nodes = state
        .drm_devices
        .iter()
        .map(|(node, _)| *node)
        .collect::<Vec<_>>();

    for node in existing_nodes {
        initialize_vulkan_backend(state, node);
        if let Err(error) = initialize_drm_surfaces(state, node) {
            eprintln!("AureaOS: unable to restore KMS surfaces for {node}: {error}");
            // The old output globals are no longer backed by active KMS
            // surfaces; withdraw them until the device becomes ready again.
            remove_device_outputs(state, node);
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
            if let Err(error) = open_and_integrate(state, node) {
                eprintln!("AureaOS: deferred DRM device integration failed for {node}: {error}");
            }
        }
    }

    // Retry connector changes that arrived while the session was inactive.
    let managed = state.drm_devices.iter().map(|(node, _)| *node).collect();

    let pending = std::mem::take(&mut state.udev_changed_devices);

    let (pending, failures) = retry_pending_changes(pending, &managed, |node| {
        // Recreate and validate native KMS surfaces before publishing the
        // refreshed logical outputs. Publishing first can expose outputs that
        // have no working scanout path after session activation.
        initialize_drm_surfaces(state, node).and_then(|()| refresh_device_outputs(state, node))
    });

    state.udev_changed_devices = pending;

    for (node, error) in failures {
        // A failed KMS restore or output reconciliation must not leave stale
        // Wayland globals visible to clients.
        remove_device_outputs(state, node);
        eprintln!("AureaOS: deferred DRM output refresh failed for {node}: {error}");
    }
}
