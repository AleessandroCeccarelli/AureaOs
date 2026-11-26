// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Public access to global and per-device pointer preferences.

use aurea_desktop_api::input::PointerSettings;

use super::AureaCompositor;

impl AureaCompositor {
    /// Sets the desktop double-click interval in milliseconds.
    /// Values are clamped to 150–1000 ms; changing it cancels a pending first click.
    pub fn set_desktop_double_click_interval_ms(&mut self, interval_ms: u64) {
        self.desktop_click_tracker
            .set_interval(std::time::Duration::from_millis(interval_ms));
    }

    /// Returns the active desktop double-click interval in milliseconds.
    pub fn desktop_double_click_interval_ms(&self) -> u64 {
        self.desktop_click_tracker.interval().as_millis() as u64
    }

    /// Returns the default pointer preferences for devices without an override.
    pub fn pointer_settings(&self) -> PointerSettings {
        self.pointer_settings
    }

    /// Returns preferences for one physical pointer.
    pub fn pointer_settings_for(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> PointerSettings {
        self.pointer_manager
            .pointer_settings(device, self.pointer_settings)
    }

    /// Updates the default pointer preferences.
    pub fn set_pointer_settings(&mut self, settings: PointerSettings) {
        self.pointer_settings = settings;
    }

    /// Sets a per-device pointer profile for a connected mouse. Touchpads use TouchpadSettings.
    pub fn set_pointer_device_settings(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        settings: PointerSettings,
    ) -> bool {
        self.pointer_manager.set_pointer_settings(device, settings)
    }

    /// Returns touchpad preferences for one device, falling back to defaults.
    pub fn touchpad_settings_for(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> aurea_desktop_api::input::TouchpadSettings {
        self.pointer_manager.touchpad_settings(device)
    }

    /// Returns the latest hardware configuration report for one touchpad.
    pub fn touchpad_configuration_report(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<aurea_desktop_api::input::TouchpadApplyReport> {
        self.pointer_manager.touchpad_report(device)
    }

    /// Records a touchpad configuration report for trusted settings clients.
    pub fn record_touchpad_configuration(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
        report: aurea_desktop_api::input::TouchpadApplyReport,
    ) -> bool {
        self.pointer_manager.set_touchpad_report(device, report)
    }

    /// Sets touchpad preferences for one connected touchpad.
    pub fn set_touchpad_device_settings(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        settings: aurea_desktop_api::input::TouchpadSettings,
    ) -> bool {
        self.pointer_manager.set_touchpad_settings(device, settings)
    }
}
