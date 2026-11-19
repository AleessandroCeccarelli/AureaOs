// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop composition of the native compositor and first visible client.

use std::io;

use aurea_compositor::service::{DEFAULT_WAYLAND_DISPLAY_NAME, WaylandCompositorFactory};
use aurea_desktop_api::{
    CompositorFactory, CompositorService, ShutdownSignal, display::DisplayColorController,
    keyboard::KeyboardLayoutId,
};

use crate::welcome_process::WelcomeClientProcess;

/// Creates the compositor and starts the first native desktop client.
pub struct DesktopCompositorFactory {
    compositor: WaylandCompositorFactory,
}

impl Default for DesktopCompositorFactory {
    fn default() -> Self {
        Self::with_keyboard_layout(None)
    }
}

impl DesktopCompositorFactory {
    /// Creates the desktop compositor with an optional persisted keyboard layout.
    pub fn with_keyboard_layout(layout: Option<KeyboardLayoutId>) -> Self {
        let mut compositor = WaylandCompositorFactory::default();
        if let Some(layout) = layout {
            compositor = compositor.with_keyboard_layout(layout);
        }
        Self { compositor }
    }
}

impl CompositorFactory for DesktopCompositorFactory {
    fn create(
        &self,
        display_color: DisplayColorController,
    ) -> io::Result<Box<dyn CompositorService>> {
        // Creating the native service also creates and binds the Wayland socket.
        let compositor = self.compositor.create(display_color)?;
        let welcome_client = WelcomeClientProcess::spawn(DEFAULT_WAYLAND_DISPLAY_NAME)?;

        Ok(Box::new(DesktopCompositorService {
            compositor,
            welcome_client,
        }))
    }
}

/// Keeps the Welcome client scoped to the compositor session.
struct DesktopCompositorService {
    compositor: Box<dyn CompositorService>,
    welcome_client: WelcomeClientProcess,
}

impl CompositorService for DesktopCompositorService {
    fn run(&mut self, shutdown: &ShutdownSignal) -> io::Result<()> {
        let result = self.compositor.run(shutdown);
        self.stop_welcome_client();
        result
    }
}

impl DesktopCompositorService {
    /// Stops and reaps the client before the compositor session is released.
    fn stop_welcome_client(&mut self) {
        match self.welcome_client.stop() {
            Ok(Some(status)) if !status.success() => {
                eprintln!("Aurea Desktop: Welcome client exited with status {status}");
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Aurea Desktop: cannot stop Welcome client: {error}");
            }
        }
    }
}
