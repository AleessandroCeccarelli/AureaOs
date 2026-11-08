// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server construction and Calloop event loop coordination.

use std::{
    io,
    os::fd::AsFd,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use smithay::reexports::wayland_server::Display;

#[cfg(feature = "drm")]
use smithay::backend::drm::DrmEvent;

#[cfg(feature = "drm")]
use crate::{
    drm::{DrmSession, device::DrmDevice},
    renderer::RenderPreference,
    vulkan::VulkanBackend,
};

use super::{
    display_events::register_display_events, socket::register_wayland_socket, state::WaylandState,
};

#[cfg(feature = "drm")]
use super::session;

#[cfg(feature = "drm")]
use super::session_events::register_session_events;

use smithay::reexports::calloop::EventLoop;

use crate::AureaCompositor;

/// Owns the Wayland display and its Calloop event loop.
pub struct WaylandServer {
    event_loop: EventLoop<'static, WaylandState>,
    state: WaylandState,
}

impl WaylandServer {
    /// Creates a server bound to the requested Wayland socket name.
    ///
    /// This constructor does not require a native DRM session.
    pub fn new(socket_name: &str) -> io::Result<Self> {
        #[cfg(feature = "drm")]
        {
            Self::build(socket_name, None, None)
        }

        #[cfg(not(feature = "drm"))]
        {
            Self::build(socket_name)
        }
    }

    /// Creates a server with a native libseat session.
    #[cfg(feature = "drm")]
    pub fn new_with_drm(socket_name: &str) -> io::Result<Self> {
        let drm_session = session::create_session()?;
        Self::build(socket_name, Some(drm_session), None)
    }

    /// Creates a server with a libseat session and the primary GPU for a seat.
    #[cfg(feature = "drm")]
    pub fn new_with_primary_drm(socket_name: &str, seat: &str) -> io::Result<Self> {
        let mut drm_session = session::create_session()?;
        let drm_device = session::open_primary_device(&mut drm_session, seat)?;

        Self::build(socket_name, Some(drm_session), Some(drm_device))
    }

    /// Creates a server with a native libseat session and an opened DRM device.
    #[cfg(feature = "drm")]
    pub fn new_with_drm_device(socket_name: &str, device_path: &str) -> io::Result<Self> {
        let mut drm_session = session::create_session()?;
        let drm_device = session::open_device(&mut drm_session, device_path)?;

        Self::build(socket_name, Some(drm_session), Some(drm_device))
    }

    /// Returns whether the native session is currently active.
    #[cfg(feature = "drm")]
    pub fn session_active(&self) -> bool {
        self.state.session_active
    }

    fn build(
        socket_name: &str,
        #[cfg(feature = "drm")] drm_session: Option<DrmSession>,
        #[cfg(feature = "drm")] drm_device: Option<DrmDevice>,
    ) -> io::Result<Self> {
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
                        eprintln!(
                            "AureaOS: Vulkan backend unavailable for DRM device {node}: {error}"
                        );
                    }
                }
            }
        }

        // Discover connected connectors on the currently initialized DRM devices.
        #[cfg(feature = "drm")]
        for (_, device) in state.drm_devices.iter() {
            crate::drm::outputs::initialize_drm_outputs(device, state.compositor.output_manager())
                .map_err(io::Error::other)?;
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
                    .ok_or_else(|| {
                        io::Error::other("DRM device disappeared during notifier setup")
                    })?
                    .take_notifier()
                    .map_err(io::Error::other)?;

                let token = loop_handle
                    .insert_source(notifier, move |event, _, state| match event {
                        DrmEvent::VBlank(_crtc) => {}
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

        Ok(Self { event_loop, state })
    }

    /// Runs the server until shutdown is requested.
    pub fn run(&mut self, shutdown: &AtomicBool) -> io::Result<()> {
        while !shutdown.load(Ordering::Acquire) {
            self.event_loop
                .dispatch(Duration::from_millis(100), &mut self.state)
                .map_err(io::Error::other)?;

            if let Some(error) = self.state.error.take() {
                return Err(error);
            }
        }

        Ok(())
    }
}
