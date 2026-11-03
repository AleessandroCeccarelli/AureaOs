// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state.

use smithay::{
    reexports::wayland_server::DisplayHandle,
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
    },
};

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
}

impl AureaCompositor {
    /// Creates the compositor protocol states and registers their globals.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        let xdg_shell_state = XdgShellState::new::<Self>(display);

        Self {
            compositor_state,
            xdg_shell_state,
        }
    }

    /// Returns the Smithay compositor protocol state.
    pub fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    /// Returns the XDG shell protocol state.
    pub fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
}
