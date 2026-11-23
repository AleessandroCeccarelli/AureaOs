// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Immutable description of the desktop surface for a render frame.

/// Opaque desktop surface data consumed by the rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopSurface {
    background_color: [f32; 4],
}

impl DesktopSurface {
    /// Creates a desktop surface from an already-resolved opaque color.
    pub(crate) const fn new(background_color: [f32; 4]) -> Self {
        Self { background_color }
    }

    /// Returns the opaque RGBA color used to initialize the desktop surface.
    pub const fn background_color(self) -> [f32; 4] {
        self.background_color
    }
}
