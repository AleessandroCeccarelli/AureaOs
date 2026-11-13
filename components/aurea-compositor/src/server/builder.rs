// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server construction and backend initialization.

use std::{
    io,
    os::fd::AsFd,
    sync::{Arc, Mutex},
};

use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmEvent;

#[cfg(feature = "drm")]
use crate::{
    drm::{DrmSession, device::DrmDevice},
    renderer::RenderPreference,
    vulkan::VulkanBackend,
};

use crate::AureaCompositor;

use super::{
    display_events::register_display_events, socket::register_wayland_socket, state::WaylandState,
};

#[cfg(feature = "drm")]
use super::session_events::register_session_events;

/// Builds and initializes the Wayland server.
///
/// This function owns the complete startup sequence. The returned event loop
/// and state are transferred to `WaylandServer` for runtime execution.
pub(super) fn build(
    socket_name: &str,
    #[cfg(feature = "drm")] drm_session: Option<DrmSession>,
    #[cfg(feature = "drm")] drm_device: Option<DrmDevice>,
) -> io::Result<(EventLoop<'static, WaylandState>, WaylandState)> {
    let event_loop = EventLoop::<WaylandState>::try_new().map_err(io::Error::other)?;

    let display = Display::<AureaCompositor>::new().map_err(io::Error::other)?;
    let display_handle = display.handle();
    let compositor = AureaCompositor::new(&display_handle);

    #[cfg(feature = "drm")]
    let session_active = drm_session
        .as_ref()
        .map(DrmSession::is_active)
        .unwrap_or(true);

    #[cfg(feature = "drm")]
    let mut drm_devices = crate::drm::device_manager::DrmDeviceManager::new();

    #[cfg(feature = "drm")]
    if let Some(device) = drm_device {
        drm_devices.register(device).map_err(io::Error::other)?;
    }

    let mut state = WaylandState {
        display,
        compositor,
        clients: Vec::new(),
        disconnected_clients: Arc::new(Mutex::new(Vec::new())),
        error: None,

        #[cfg(feature = "drm")]
        drm_session,

        #[cfg(feature = "drm")]
        drm_devices,

        #[cfg(feature = "drm")]
        drm_surfaces: std::collections::HashMap::new(),

        #[cfg(feature = "drm")]
        gbm_devices: crate::drm::gbm::GbmDeviceManager::new(),

        #[cfg(feature = "drm")]
        drm_notifiers: std::collections::HashMap::new(),

        #[cfg(feature = "drm")]
        loop_handle: None,

        #[cfg(feature = "drm")]
        session_active,

        #[cfg(feature = "drm")]
        gpu_manager: crate::gpu::GpuManager::new(),

        #[cfg(feature = "drm")]
        drm_device_registry: crate::drm::registry::DrmDeviceRegistry::new(),

        #[cfg(feature = "drm")]
        udev_device_ids: Vec::new(),

        #[cfg(feature = "drm")]
        udev_changed_devices: Vec::new(),

        #[cfg(feature = "drm")]
        vulkan_backends: std::collections::HashMap::new(),
    };

    state.compositor.initialize_keyboard()?;

    let loop_handle = event_loop.handle();

    #[cfg(feature = "drm")]
    {
        state.loop_handle = Some(loop_handle.clone());

        if let Some(session) = state.drm_session.as_ref() {
            super::input::register_input_backend(&loop_handle, session)?;
        }
    }

    // Register Udev monitoring and obtain the initial DRM inventory.
    #[cfg(feature = "drm")]
    if let Some(seat) = state.drm_session.as_ref().map(DrmSession::seat) {
        let initial_devices = super::udev_events::register_udev_events(&loop_handle, &seat)?;

        for (node, path) in initial_devices {
            state
                .drm_device_registry
                .register(node, path.clone())
                .map_err(io::Error::other)?;

            state
                .gpu_manager
                .register_device(path.clone())
                .map_err(io::Error::other)?;

            state.udev_device_ids.push((node, path));
        }
    }

    // Open every discovered device that can be acquired through the active
    // seat. A secondary GPU may be render-only or unavailable for KMS, so
    // failures here do not prevent the primary compositor from starting.
    #[cfg(feature = "drm")]
    if state.session_active {
        if let Some(session) = state.drm_session.as_mut() {
            let discovered = state.udev_device_ids.clone();

            for (node, path) in discovered {
                if state.drm_devices.contains(node) {
                    continue;
                }

                let Ok(device) = DrmDevice::open(session, &path) else {
                    continue;
                };

                if device.node().ok() != Some(node) {
                    continue;
                }

                let _ = state.drm_devices.register(device);
            }
        }
    }

    // Initialize GBM independently for every opened DRM device.
    #[cfg(feature = "drm")]
    super::gbm_lifecycle::initialize_gbm_devices(&mut state);

    // Initialize one Vulkan backend for each opened DRM device.
    //
    // A GPU without a compatible Vulkan physical device is skipped.
    // Its failure must not prevent other devices or the Wayland server
    // from starting.
    #[cfg(feature = "drm")]
    {
        let nodes = state
            .drm_devices
            .iter()
            .map(|(node, _)| *node)
            .collect::<Vec<_>>();

        for node in nodes {
            match VulkanBackend::new_for_drm_node(node, RenderPreference::Automatic) {
                Ok(backend) => {
                    state.vulkan_backends.insert(node, backend);
                }
                Err(error) => {
                    eprintln!("AureaOS: Vulkan backend unavailable for DRM device {node}: {error}");
                }
            }
        }
    }

    // Register outputs only after every required rendering dependency and
    // the native KMS surfaces have been initialized. This prevents publishing
    // logical outputs that cannot be scanned out.
    #[cfg(feature = "drm")]
    {
        let nodes = state
            .drm_devices
            .iter()
            .map(|(node, _)| *node)
            .collect::<Vec<_>>();

        for node in nodes {
            if !super::gbm_lifecycle::graphics_dependencies_ready(
                state.gbm_devices.contains(node),
                state.vulkan_backends.contains_key(&node),
            ) {
                eprintln!(
                    "AureaOS: skipping outputs for {node}: GBM or Vulkan backend is unavailable"
                );
                continue;
            }

            let manager = {
                let Some(device) = state.drm_devices.get_mut(node) else {
                    continue;
                };

                match crate::drm::surfaces::DrmSurfaceManager::initialize(device) {
                    Ok(manager) => manager,
                    Err(failure) => {
                        let (partial_manager, cause) = failure.into_parts();
                        eprintln!(
                            "AureaOS: KMS surfaces unavailable for DRM device {node}: {cause}"
                        );
                        // Preserve any surfaces already created before a later
                        // connector failed. Teardown must retain their KMS owners.
                        state.drm_surfaces.insert(node, partial_manager);
                        continue;
                    }
                }
            };

            let output_result = {
                let Some(device) = state.drm_devices.get(node) else {
                    continue;
                };
                crate::drm::outputs::initialize_drm_outputs(
                    device,
                    state.compositor.output_manager(),
                )
            };

            match output_result {
                Ok(_) => {
                    state.drm_surfaces.insert(node, manager);
                }
                Err(error) => {
                    eprintln!(
                        "AureaOS: logical outputs unavailable for DRM device {node}: {error}"
                    );
                    // Keep ownership of the native surfaces even when logical
                    // output registration fails; shutdown must still clear them.
                    state.drm_surfaces.insert(node, manager);
                }
            }
        }
    }

    // Publish registered outputs as Wayland globals.
    state
        .compositor
        .publish_wayland_outputs(&display_handle)
        .map_err(io::Error::other)?;

    // Register the listening socket and incoming client handling.
    register_wayland_socket(&loop_handle, socket_name)?;

    // Register native session activation and pause events.
    #[cfg(feature = "drm")]
    if let Some(session) = state.drm_session.as_mut() {
        register_session_events(&loop_handle, session)?;
    }

    // Register DRM device events with Calloop.
    #[cfg(feature = "drm")]
    {
        let nodes = state
            .drm_devices
            .iter()
            .map(|(node, _)| *node)
            .collect::<Vec<_>>();

        for node in nodes {
            let notifier = state
                .drm_devices
                .get_mut(node)
                .ok_or_else(|| io::Error::other("DRM device disappeared during notifier setup"))?
                .take_notifier()
                .map_err(io::Error::other)?;

            let token = loop_handle
                .insert_source(notifier, move |event, _, state| match event {
                    DrmEvent::VBlank(crtc) => {
                        if let Some(surfaces) = state.drm_surfaces.get_mut(&node) {
                            match surfaces.on_vblank(node, crtc) {
                                Ok(Some(released)) => {
                                    if let Some(Err(error)) =
                                        surfaces.retire_released_frame(crtc, released)
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
                    DrmEvent::Error(error) => {
                        state.error =
                            Some(io::Error::other(format!("DRM device {node}: {error}")));
                    }
                })
                .map_err(io::Error::other)?;

            state.drm_notifiers.insert(node, token);
        }
    }

    // Register the Wayland display fd for client dispatch.
    let display_fd = state
        .display
        .backend()
        .poll_fd()
        .as_fd()
        .try_clone_to_owned()?;

    register_display_events(&loop_handle, display_fd)?;

    Ok((event_loop, state))
}
