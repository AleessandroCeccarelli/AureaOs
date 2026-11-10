
// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state and module coordination.

use std::collections::{BTreeMap, HashSet};

use smithay::{
    backend::input::Keycode,
    input::{Seat, SeatHandler, SeatState},
    output::Output,
    reexports::wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    wayland::{compositor::CompositorState, shell::xdg::XdgShellState},
};

use crate::{
    frame::FrameScheduler,
    output::{OutputId, manager::OutputManager},
    window_manager::WindowManager,
};

mod frame_scheduling;
mod keyboard;
mod output_publication;
mod output_reconciliation;
mod rendering;
mod workspace;

pub use frame_scheduling::CompositorFrameError;
pub use output_reconciliation::OutputReconciliationError;
pub use rendering::CompositorRenderError;

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    suppressed_shortcut_releases: HashSet<Keycode>,
    window_manager: WindowManager,
    output_manager: OutputManager,
    frame_scheduler: FrameScheduler,
    wayland_outputs: BTreeMap<OutputId, Output>,
}

impl SeatHandler for AureaCompositor {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

impl AureaCompositor {
    /// Creates the compositor protocol states and managers.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        let xdg_shell_state = XdgShellState::new::<Self>(display);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(display, "seat0");

        Self {
            compositor_state,
            xdg_shell_state,
            seat_state,
            seat,
            suppressed_shortcut_releases: HashSet::new(),
            window_manager: WindowManager::new(),
            output_manager: OutputManager::new(),
            frame_scheduler: FrameScheduler::new(),
            wayland_outputs: BTreeMap::new(),
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

    /// Returns the window manager.
    pub fn window_manager(&mut self) -> &mut WindowManager {
        &mut self.window_manager
    }

    /// Returns the output manager.
    pub fn output_manager(&mut self) -> &mut OutputManager {
        &mut self.output_manager
    }
}

#[cfg(test)]
#[path = "../tests/unit/compositor_output_reconciliation.rs"]
mod output_reconciliation_tests;
