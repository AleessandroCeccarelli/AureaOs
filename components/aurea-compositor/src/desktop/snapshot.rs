// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Immutable desktop state captured for one compositor render frame.

use super::DesktopSurface;

/// Frame-safe desktop data passed from the manager to the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopSnapshot {
    surface: DesktopSurface,
}

impl DesktopSnapshot {
    /// Creates a snapshot containing the desktop surface.
    pub(crate) const fn new(surface: DesktopSurface) -> Self {
        Self { surface }
    }

    /// Returns the desktop surface included in this snapshot.
    pub const fn surface(self) -> DesktopSurface {
        self.surface
    }
}
