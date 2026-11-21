// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Compositor
// Copyright (C) 2026 Alessandro Ceccarelli

//! Adapter between the shared desktop API and the native Wayland server.

use std::io;

use aurea_desktop_api::{
    CompositorFactory, CompositorService, ShutdownSignal, display::DisplayColorController,
    keyboard::KeyboardLayoutId, desktop_interaction::DesktopInteractionSender,
};

use crate::WaylandServer;

/// Default Wayland display socket shared with native desktop clients.
pub const DEFAULT_WAYLAND_DISPLAY_NAME: &str = "wayland-0";

/// Default libseat seat used by the native desktop session.
pub const DEFAULT_WAYLAND_SEAT_NAME: &str = "seat0";

/// Configuration and factory for the native Wayland compositor.
#[derive(Clone, Debug)]
pub struct WaylandCompositorFactory {
    socket_name: String,
    seat: String,
    keyboard_layout: Option<KeyboardLayoutId>,
}

impl WaylandCompositorFactory {
    /// Creates a factory with the requested Wayland socket and seat.
    pub fn new(socket_name: impl Into<String>, seat: impl Into<String>) -> Self {
        Self {
            socket_name: socket_name.into(),
            seat: seat.into(),
            keyboard_layout: None,
        }
    }

    /// Uses a persisted user-selected keyboard layout for the session.
    pub fn with_keyboard_layout(mut self, layout: KeyboardLayoutId) -> Self {
        self.keyboard_layout = Some(layout);
        self
    }
}

impl Default for WaylandCompositorFactory {
    fn default() -> Self {
        Self::new(DEFAULT_WAYLAND_DISPLAY_NAME, DEFAULT_WAYLAND_SEAT_NAME)
    }
}

impl CompositorFactory for WaylandCompositorFactory {
    fn create(
        &self,
        display_color: DisplayColorController,
        interactions: DesktopInteractionSender,
    ) -> io::Result<Box<dyn CompositorService>> {
        let mut server =
            WaylandServer::new_with_primary_drm(&self.socket_name, &self.seat, display_color, interactions)?;
        if let Some(layout) = self.keyboard_layout.as_ref() {
            if let Err(error) = server.set_keyboard_layout(layout) {
                eprintln!(
                    "AureaOS: saved keyboard layout could not be applied; keeping the default XKB layout: {error}"
                );
            }
        }

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
