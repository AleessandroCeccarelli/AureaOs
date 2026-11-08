// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Virtual workspace operations and workspace-related window management.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use super::AureaCompositor;

impl AureaCompositor {
    /// Returns the zero-based index of the active virtual workspace.
    pub fn active_workspace(&self) -> usize {
        self.window_manager.active_workspace()
    }

    /// Switches the visible scene to a workspace by its zero-based index.
    ///
    /// Returns `false` when the index is outside the four-workspace set.
    /// Keyboard focus is moved to a mapped surface on the new workspace.
    pub fn switch_workspace(&mut self, workspace: usize) -> bool {
        if !self.window_manager.switch_workspace(workspace) {
            return false;
        }

        self.focus_surface(self.window_manager.active_workspace_surface());

        true
    }

    /// Moves a managed toplevel surface to a workspace by its zero-based index.
    ///
    /// Returns `false` when the surface is unmanaged or the workspace index
    /// is invalid. Focus is refreshed after a successful move.
    pub fn move_toplevel_to_workspace(&mut self, surface: &WlSurface, workspace: usize) -> bool {
        if !self
            .window_manager
            .move_toplevel_to_workspace(surface, workspace)
        {
            return false;
        }

        self.focus_surface(self.window_manager.active_workspace_surface());

        true
    }
}
