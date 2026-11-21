// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Public access to global and per-device pointer preferences.

use aurea_desktop_api::input::PointerSettings;

use super::AureaCompositor;

impl AureaCompositor {
    /// Returns the default pointer preferences for devices without an override.
    pub fn pointer_settings(&self) -> PointerSettings {
        self.pointer_settings
    }

    /// Returns preferences for one physical pointer.
    pub fn pointer_settings_for(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> PointerSettings {
        self.pointing_settings.pointer(device, self.pointer_settings)
    }

    /// Updates the default pointer preferences.
    pub fn set_pointer_settings(&mut self, settings: PointerSettings) {
        self.pointer_settings = settings;
    }

    /// Sets a per-device pointer profile for a connected mouse or touchpad.
    pub fn set_pointer_device_settings(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        settings: PointerSettings,
    ) -> bool {
        self.pointing_settings.set_pointer(&self.pointing_devices, device, settings)
    }

    /// Returns touchpad preferences for one device, falling back to defaults.
    pub fn touchpad_settings_for(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> aurea_desktop_api::input::TouchpadSettings {
        self.pointing_settings.touchpad(device)
    }

    /// Returns the latest hardware configuration report for one touchpad.
    pub fn touchpad_configuration_report(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
    ) -> Option<aurea_desktop_api::input::TouchpadApplyReport> {
        self.pointing_devices.touchpad_report(device)
    }

    /// Records a touchpad configuration report for trusted settings clients.
    pub fn record_touchpad_configuration(
        &self,
        device: aurea_desktop_api::input::InputDeviceId,
        report: aurea_desktop_api::input::TouchpadApplyReport,
    ) -> bool {
        self.pointing_devices.set_touchpad_report(device, report)
    }

    /// Sets touchpad preferences for one connected touchpad.
    pub fn set_touchpad_device_settings(
        &mut self,
        device: aurea_desktop_api::input::InputDeviceId,
        settings: aurea_desktop_api::input::TouchpadSettings,
    ) -> bool {
        self.pointing_settings.set_touchpad(&self.pointing_devices, device, settings)
    }
}
