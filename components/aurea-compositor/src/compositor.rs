
 // SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state.

use smithay::wayland::compositor::CompositorState;
use smithay::reexports::wayland_server::DisplayHandle;

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
}

impl AureaCompositor {
    /// Creates the compositor protocol state and registers its globals.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);

        Self { compositor_state }
    }

    /// Returns the Smithay compositor protocol state.
    pub fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
}
