// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! DRM device open, graphics initialization and hotplug lifecycle.

use std::{io, path::PathBuf};

use smithay::backend::drm::DrmNode;

use crate::{drm::device::DrmDevice, renderer::RenderPreference, vulkan::VulkanBackend};

use super::{
    super::state::WaylandState,
    outputs::{refresh_device_outputs, remove_device_outputs, restore_before_publish},
    tracking::{
        forget_changed_device, forget_device, queue_changed_device, retry_pending_changes,
        take_notifier_token, track_device,
    },
};

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
pub(super) fn open_and_integrate(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
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

    if let Err(error) = super::super::gbm_lifecycle::register_gbm_device(state, node) {
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
        super::super::gbm_lifecycle::remove_gbm_device(state, node);
        state.vulkan_backends.remove(&node);
        state.drm_devices.remove(node);

        return Err(error);
    }

    Ok(())
}

/// Initializes the Vulkan backend for one DRM device.
///
/// Failure is isolated to the affected GPU.
pub(super) fn initialize_vulkan_backend(state: &mut WaylandState, node: DrmNode) {
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

pub(super) fn initialize_drm_surfaces(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    if !super::super::gbm_lifecycle::graphics_dependencies_ready(
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

    let initialization = {
        let Some(device) = state.drm_devices.get_mut(node) else {
            return Err(io::Error::other(format!(
                "DRM device {node} disappeared during surface initialization"
            )));
        };
        crate::drm::surfaces::DrmSurfaceManager::initialize(device)
    };

    let manager = match initialization {
        Ok(manager) => manager,
        Err(failure) => {
            let (partial_manager, cause) = failure.into_parts();
            state.drm_surfaces.insert(node, partial_manager);
            return Err(io::Error::other(cause.to_string()));
        }
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

fn remove_device_notifier(state: &mut WaylandState, node: DrmNode) {
    if let (Some(token), Some(handle)) = (
        take_notifier_token(&mut state.drm_notifiers, &node),
        state.loop_handle.as_ref(),
    ) {
        handle.remove(token);
    }
}

pub(super) fn changed_device(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
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
            let restore_result = restore_before_publish(
                state,
                |state| initialize_drm_surfaces(state, node),
                |state| refresh_device_outputs(state, node),
            );

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

pub(super) fn remove_device(state: &mut WaylandState, node: DrmNode) -> io::Result<()> {
    remove_device_outputs(state, node);

    // This path runs only after Udev has confirmed device removal. The
    // kernel has detached the DRM device and torn down its KMS state, so do
    // not issue further KMS ioctls against the unplugged device. Release
    // userspace scanout references first, then their GBM/Vulkan/DRM owners.
    remove_device_notifier(state, node);
    if let Some(surfaces) = state.drm_surfaces.get_mut(&node) {
        surfaces.release_after_device_removal();
    }
    state.drm_surfaces.remove(&node);

    // Release GBM resources before removing the DRM device they depend on.
    super::super::gbm_lifecycle::remove_gbm_device(state, node);

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
