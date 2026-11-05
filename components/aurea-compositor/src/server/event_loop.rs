
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland display dispatch and Calloop event loop.

use std::{
    io,
    os::fd::AsFd,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use smithay::reexports::{
    calloop::{
        EventLoop, Interest, Mode,
        generic::Generic,
    },
    wayland_server::Display,
};

#[cfg(feature = "drm")]
use crate::drm::{DrmSession, device::DrmDevice};

use super::{
    session,
    socket::register_wayland_socket,
    state::WaylandState,
};

#[cfg(feature = "drm")]
use super::session_events::register_session_events;

use crate::AureaCompositor;

/// Owns the Wayland display and its Calloop event loop.
pub struct WaylandServer {
    event_loop: EventLoop<WaylandState>,
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
    ///
    /// The GPU is discovered automatically; no `/dev/dri/cardN` path is needed.
    #[cfg(feature = "drm")]
    pub fn new_with_primary_drm(
        socket_name: &str,
        seat: &str,
    ) -> io::Result<Self> {
        let drm_session = session::create_session()?;
        let drm_device = session::open_primary_device(&drm_session, seat)?;
        Self::build(socket_name, Some(drm_session), Some(drm_device))
    }

    /// Creates a server with a native libseat session and an opened DRM device.
    #[cfg(feature = "drm")]
    pub fn new_with_drm_device(
        socket_name: &str,
        device_path: &str,
    ) -> io::Result<Self> {
        let drm_session = session::create_session()?;
        let drm_device = session::open_device(&drm_session, device_path)?;
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
        let mut event_loop =
            EventLoop::<WaylandState>::try_new().map_err(io::Error::other)?;

        let display = Display::<AureaCompositor>::new()?;
        let display_handle = display.handle();
        let compositor = AureaCompositor::new(&display_handle);

        #[cfg(feature = "drm")]
        let session_active = drm_session
            .as_ref()
            .map(|session| session.session().is_active())
            .unwrap_or(true);

        let mut state = WaylandState {
            display,
            compositor,
            clients: Vec::new(),
            error: None,
            #[cfg(feature = "drm")]
            drm_session,
            #[cfg(feature = "drm")]
            _drm_device: drm_device,
            #[cfg(feature = "drm")]
            session_active,
        };

        let loop_handle = event_loop.handle();

        // Register the listening socket and incoming client handling.
        register_wayland_socket(&loop_handle, socket_name)?;

        // Register native session activation and pause events.
        #[cfg(feature = "drm")]
        if let Some(session) = state.drm_session.as_mut() {
            register_session_events(&loop_handle, session)?;
        }

        // Monitor the Wayland display fd for incoming client requests.
        let display_fd = state
            .display
            .backend()
            .poll_fd()
            .as_fd()
            .try_clone_to_owned()?;

        let stop_handle = loop_handle.clone();

        loop_handle
            .insert_source(
                Generic::new(display_fd, Interest::READ, Mode::Level),
                move |_, _, state| {
                    if let Err(error) =
                        state.display.dispatch_clients(&mut state.compositor)
                    {
                        state.error = Some(io::Error::other(error));
                        stop_handle.stop();
                        return;
                    }

                    if let Err(error) = state.display.flush_clients() {
                        state.error = Some(io::Error::other(error));
                        stop_handle.stop();
                    }
                },
            )
            .map_err(io::Error::other)?;

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
