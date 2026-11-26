// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server construction and backend initialization.

use aurea_desktop_api::{
    desktop_interaction::DesktopInteractionSender, display::DisplayColorController,
    input::{DesktopPointerBehaviorCommand, PointerSettingsCommand, TouchpadSettingsCommand},
};
use std::sync::mpsc::Receiver;

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
use crate::drm::{DrmSession, device::DrmDevice};

#[cfg(feature = "drm")]
use crate::input::pointing::AppliedTouchpadSettings;

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
    display_color: DisplayColorController,
    desktop_interactions: DesktopInteractionSender,
    touchpad_settings_receiver: Receiver<TouchpadSettingsCommand>,
    pointer_settings_receiver: Receiver<PointerSettingsCommand>,
    desktop_pointer_behavior_receiver: Receiver<DesktopPointerBehaviorCommand>,
) -> io::Result<(EventLoop<'static, WaylandState>, WaylandState)> {
    let event_loop = EventLoop::<WaylandState>::try_new().map_err(io::Error::other)?;

    let display = Display::<AureaCompositor>::new().map_err(io::Error::other)?;
    let display_handle = display.handle();
    let compositor = AureaCompositor::new(&display_handle, desktop_interactions);

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

    #[cfg(not(feature = "drm"))]
    let _ = (touchpad_settings_receiver, pointer_settings_receiver, desktop_pointer_behavior_receiver);

    let mut state = WaylandState {
        display,
        compositor,
        clients: Vec::new(),
        disconnected_clients: Arc::new(Mutex::new(Vec::new())),
        error: None,
        display_color,
        #[cfg(feature = "drm")]
        touchpad_settings_receiver,
        #[cfg(feature = "drm")]
        pointer_settings_receiver,
        #[cfg(feature = "drm")]
        desktop_pointer_behavior_receiver,
        #[cfg(feature = "drm")]
        desktop_pointer_behavior_store: crate::input::pointing::DesktopPointerBehaviorStore::default(),
        #[cfg(feature = "drm")]
        applied_touchpad_settings: AppliedTouchpadSettings::default(),
        #[cfg(feature = "drm")]
        touchpad_devices: std::collections::HashMap::new(),
        #[cfg(feature = "drm")]
        mouse_devices: std::collections::HashMap::new(),

        #[cfg(feature = "drm")]
        drm_session,

        #[cfg(feature = "drm")]
        drm_devices,

        #[cfg(feature = "drm")]
        drm_display_identities: crate::drm::identity_registry::DrmDisplayIdentityRegistry::new(),

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
    state.compositor.initialize_pointer()?;

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

                let device = match DrmDevice::open(session, &path) {
                    Ok(device) => device,
                    Err(error) => {
                        eprintln!(
                            "AureaOS: unable to open discovered DRM device {node} at {}: {error}",
                            path.display()
                        );
                        continue;
                    }
                };

                match device.node() {
                    Ok(opened_node) if opened_node == node => {}
                    Ok(opened_node) => {
                        eprintln!(
                            "AureaOS: refusing DRM device at {}: Udev identified {node}, but the opened descriptor identifies {opened_node}",
                            path.display()
                        );
                        continue;
                    }
                    Err(error) => {
                        eprintln!(
                            "AureaOS: unable to verify DRM device identity for {} ({node}): {error}",
                            path.display()
                        );
                        continue;
                    }
                }

                if let Err(error) = state.drm_devices.register(device) {
                    eprintln!("AureaOS: unable to register DRM device {node}: {error}");
                }
            }
        }
    }

    // Initialize GBM independently for every opened DRM device.
    #[cfg(feature = "drm")]
    super::gbm_lifecycle::initialize_gbm_devices(&mut state);

    // Initialize a DRM-matched Vulkan renderer, with a CPU fallback.
    #[cfg(feature = "drm")]
    super::vulkan_lifecycle::initialize_vulkan_backends(&mut state);

    // Publish only outputs with retained native KMS presentation surfaces.
    #[cfg(feature = "drm")]
    super::output_lifecycle::initialize_outputs(&mut state)?;

    // Publish registered outputs as Wayland globals.
    state
        .compositor
        .publish_wayland_outputs(&display_handle)
        .map_err(io::Error::other)?;

    // A DRM compositor with an active display must start with a valid,
    // visible pointer location. Do not silently boot at (0, 0), which may be
    // outside a negative-origin or rearranged monitor layout.
    let active_outputs = state.compositor.output_manager().active_output_count();
    let pointer_centered = state.compositor.center_pointer_on_primary_output();
    if active_outputs > 0 && !pointer_centered {
        return Err(io::Error::other(
            "AureaOS: active display outputs are available, but the Wayland pointer could not be placed on the primary output",
        ));
    }
    // Start the native cursor's asynchronous first-load window only now:
    // DRM/GPU discovery above may have consumed the constructor's initial
    // warm-up period before the first output frame can be rendered.
    if pointer_centered {
        state.compositor.begin_cursor_asset_warmup();
    }

    eprintln!(
        "AureaOS pointer: startup capability ready; active_outputs={} centered={}",
        active_outputs, pointer_centered
    );

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
                            match surfaces.on_cpu_vblank(node, crtc) {
                                Ok(Some(released)) => {
                                    if let Some(Err(error)) =
                                        surfaces.retire_released_cpu_frame(crtc, released)
                                    {
                                        eprintln!(
                                            "AureaOS: failed to retire released CPU scanout frame on {node}: {error}"
                                        );
                                    }
                                }
                                Ok(None) => match surfaces.on_vblank(node, crtc) {
                                    Ok(Some(released)) => {
                                        if let Some(Err(error)) =
                                            surfaces.retire_released_frame(crtc, released)
                                        {
                                            eprintln!(
                                                "AureaOS: failed to reacquire released Vulkan scanout frame on {node}: {error}"
                                            );
                                        }
                                    }
                                    Ok(None) => {}
                                    Err(error) => eprintln!(
                                        "AureaOS: DRM presentation completion failed for {node}: {error}"
                                    ),
                                },
                                Err(error) => eprintln!(
                                    "AureaOS: CPU KMS presentation completion failed for {node}: {error}"
                                ),
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

    #[cfg(feature = "drm")]
    state.compositor.set_desktop_double_click_interval_ms(
        state.desktop_pointer_behavior_store.settings().double_click_interval_ms(),
    );

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
