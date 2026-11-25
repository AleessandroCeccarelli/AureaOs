// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates pointing-device inventory, preferences, and physical button state.

use std::collections::{HashMap, HashSet};

use aurea_desktop_api::input::{
    InputDeviceId, InputDeviceProvider, PointerSettings, TouchpadSettings,
    TouchpadApplyReport, InputDeviceDescriptor,
};

use super::{PointingDeviceService, PointingDeviceSettings};

/// Owns the policy state shared by mouse, touchpad, touchscreen, and tablet paths.
///
/// Protocol delivery remains in the compositor's focused event modules. This
/// manager owns device inventory, per-device preferences, and physical button
/// bookkeeping so those concerns do not become fields scattered across the
/// compositor root.
#[derive(Default)]
pub struct PointerManager {
    devices: PointingDeviceService,
    settings: PointingDeviceSettings,
    pressed_buttons: HashMap<InputDeviceId, HashSet<u32>>,
}

impl PointerManager {
    /// Creates an empty pointer manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the connected pointing-device inventory.
    pub fn devices(&self) -> &PointingDeviceService {
        &self.devices
    }

    /// Registers or refreshes a discovered pointing device.
    pub fn connect(&self, descriptor: InputDeviceDescriptor) -> bool {
        self.devices.connect(descriptor)
    }

    /// Removes a device from the inventory.
    pub fn disconnect(&self, device: InputDeviceId) -> bool {
        self.devices.disconnect(device)
    }

    /// Returns the stored per-device preference collection.
    pub fn settings(&self) -> &PointingDeviceSettings {
        &self.settings
    }

    /// Returns mutable access to per-device preferences.
    pub fn settings_mut(&mut self) -> &mut PointingDeviceSettings {
        &mut self.settings
    }

    /// Returns the effective pointer preferences for one device.
    pub fn pointer_settings(
        &self,
        device: InputDeviceId,
        fallback: PointerSettings,
    ) -> PointerSettings {
        self.settings.pointer(device, fallback)
    }

    /// Sets a pointer profile only when the device is a registered mouse or touchpad.
    pub fn set_pointer_settings(
        &mut self,
        device: InputDeviceId,
        settings: PointerSettings,
    ) -> bool {
        self.settings.set_pointer(&self.devices, device, settings)
    }

    /// Returns touchpad preferences, falling back to the shared defaults.
    pub fn touchpad_settings(&self, device: InputDeviceId) -> TouchpadSettings {
        self.settings.touchpad(device)
    }

    /// Sets touchpad preferences only for a registered touchpad.
    pub fn set_touchpad_settings(
        &mut self,
        device: InputDeviceId,
        settings: TouchpadSettings,
    ) -> bool {
        self.settings.set_touchpad(&self.devices, device, settings)
    }

    /// Stores the result of applying touchpad settings to hardware.
    pub fn set_touchpad_report(&self, device: InputDeviceId, report: TouchpadApplyReport) -> bool {
        self.devices.set_touchpad_report(device, report)
    }

    /// Returns the latest touchpad hardware-application report.
    pub fn touchpad_report(&self, device: InputDeviceId) -> Option<TouchpadApplyReport> {
        self.devices.touchpad_report(device)
    }

    /// Tracks a physical button transition for its originating device.
    pub fn update_button(&mut self, device: InputDeviceId, button: u32, pressed: bool) {
        if pressed {
            self.pressed_buttons
                .entry(device)
                .or_default()
                .insert(button);
        } else if let Some(buttons) = self.pressed_buttons.get_mut(&device) {
            buttons.remove(&button);
            if buttons.is_empty() {
                self.pressed_buttons.remove(&device);
            }
        }
    }

    /// Takes all still-pressed buttons when a physical device is disconnected.
    pub fn take_pressed_buttons(&mut self, device: InputDeviceId) -> Vec<u32> {
        self.pressed_buttons
            .remove(&device)
            .map(|buttons| buttons.into_iter().collect())
            .unwrap_or_default()
    }

    /// Clears preferences and button state associated with a disconnected device.
    pub fn remove_device_state(&mut self, device: InputDeviceId) {
        self.settings.remove(device);
        self.pressed_buttons.remove(&device);
    }
}

