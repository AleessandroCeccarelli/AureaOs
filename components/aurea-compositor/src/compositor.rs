// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state and module coordination.

use std::collections::{BTreeMap, HashSet};

use smithay::{
    backend::input::Keycode,
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    output::Output,
    reexports::wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    wayland::{compositor::CompositorState, shell::xdg::XdgShellState, shm::ShmState},
};

use crate::{
    frame::FrameScheduler,
    input::{
        cursor_animation::CursorLocateAnimation, pointer_settings::PointerSettings,
        pointer_shake::PointerShakeDetector,
    },
    output::{OutputId, manager::OutputManager},
    window_manager::WindowManager,
};

mod frame_scheduling;
mod keyboard;
mod output_publication;
mod output_reconciliation;
mod pointer;
mod pointer_axis;
mod pointer_bounds;
mod pointer_button;
mod rendering;
mod workspace;

pub use frame_scheduling::CompositorFrameError;
pub use output_reconciliation::OutputReconciliationError;
pub use rendering::CompositorRenderError;

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    shm_state: ShmState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    suppressed_shortcut_releases: HashSet<Keycode>,
    window_manager: WindowManager,
    output_manager: OutputManager,
    frame_scheduler: FrameScheduler,
    wayland_outputs: BTreeMap<OutputId, Output>,
    cursor_image: CursorImageStatus,
    pointer_settings: PointerSettings,
    pointer_shake: PointerShakeDetector,
    cursor_locate_animation: CursorLocateAnimation,
}

impl SeatHandler for AureaCompositor {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_image = image;
        self.request_all_output_frames();
    }
}

impl AureaCompositor {
    /// Creates the compositor protocol states and managers.
    pub fn new(display: &DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        let shm_state = ShmState::new::<Self>(display, Vec::new());
        let xdg_shell_state = XdgShellState::new::<Self>(display);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(display, "seat0");

        Self {
            compositor_state,
            shm_state,
            xdg_shell_state,
            seat_state,
            seat,
            suppressed_shortcut_releases: HashSet::new(),
            window_manager: WindowManager::new(),
            output_manager: OutputManager::new(),
            frame_scheduler: FrameScheduler::new(),
            wayland_outputs: BTreeMap::new(),
            cursor_image: CursorImageStatus::default_named(),
            pointer_settings: PointerSettings::default(),
            pointer_shake: PointerShakeDetector::default(),
            cursor_locate_animation: CursorLocateAnimation::default(),
        }
    }

    /// Returns the Smithay shared-memory protocol state.
    pub(crate) fn wayland_shm_state(&self) -> &ShmState {
        &self.shm_state
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

    /// Returns the cursor image currently requested by the focused Wayland client.
    pub fn cursor_image_status(&self) -> &CursorImageStatus {
        &self.cursor_image
    }

    /// Returns the current cursor locate animation scale for a frame timestamp.
    pub fn cursor_locate_scale(&self) -> f64 {
        self.cursor_locate_animation
            .scale_at(std::time::Instant::now())
    }

    /// Reports whether the locate effect still needs scheduled frames.
    pub fn cursor_animation_active(&self) -> bool {
        self.cursor_locate_animation
            .is_active(std::time::Instant::now())
    }

    /// Returns the active pointer preferences.
    pub fn pointer_settings(&self) -> PointerSettings {
        self.pointer_settings
    }

    /// Updates validated pointer preferences.
    pub fn set_pointer_settings(&mut self, settings: PointerSettings) {
        self.pointer_settings = settings;
    }

    /// Returns the output manager.
    pub fn output_manager(&mut self) -> &mut OutputManager {
        &mut self.output_manager
    }
}

#[cfg(test)]
#[path = "../tests/unit/shm_protocol.rs"]
mod shm_protocol_tests;
