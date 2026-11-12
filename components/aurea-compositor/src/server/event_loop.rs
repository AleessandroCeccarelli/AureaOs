// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland server runtime and Calloop event loop execution.

use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use smithay::reexports::calloop::EventLoop;

#[cfg(feature = "drm")]
use crate::drm::{DrmSession, device::DrmDevice};

use super::state::WaylandState;

#[cfg(feature = "drm")]
use super::session;

/// Owns the Wayland display state and its Calloop event loop.
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
        let parts = super::builder::build(socket_name, None, None);

        #[cfg(not(feature = "drm"))]
        let parts = super::builder::build(socket_name);

        let (event_loop, state) = parts?;

        Ok(Self { event_loop, state })
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

    #[cfg(feature = "drm")]
    fn build(
        socket_name: &str,
        drm_session: Option<DrmSession>,
        drm_device: Option<DrmDevice>,
    ) -> io::Result<Self> {
        let (event_loop, state) = super::builder::build(socket_name, drm_session, drm_device)?;

        Ok(Self { event_loop, state })
    }

    /// Runs the server until shutdown is requested.
    pub fn run(&mut self, shutdown: &AtomicBool) -> io::Result<()> {
        #[cfg(feature = "drm")]
        self.state.compositor.request_all_output_frames();

        while !shutdown.load(Ordering::Acquire) {
            self.event_loop
                .dispatch(Duration::from_millis(100), &mut self.state)
                .map_err(io::Error::other)?;

            #[cfg(feature = "drm")]
            super::rendering::render_pending_outputs(&mut self.state);

            if let Some(error) = self.state.error.take() {
                return Err(error);
            }
        }

        Ok(())
    }
}
