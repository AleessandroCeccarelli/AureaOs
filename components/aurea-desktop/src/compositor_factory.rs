// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Desktop composition of the native compositor and first visible client.

use std::{io, os::unix::process::ExitStatusExt};

use aurea_compositor::service::{DEFAULT_WAYLAND_DISPLAY_NAME, WaylandCompositorFactory};
use aurea_desktop_api::{
    CompositorFactory, CompositorService, ShutdownSignal,
    desktop_interaction::DesktopInteractionSender, display::DisplayColorController,
    input::{DesktopPointerBehaviorCommand, PointerSettingsCommand, TouchpadSettingsCommand}, keyboard::KeyboardLayoutId,
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
        interactions: DesktopInteractionSender,
        touchpad_settings: std::sync::mpsc::Receiver<TouchpadSettingsCommand>,
        pointer_settings: std::sync::mpsc::Receiver<PointerSettingsCommand>,
        desktop_pointer_behavior: std::sync::mpsc::Receiver<DesktopPointerBehaviorCommand>,
    ) -> io::Result<Box<dyn CompositorService>> {
        // Read the initial Trash state in the desktop composition root.
        // The compositor receives only the semantic value, never filesystem access.
        let compositor_factory = match aurea_desktop::trash::initial_visual_state() {
            Ok(state) => self.compositor.clone().with_trash_state(state),
            Err(error) => {
                eprintln!("Aurea Desktop: initial Trash state unavailable: {error}");
                self.compositor.clone()
            }
        };

        // Creating the native service also creates and binds the Wayland socket.
        let compositor = compositor_factory
            .create(display_color, interactions, touchpad_settings, pointer_settings, desktop_pointer_behavior)?;
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
            Ok(Some(status))
                if !status.success()
                    && !matches!(status.signal(), Some(libc::SIGTERM | libc::SIGINT)) =>
            {
                eprintln!("Aurea Desktop: Welcome client exited unexpectedly with status {status}");
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Aurea Desktop: cannot stop Welcome client: {error}");
            }
        }
    }
}
