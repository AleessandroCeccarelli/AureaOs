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

mod fractional_scale;
mod frame_scheduling;
mod keyboard;
mod keyboard_shortcuts;
mod output_publication;
mod output_reconciliation;
mod pointer;
mod pointer_axis;
mod pointer_bounds;
mod pointer_button;
mod pointer_placement;
mod rendering;
mod surface_output_reconciliation;
mod surface_output_tracking;
mod workspace;

use surface_output_tracking::SurfaceOutputTracker;

pub use frame_scheduling::CompositorFrameError;
pub use output_reconciliation::OutputReconciliationError;
pub use rendering::CompositorRenderError;

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    shm_state: ShmState,
    xdg_shell_state: XdgShellState,
    _fractional_scale_manager_state:
        smithay::wayland::fractional_scale::FractionalScaleManagerState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    consumed_shortcut_keys: HashSet<Keycode>,
    window_manager: WindowManager,
    output_manager: OutputManager,
    frame_scheduler: FrameScheduler,
    wayland_outputs: BTreeMap<OutputId, Output>,
    surface_output_tracker: SurfaceOutputTracker,
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
        let fractional_scale_manager_state = fractional_scale::create_manager(display);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(display, "seat0");

        Self {
            compositor_state,
            shm_state,
            xdg_shell_state,
            _fractional_scale_manager_state: fractional_scale_manager_state,
            seat_state,
            seat,
            consumed_shortcut_keys: HashSet::new(),
            window_manager: WindowManager::new(),
            output_manager: OutputManager::new(),
            frame_scheduler: FrameScheduler::new(),
            wayland_outputs: BTreeMap::new(),
            surface_output_tracker: SurfaceOutputTracker::new(),
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

    /// Updates a surface's Wayland output membership after its geometry changes.
    ///
    /// The caller supplies only enabled outputs whose logical bounds intersect
    /// the surface's committed geometry.
    pub fn update_surface_output_membership(
        &mut self,
        surface: &WlSurface,
        intersecting_outputs: impl IntoIterator<Item = (OutputId, Output)>,
    ) {
        self.surface_output_tracker
            .reconcile(surface, intersecting_outputs);
    }

    /// Sends leave events and forgets a surface that has been unmapped.
    pub fn forget_surface_output_membership(&mut self, surface: &WlSurface) {
        self.surface_output_tracker.remove_surface(surface);
    }

    /// Sends leave events for every surface in an unmapped surface tree.
    pub(crate) fn forget_surface_tree_output_membership(&mut self, root: &WlSurface) {
        for surface in crate::render::surface_tree::collect_surface_tree_surfaces(root) {
            self.surface_output_tracker.remove_surface(&surface);
        }
    }

    /// Sends leave events for an output that has been disabled or removed.
    pub fn forget_output_membership(&mut self, output_id: OutputId) {
        self.surface_output_tracker.remove_output(output_id);
    }

    /// Reconciles mapped surfaces against enabled outputs and updates client scale.
    pub(crate) fn reconcile_surface_outputs(&mut self) {
        surface_output_reconciliation::reconcile(self);
    }

    /// Releases tracker entries for destroyed Wayland surfaces.
    pub fn prune_surface_output_membership(&mut self) {
        self.surface_output_tracker.prune_dead_surfaces();
    }
}

#[cfg(test)]
#[path = "../tests/unit/shm_protocol.rs"]
mod shm_protocol_tests;
