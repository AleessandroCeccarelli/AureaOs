// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Top-level window management.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface, utils::Point,
    wayland::shell::xdg::ToplevelSurface,
};

/// Manages the lifecycle of XDG toplevel windows.
#[derive(Default)]
pub struct WindowManager {
    toplevels: Vec<ManagedToplevel>,
    active_workspace: usize,
}

struct ManagedToplevel {
    surface: ToplevelSurface,
    mapped: bool,
    location: Point<i32, smithay::utils::Logical>,
    workspace: usize,
}

impl WindowManager {
    /// Number of virtual workspaces provided by the initial desktop model.
    pub const WORKSPACE_COUNT: usize = 4;

    /// Creates an empty window manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new toplevel and sends its initial configure.
    pub fn add_toplevel(&mut self, surface: ToplevelSurface) {
        self.remove_dead_toplevels();
        surface.send_configure();

        self.toplevels.push(ManagedToplevel {
            surface,
            mapped: false,
            location: Point::from((0, 0)),
            workspace: self.active_workspace,
        });
    }

    /// Returns the zero-based index of the active workspace.
    pub fn active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// Switches to a workspace by its zero-based index.
    ///
    /// Returns `false` when the index is outside the configured workspace set.
    pub fn switch_workspace(&mut self, workspace: usize) -> bool {
        if workspace >= Self::WORKSPACE_COUNT {
            return false;
        }

        self.active_workspace = workspace;
        true
    }

    /// Moves a managed toplevel to a workspace by its zero-based index.
    ///
    /// Returns `false` when either the surface is unmanaged or the workspace
    /// index is outside the configured workspace set.
    pub fn move_toplevel_to_workspace(&mut self, surface: &WlSurface, workspace: usize) -> bool {
        if workspace >= Self::WORKSPACE_COUNT {
            return false;
        }

        self.remove_dead_toplevels();
        let Some(toplevel) = self
            .toplevels
            .iter_mut()
            .find(|toplevel| toplevel.surface.wl_surface() == surface)
        else {
            return false;
        };

        toplevel.workspace = workspace;
        true
    }

    /// Sets the logical position of a managed toplevel.
    ///
    /// Returns `false` when the surface is not a managed toplevel.
    pub fn set_toplevel_location(&mut self, surface: &WlSurface, location: (i32, i32)) -> bool {
        self.remove_dead_toplevels();

        let Some(toplevel) = self
            .toplevels
            .iter_mut()
            .find(|toplevel| toplevel.surface.wl_surface() == surface)
        else {
            return false;
        };

        toplevel.location = Point::from(location);
        true
    }

    /// Updates mapping state when a toplevel's buffer changes.
    pub fn update_surface_buffer(
        &mut self,
        surface: &WlSurface,
        mapped: Option<bool>,
    ) -> Option<bool> {
        self.remove_dead_toplevels();

        if let Some(mapped) = mapped
            && let Some(toplevel) = self
                .toplevels
                .iter_mut()
                .find(|toplevel| toplevel.surface.wl_surface() == surface)
        {
            toplevel.mapped = mapped;
            return Some(mapped);
        }

        None
    }

    /// Removes handles for windows that have been destroyed.
    pub fn remove_dead_toplevels(&mut self) {
        self.toplevels.retain(|toplevel| toplevel.surface.alive());
    }

    /// Returns the number of currently alive toplevels.
    pub fn window_count(&self) -> usize {
        self.toplevels
            .iter()
            .filter(|toplevel| toplevel.surface.alive())
            .count()
    }

    /// Returns the number of currently mapped toplevels.
    pub fn mapped_window_count(&self) -> usize {
        self.toplevels
            .iter()
            .filter(|toplevel| toplevel.surface.alive() && toplevel.mapped)
            .count()
    }

    /// Returns the alive, mapped toplevels for rendering.
    pub fn mapped_toplevels(&self) -> impl Iterator<Item = &ToplevelSurface> {
        self.toplevels
            .iter()
            .filter(|toplevel| toplevel.surface.alive() && toplevel.mapped)
            .map(|toplevel| &toplevel.surface)
    }

    /// Returns mapped toplevels and their logical positions in registration order.
    pub(crate) fn mapped_toplevels_with_location(
        &self,
    ) -> impl Iterator<Item = (&ToplevelSurface, Point<i32, smithay::utils::Logical>)> {
        self.toplevels
            .iter()
            .filter(|toplevel| {
                toplevel.surface.alive()
                    && toplevel.mapped
                    && toplevel.workspace == self.active_workspace
            })
            .map(|toplevel| (&toplevel.surface, toplevel.location))
    }

    /// Returns one mapped surface on the active workspace for keyboard focus.
    pub(crate) fn active_workspace_surface(&self) -> Option<WlSurface> {
        self.toplevels
            .iter()
            .find(|toplevel| {
                toplevel.surface.alive()
                    && toplevel.mapped
                    && toplevel.workspace == self.active_workspace
            })
            .map(|toplevel| toplevel.surface.wl_surface().clone())
    }
}
