
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state.

use smithay::{
    output::Output,
    reexports::wayland_server::DisplayHandle,
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
    },
};

use crate::{
    output::{manager::OutputManager, wayland::publish_outputs},
    render::RenderScene,
    window_manager::WindowManager,
};

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    window_manager: WindowManager,
    output_manager: OutputManager,
    wayland_outputs: Vec<Output>,
}

impl AureaCompositor {
    /// Creates the compositor protocol states and managers.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        let xdg_shell_state = XdgShellState::new::<Self>(display);

        Self {
            compositor_state,
            xdg_shell_state,
            window_manager: WindowManager::new(),
            output_manager: OutputManager::new(),
            wayland_outputs: Vec::new(),
        }
    }

    /// Publishes registered outputs as Wayland globals.
    pub fn publish_wayland_outputs(&mut self, display: &DisplayHandle) {
        self.wayland_outputs = publish_outputs(display, &self.output_manager);
    }

    /// Returns the Smithay compositor protocol state.
    pub fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    /// Returns the XDG shell protocol state.
    pub fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    /// Returns the window manager.
    pub fn window_manager(&mut self) -> &mut WindowManager {
        &mut self.window_manager
    }

    /// Returns the output manager.
    pub fn output_manager(&mut self) -> &mut OutputManager {
        &mut self.output_manager
    }

    /// Builds a snapshot of the currently mapped surfaces.
    pub fn render_scene(&self) -> RenderScene<'_> {
        RenderScene::from_window_manager(&self.window_manager)
    }
}
