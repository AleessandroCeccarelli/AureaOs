// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Wayland fractional-scale protocol registration.

use smithay::{
    delegate_fractional_scale,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{
        compositor,
        fractional_scale::{self, FractionalScaleHandler, FractionalScaleManagerState},
    },
};

use crate::AureaCompositor;

impl FractionalScaleHandler for AureaCompositor {
    /// Initializes a newly created fractional-scale object with the
    /// conservative 1.0 fallback. Output-aware preferred-scale updates are
    /// applied by the surface/output reconciliation coordinator.
    fn new_fractional_scale(&mut self, surface: WlSurface) {
        compositor::with_states(&surface, |states| {
            fractional_scale::with_fractional_scale(states, |scale| {
                scale.set_preferred_scale(1.0);
            });
        });
    }
}

delegate_fractional_scale!(AureaCompositor);

/// Creates the fractional-scale manager global for a compositor display.
pub(super) fn create_manager(
    display: &smithay::reexports::wayland_server::DisplayHandle,
) -> FractionalScaleManagerState {
    FractionalScaleManagerState::new::<AureaCompositor>(display)
}

/// Updates the preferred fractional scale of one Wayland surface.
pub(super) fn set_surface_preferred_scale(surface: &WlSurface, scale: f64) {
    compositor::with_states(surface, |states| {
        fractional_scale::with_fractional_scale(states, |fractional| {
            fractional.set_preferred_scale(scale);
        });
    });
}
