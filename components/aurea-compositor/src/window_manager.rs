// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli
//! Top-level window management.
use smithay::wayland::shell::xdg::ToplevelSurface;
/// Manages the lifecycle of XDG toplevel windows.
#[derive(Default)]
pub struct WindowManager {
    toplevels: Vec<ToplevelSurface>,
}
impl WindowManager {
    /// Creates an empty window manager.
    pub fn new() -> Self {
        Self::default()
    }
    /// Registers a new toplevel and sends its initial configure.
    pub fn add_toplevel(&mut self, surface: ToplevelSurface) {
        self.remove_dead_toplevels();
        surface.send_configure();
        self.toplevels.push(surface);
    }
    /// Removes handles for windows that have been destroyed.
    pub fn remove_dead_toplevels(&mut self) {
        self.toplevels.retain(ToplevelSurface::alive);
    }
    /// Returns the number of currently alive toplevels.
    pub fn window_count(&self) -> usize {
        self.toplevels
            .iter()
            .filter(|surface| surface.alive())
            .count()
    }
}
