// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop composition of the native compositor and first visible client.

use std::process::{Child, Command, Stdio};
use std::{io, process};

use aurea_compositor::service::WaylandCompositorFactory;
use aurea_desktop_api::{
    CompositorFactory, CompositorService, ShutdownSignal, display::DisplayColorController,
};

const WAYLAND_DISPLAY_NAME: &str = "wayland-0";
const WAYLAND_SEAT_NAME: &str = "seat0";
const WELCOME_CLIENT_ARGUMENT: &str = "--welcome-client";

/// Creates the compositor and starts the first native desktop client.
pub struct DesktopCompositorFactory {
    compositor: WaylandCompositorFactory,
}

impl Default for DesktopCompositorFactory {
    fn default() -> Self {
        Self {
            compositor: WaylandCompositorFactory::new(
                WAYLAND_DISPLAY_NAME,
                WAYLAND_SEAT_NAME,
            ),
        }
    }
}

impl CompositorFactory for DesktopCompositorFactory {
    fn create(
        &self,
        display_color: DisplayColorController,
    ) -> io::Result<Box<dyn CompositorService>> {
        // Creating the native service also creates and binds the Wayland socket.
        let compositor = self.compositor.create(display_color)?;
        let executable = std::env::current_exe()?;
        let welcome_client = Command::new(executable)
            .arg(WELCOME_CLIENT_ARGUMENT)
            .env("WAYLAND_DISPLAY", WAYLAND_DISPLAY_NAME)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(Box::new(DesktopCompositorService {
            compositor,
            welcome_client: Some(welcome_client),
        }))
    }
}

/// Keeps the welcome client alive only for the compositor session.
struct DesktopCompositorService {
    compositor: Box<dyn CompositorService>,
    welcome_client: Option<Child>,
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
        let Some(mut child) = self.welcome_client.take() else {
            return;
        };

        match child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
            }
            Err(error) => {
                eprintln!("Aurea Desktop: cannot inspect Welcome client: {error}");
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

impl Drop for DesktopCompositorService {
    fn drop(&mut self) {
        self.stop_welcome_client();
    }
}
