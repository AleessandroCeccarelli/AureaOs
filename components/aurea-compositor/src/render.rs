// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Collects mapped surfaces into a renderable scene.

use smithay::wayland::shell::xdg::ToplevelSurface;

use crate::window_manager::WindowManager;

/// A snapshot of the mapped toplevels for one rendering pass.
pub struct RenderScene<'a> {
    toplevels: Vec<&'a ToplevelSurface>,
}

impl<'a> RenderScene<'a> {
    /// Builds a scene from the window manager's currently mapped toplevels.
    pub fn from_window_manager(window_manager: &'a WindowManager) -> Self {
        Self {
            toplevels: window_manager.mapped_toplevels().collect(),
        }
    }

    /// Returns the toplevel surfaces in this scene.
    pub fn toplevels(&self) -> impl Iterator<Item = &'a ToplevelSurface> + '_ {
        self.toplevels.iter().copied()
    }

    /// Returns the number of surfaces collected for this scene.
    pub fn len(&self) -> usize {
        self.toplevels.len()
    }

    /// Returns whether this scene contains no mapped surfaces.
    pub fn is_empty(&self) -> bool {
        self.toplevels.is_empty()
    }
}
