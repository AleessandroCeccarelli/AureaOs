// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Immutable desktop state captured for one compositor render frame.

use super::{DesktopSurface, WallpaperSnapshot};

/// Frame-safe desktop data passed from the manager to the renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopSnapshot {
    surface: DesktopSurface,
    wallpaper: Option<WallpaperSnapshot>,
}

impl DesktopSnapshot {
    /// Creates a snapshot containing the desktop surface and optional wallpaper.
    pub(crate) fn new(
        surface: DesktopSurface,
        wallpaper: Option<WallpaperSnapshot>,
    ) -> Self {
        Self { surface, wallpaper }
    }

    /// Returns the desktop surface included in this snapshot.
    pub const fn surface(&self) -> DesktopSurface {
        self.surface
    }

    /// Returns the selected wallpaper included in this snapshot, if valid.
    pub fn wallpaper(&self) -> Option<&WallpaperSnapshot> {
        self.wallpaper
    }
}
