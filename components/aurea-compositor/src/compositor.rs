// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Core Wayland compositor state and module coordination.

use std::collections::{BTreeMap, HashMap, HashSet};

use aurea_desktop_api::{
    desktop::DesktopManager as DesktopItemsManager,
    desktop_interaction::DesktopInteractionSender,
    input::InputDeviceProvider,
};

use smithay::{
    backend::input::Keycode,
    input::{Seat, SeatHandler, SeatState, pointer::{CursorImageStatus, Focus}, touch::TouchHandle},
    output::Output,
    reexports::wayland_server::{
        DisplayHandle,
        protocol::{wl_shm::Format, wl_surface::WlSurface},
    },
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        tablet_manager::{TabletManagerState, TabletSeatHandler},
    },
};

use crate::{
    desktop::DesktopManager,
    frame::FrameScheduler,
    input::{
        cursor_animation::CursorLocateAnimation,
        interaction::{GestureProfile, InteractionEngine},
        pointer_settings::PointerSettings,
        pointer_shake::PointerShakeDetector,
        pointing::{
            AbsolutePointerOutputMap, PointerManager, PointingDeviceService, TabletCalibrationMap, TabletOutputMap,
            TabletToolRegistry,
            TouchscreenCalibration, TouchscreenCalibrationError, TouchscreenCalibrationMap,
            TouchscreenOutputMap, TouchscreenSlotMap,
        },
    },
    output::{OutputId, manager::OutputManager},
    window_manager::WindowManager,
};

mod desktop_click_tracker;
mod desktop_selection;
mod fractional_scale;
mod frame_scheduling;
mod keyboard;
mod keyboard_shortcuts;
mod output_configuration;
mod output_publication;
mod output_reconciliation;
mod pointer;
mod pointer_axis;
mod pointer_bounds;
mod pointer_button;
#[cfg(feature = "drm")]
mod pointer_gestures;
mod pointer_placement;
mod pointer_settings;
pub(crate) mod popup_placement;
mod rendering;
mod surface_output_reconciliation;
mod surface_output_tracking;
#[cfg(feature = "drm")]
mod tablet;
mod tablet_calibration;
mod tablet_output_mapping;
mod touchscreen;
pub(crate) mod window_placement;
mod workspace;

use desktop_click_tracker::DesktopClickTracker;
use surface_output_tracking::SurfaceOutputTracker;

pub use frame_scheduling::CompositorFrameError;
pub use output_configuration::CompositorOutputConfigurationError;
pub use output_reconciliation::OutputReconciliationError;
pub use rendering::CompositorRenderError;

/// Core state owned by the AureaOS compositor.
pub struct AureaCompositor {
    compositor_state: CompositorState,
    display_handle: DisplayHandle,
    shm_state: ShmState,
    xdg_shell_state: XdgShellState,
    _fractional_scale_manager_state:
        smithay::wayland::fractional_scale::FractionalScaleManagerState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    touch: Option<TouchHandle<Self>>,
    consumed_shortcut_keys: HashSet<Keycode>,
    window_manager: WindowManager,
    /// Tracks nested XDG popup trees and protocol popup grabs.
    popup_manager: smithay::desktop::PopupManager,
    pub(crate) popup_placement_registry: popup_placement::PopupPlacementRegistry,
    desktop_manager: DesktopManager,
    desktop_items: DesktopItemsManager,
    output_manager: OutputManager,
    frame_scheduler: FrameScheduler,
    wayland_outputs: BTreeMap<OutputId, Output>,
    surface_output_tracker: SurfaceOutputTracker,
    cursor_image: CursorImageStatus,
    pointer_settings: PointerSettings,
    _pointer_gestures_state: smithay::wayland::pointer_gestures::PointerGesturesState,
    _tablet_manager_state: TabletManagerState,
    pointer_manager: PointerManager,
    touchscreen_outputs: TouchscreenOutputMap,
    absolute_pointer_outputs: AbsolutePointerOutputMap,
    tablet_outputs: TabletOutputMap,
    /// Physical tablet ownership and active-source arbitration for each pen/tool.
    tablet_tools: TabletToolRegistry,
    tablet_calibrations: TabletCalibrationMap,
    touchscreen_calibrations: TouchscreenCalibrationMap,
    touchscreen_slots: TouchscreenSlotMap,
    interaction_engine: InteractionEngine,
    #[cfg(feature = "drm")]
    pointer_gesture_owner: pointer_gestures::PointerGestureOwner,
    desktop_interactions: DesktopInteractionSender,
    desktop_selection: aurea_desktop_api::desktop_interaction::DesktopSelectionController,
    desktop_click_tracker: DesktopClickTracker,
    /// Pointer buttons consumed by the compositor-owned desktop context menu.
    context_menu_consumed_buttons: HashSet<(aurea_desktop_api::input::InputDeviceId, u32)>,
    /// Independent shake-detection history for each physical pointer device.
    pointer_shakes: HashMap<aurea_desktop_api::input::InputDeviceId, PointerShakeDetector>,
    cursor_locate_animation: CursorLocateAnimation,
    /// Defers output-driven pointer recentering until any active client grab ends.
    pointer_recenter_pending: bool,
}

impl TabletSeatHandler for AureaCompositor {}

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
    pub fn new(display: &DisplayHandle, desktop_interactions: DesktopInteractionSender) -> Self {
        let compositor_state = CompositorState::new::<Self>(display);
        // Welcome and application clients use the two mandatory Wayland SHM formats.
        // Advertising no formats prevents clients from creating their first buffer.
        let shm_state = ShmState::new::<Self>(display, vec![Format::Argb8888, Format::Xrgb8888]);
        let xdg_shell_state = XdgShellState::new::<Self>(display);
        let fractional_scale_manager_state = fractional_scale::create_manager(display);
        let pointer_gestures_state =
            smithay::wayland::pointer_gestures::PointerGesturesState::new::<Self>(display);
        let tablet_manager_state = TabletManagerState::new::<Self>(display);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(display, "seat0");

        Self {
            compositor_state,
            display_handle: display.clone(),
            shm_state,
            xdg_shell_state,
            _fractional_scale_manager_state: fractional_scale_manager_state,
            seat_state,
            seat,
            touch: None,
            consumed_shortcut_keys: HashSet::new(),
            window_manager: WindowManager::new(),
            popup_manager: smithay::desktop::PopupManager::default(),
            popup_placement_registry: popup_placement::PopupPlacementRegistry::default(),
            desktop_manager: DesktopManager::default(),
            desktop_items: DesktopItemsManager::default(),
            output_manager: OutputManager::new(),
            frame_scheduler: FrameScheduler::new(),
            wayland_outputs: BTreeMap::new(),
            surface_output_tracker: SurfaceOutputTracker::new(),
            cursor_image: CursorImageStatus::default_named(),
            pointer_settings: PointerSettings::default(),
            _pointer_gestures_state: pointer_gestures_state,
            _tablet_manager_state: tablet_manager_state,
            pointer_manager: PointerManager::new(),
            touchscreen_outputs: TouchscreenOutputMap::new(),
            absolute_pointer_outputs: AbsolutePointerOutputMap::new(),
            tablet_outputs: TabletOutputMap::new(),
            tablet_tools: TabletToolRegistry::default(),
            tablet_calibrations: TabletCalibrationMap::default(),
            touchscreen_calibrations: TouchscreenCalibrationMap::default(),
            touchscreen_slots: TouchscreenSlotMap::new(),
            interaction_engine: InteractionEngine::new(),
            #[cfg(feature = "drm")]
            pointer_gesture_owner: pointer_gestures::PointerGestureOwner::default(),
            desktop_selection: desktop_interactions.selection_controller(),
            desktop_interactions,
            desktop_click_tracker: DesktopClickTracker::default(),
            context_menu_consumed_buttons: HashSet::new(),
            pointer_shakes: HashMap::new(),
            cursor_locate_animation: CursorLocateAnimation::default(),
            pointer_recenter_pending: false,
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

    /// Returns the compositor-owned desktop manager.
    pub fn desktop_manager(&self) -> &DesktopManager {
        &self.desktop_manager
    }

    /// Returns mutable access to desktop appearance policy.
    pub fn desktop_manager_mut(&mut self) -> &mut DesktopManager {
        &mut self.desktop_manager
    }

    /// Returns the shared desktop-entry model used by desktop presentation.
    pub fn desktop_items(&self) -> &DesktopItemsManager {
        &self.desktop_items
    }

    /// Returns mutable access to the shared desktop-entry model.
    pub fn desktop_items_mut(&mut self) -> &mut DesktopItemsManager {
        &mut self.desktop_items
    }

    /// Returns the window manager.
    pub fn window_manager(&mut self) -> &mut WindowManager {
        &mut self.window_manager
    }

    /// Returns mutable access to Smithay's nested popup registry.
    pub(crate) fn popup_manager(&mut self) -> &mut smithay::desktop::PopupManager {
        &mut self.popup_manager
    }

    /// Updates popup lifecycle bookkeeping after a surface commit.
    pub(crate) fn commit_popup_surface(&mut self, surface: &WlSurface) {
        self.popup_manager.commit(surface);
        self.popup_manager.cleanup();
    }

    /// Starts the protocol-mandated pointer and keyboard grabs for an XDG popup.
    pub(crate) fn start_popup_grab(
        &mut self,
        popup: smithay::desktop::PopupKind,
        seat_resource: &smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) -> Result<(), smithay::desktop::PopupGrabError> {
        use smithay::desktop::{
            PopupKeyboardGrab, PopupPointerGrab, find_popup_root_surface,
        };

        let Some(seat) = Seat::<Self>::from_resource(seat_resource) else {
            return Err(smithay::desktop::PopupGrabError::InvalidGrab);
        };
        let (Some(pointer), Some(keyboard)) = (seat.get_pointer(), seat.get_keyboard()) else {
            return Err(smithay::desktop::PopupGrabError::InvalidGrab);
        };

        let root = find_popup_root_surface(&popup)?;
        let grab = self
            .popup_manager
            .grab_popup::<Self>(root, popup, &seat, serial)?;

        // Both capabilities share the same PopupGrab serial. Smithay's default
        // handlers keep focus inside the popup tree, dismiss on outside clicks,
        // and restore the root surface when the grab ends.
        keyboard.set_grab(self, PopupKeyboardGrab::new(&grab), serial);
        pointer.set_grab(self, PopupPointerGrab::new(&grab), serial, Focus::Keep);
        Ok(())
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

    /// Returns the seat-wide registry of connected pointing devices.
    pub fn pointing_devices(&self) -> &PointingDeviceService {
        self.pointer_manager.devices()
    }

    /// Registers a discovered pointing device and reconciles its per-device preferences.
    pub(crate) fn register_pointing_device(
        &mut self,
        descriptor: aurea_desktop_api::input::InputDeviceDescriptor,
    ) -> bool {
        self.pointer_manager.connect(descriptor)
    }

    /// Clears protocol state and unregisters a disconnected pointing device.
    pub(crate) fn disconnect_pointing_device(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> bool {
        self.clear_pointing_device_state(device);
        self.pointer_manager.disconnect(device)
    }

    /// Assigns an absolute pointer-capable device to one enabled display.
    pub fn assign_absolute_pointer_output(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        output: OutputId,
    ) -> bool {
        use aurea_desktop_api::input::{InputDeviceProvider, StandardInputCapability};
        if !self.pointer_manager.devices().device(device).is_some_and(|descriptor| {
            descriptor.capabilities.supports(StandardInputCapability::PointerMotion)
        }) || !self.output_manager.registry().get(output).is_some_and(|display| display.is_enabled()) {
            return false;
        }
        self.absolute_pointer_outputs.assign(device, output);
        true
    }

    /// Removes an explicit absolute-pointer mapping; single-output fallback applies.
    pub fn unassign_absolute_pointer_output(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> bool {
        self.absolute_pointer_outputs.unassign(device)
    }

    /// Resolves the output without guessing on multi-monitor desktops.
    pub fn absolute_pointer_output(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<OutputId> {
        let enabled = self.output_manager.registry().iter()
            .filter(|output| output.is_enabled())
            .map(|output| output.id());
        self.absolute_pointer_outputs.resolve(device, enabled)
    }

    /// Initializes the Wayland touch capability when a touchscreen is present.
    pub fn sync_touchscreen_capability(&mut self) {
        let connected = self
            .pointer_manager.devices()
            .has_kind(aurea_desktop_api::input::InputDeviceKind::Touchscreen);

        if connected && self.touch.is_none() {
            self.touch = Some(self.seat.add_touch());
        } else if !connected && self.touch.is_some() {
            self.seat.remove_touch();
            self.touch = None;
        }
    }

    /// Assigns a touchscreen to an enabled physical display.
    ///
    /// Returns false when the target output is missing or disabled.
    pub fn assign_touchscreen_output(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        output: OutputId,
    ) -> bool {
        if !self
            .pointer_manager.devices()
            .device(device)
            .is_some_and(|descriptor| {
                descriptor.kind == aurea_desktop_api::input::InputDeviceKind::Touchscreen
            })
            || !self
                .output_manager
                .registry()
                .get(output)
                .is_some_and(|display| display.is_enabled())
        {
            return false;
        }

        self.touchscreen_outputs.assign(device, output);
        true
    }

    /// Sets an affine calibration profile for a connected touchscreen.
    ///
    /// Profiles are session-scoped and keyed by the compositor's device ID.
    pub fn set_touchscreen_calibration(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        coefficients: [f64; 6],
    ) -> Result<(), TouchscreenCalibrationError> {
        if !self
            .pointer_manager.devices()
            .device(device)
            .is_some_and(|descriptor| {
                descriptor.kind == aurea_desktop_api::input::InputDeviceKind::Touchscreen
            })
        {
            return Err(TouchscreenCalibrationError::UnknownDevice);
        }

        let calibration = TouchscreenCalibration::new(coefficients)?;
        self.touchscreen_calibrations.set(device, calibration);
        Ok(())
    }

    /// Resets one touchscreen to the identity calibration.
    pub fn reset_touchscreen_calibration(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> bool {
        self.touchscreen_calibrations.remove_device(device)
    }

    /// Removes an explicit touchscreen-to-display assignment.
    pub fn unassign_touchscreen_output(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> bool {
        self.touchscreen_outputs.unassign(device)
    }

    /// Resolves a touchscreen against the currently enabled displays.
    ///
    /// Automatic routing is intentionally limited to single-monitor systems.
    pub fn touchscreen_output(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<OutputId> {
        let enabled = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .map(|output| output.id());

        self.touchscreen_outputs.resolve(device, enabled)
    }

    /// Returns the shared desktop interaction engine.
    pub fn interaction_engine(&self) -> &InteractionEngine {
        &self.interaction_engine
    }

    /// Replaces the default desktop gesture bindings.
    pub fn set_gesture_profile(&mut self, profile: GestureProfile) {
        self.interaction_engine.set_profile(profile);
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
        self.touchscreen_outputs.remove_output(output_id);
        self.absolute_pointer_outputs.remove_output(output_id);
        self.tablet_outputs.remove_output(output_id);
    }

    /// Reconciles mapped surfaces against enabled outputs and updates client scale.
    pub(crate) fn reconcile_surface_outputs(&mut self) {
        surface_output_reconciliation::reconcile(self);
    }

    /// Clears gesture and touchscreen routing state for a disconnected device.
    pub fn remove_pointing_device_state(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) {
        let time = smithay::utils::Clock::<smithay::utils::Monotonic>::new()
            .now()
            .as_millis() as u32;
        self.handle_touch_cancel(device, time);
        self.release_pointer_buttons_for_device(device, time);
        self.interaction_engine.remove_device(device);
        self.touchscreen_outputs.remove_device(device);
        self.absolute_pointer_outputs.remove_device(device);
        self.tablet_outputs.remove_device(device);
        self.tablet_calibrations.remove_device(device);
        self.touchscreen_calibrations.remove_device(device);
        self.touchscreen_slots.remove_device(device);
        self.pointer_manager.remove_device_state(device);
    }

    /// Releases tracker entries for destroyed Wayland surfaces.
    pub fn prune_surface_output_membership(&mut self) {
        self.surface_output_tracker.prune_dead_surfaces();
    }
}

#[cfg(test)]
#[path = "../tests/unit/shm_protocol.rs"]
mod shm_protocol_tests;
