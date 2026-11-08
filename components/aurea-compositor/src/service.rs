
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Compositor
// Copyright (C) 2026 Alessandro Ceccarelli

//! Adapter between the shared desktop API and the native Wayland server.

use std::io;

use aurea_desktop_api::{
    CompositorFactory,
    CompositorService,
    ShutdownSignal,
};

use crate::WaylandServer;

/// Configuration and factory for the native Wayland compositor.
#[derive(Clone, Debug)]
pub struct WaylandCompositorFactory {
    socket_name: String,
    seat: String,
}

impl WaylandCompositorFactory {
    /// Creates a factory with the requested Wayland socket and seat.
    pub fn new(
        socket_name: impl Into<String>,
        seat: impl Into<String>,
    ) -> Self {
        Self {
            socket_name: socket_name.into(),
            seat: seat.into(),
        }
    }
}

impl Default for WaylandCompositorFactory {
    fn default() -> Self {
        Self::new("wayland-0", "seat0")
    }
}

impl CompositorFactory for WaylandCompositorFactory {
    fn create(&self) -> io::Result<Box<dyn CompositorService>> {
        let server = WaylandServer::new_with_primary_drm(
            &self.socket_name,
            &self.seat,
        )?;

        Ok(Box::new(WaylandCompositorService { server }))
    }
}

/// Native Wayland server implementing the shared compositor contract.
struct WaylandCompositorService {
    server: WaylandServer,
}

impl CompositorService for WaylandCompositorService {
    fn run(&mut self, shutdown: &ShutdownSignal) -> io::Result<()> {
        let shutdown_flag = shutdown.atomic_flag();
        self.server.run(&shutdown_flag)
    }
}
