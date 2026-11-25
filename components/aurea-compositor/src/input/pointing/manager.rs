// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates pointing-device inventory, preferences, and physical button state.

use std::collections::{HashMap, HashSet};

use aurea_desktop_api::input::{
    InputDeviceDescriptor, InputDeviceId, PointerSettings, TouchpadApplyReport, TouchpadSettings,
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
    pub(crate) fn connect(&mut self, descriptor: InputDeviceDescriptor) -> bool {
        let id = descriptor.id;
        let kind = descriptor.kind;
        if !self.devices.connect(descriptor) {
            return false;
        }
        self.settings.reconcile_kind(id, kind);
        true
    }

    /// Removes a device from the inventory.
    /// Removes a device after the compositor has synthesized protocol releases.
    pub(crate) fn disconnect(&mut self, device: InputDeviceId) -> bool {
        self.settings.remove(device);
        self.pressed_buttons.remove(&device);
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
        let mut buttons: Vec<_> = self
            .pressed_buttons
            .remove(&device)
            .map(|buttons| buttons.into_iter().collect())
            .unwrap_or_default();
        buttons.sort_unstable();
        buttons
    }

    /// Clears preferences and button state associated with a disconnected device.
    pub fn remove_device_state(&mut self, device: InputDeviceId) {
        self.settings.remove(device);
        self.pressed_buttons.remove(&device);
    }
}



#[cfg(test)]
mod tests {
    use super::PointerManager;
    use aurea_desktop_api::input::{
        InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceId, InputDeviceKind,
        PointerAccelerationProfile, PointerSettings, TouchpadSettings,
    };

    fn descriptor(id: u64, kind: InputDeviceKind) -> InputDeviceDescriptor {
        InputDeviceDescriptor {
            id: InputDeviceId(id),
            kind,
            name: format!("Pointing device {id}"),
            vendor_id: None,
            product_id: None,
            capabilities: InputDeviceCapabilities::default(),
        }
    }

    #[test]
    fn pressed_buttons_are_isolated_and_released_in_stable_order() {
        let mut manager = PointerManager::new();
        let first = InputDeviceId(1);
        let second = InputDeviceId(2);

        manager.update_button(first, 0x112, true);
        manager.update_button(first, 0x110, true);
        manager.update_button(first, 0x111, true);
        manager.update_button(second, 0x110, true);

        assert_eq!(
            manager.take_pressed_buttons(first),
            vec![0x110, 0x111, 0x112]
        );
        assert_eq!(manager.take_pressed_buttons(first), Vec::<u32>::new());
        assert_eq!(manager.take_pressed_buttons(second), vec![0x110]);
    }

    #[test]
    fn releasing_one_button_preserves_other_pressed_buttons() {
        let mut manager = PointerManager::new();
        let device = InputDeviceId(7);

        manager.update_button(device, 0x110, true);
        manager.update_button(device, 0x111, true);
        manager.update_button(device, 0x110, false);

        assert_eq!(manager.take_pressed_buttons(device), vec![0x111]);
    }

    #[test]
    fn device_preferences_are_scoped_and_removed_with_device_state() {
        let mut manager = PointerManager::new();
        let mouse = InputDeviceId(10);
        let touchpad = InputDeviceId(11);
        assert!(manager.connect(descriptor(10, InputDeviceKind::Mouse)));
        assert!(manager.connect(descriptor(11, InputDeviceKind::Touchpad)));

        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.6, false)
            .expect("valid pointer profile");
        let touchpad_profile = TouchpadSettings::new(false, true, false, true, false);
        assert!(manager.set_pointer_settings(mouse, pointer));
        assert!(manager.set_touchpad_settings(touchpad, touchpad_profile));
        assert_eq!(
            manager.pointer_settings(mouse, PointerSettings::default()),
            pointer
        );
        assert_eq!(manager.touchpad_settings(touchpad), touchpad_profile);
        assert_eq!(
            manager.pointer_settings(touchpad, PointerSettings::default()),
            PointerSettings::default()
        );

        manager.update_button(mouse, 0x110, true);
        manager.remove_device_state(mouse);

        assert_eq!(
            manager.pointer_settings(mouse, PointerSettings::default()),
            PointerSettings::default()
        );
        assert!(manager.take_pressed_buttons(mouse).is_empty());
        assert_eq!(manager.touchpad_settings(touchpad), touchpad_profile);
    }

    #[test]
    fn reclassification_reconciles_per_device_preferences() {
        let mut manager = PointerManager::new();
        let device = InputDeviceId(13);
        assert!(manager.connect(descriptor(13, InputDeviceKind::Touchpad)));

        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.55, false)
            .expect("valid pointer profile");
        let touchpad = TouchpadSettings::new(false, true, false, true, false);
        assert!(manager.set_pointer_settings(device, pointer));
        assert!(manager.set_touchpad_settings(device, touchpad));

        assert!(manager.connect(descriptor(13, InputDeviceKind::Mouse)));
        assert_eq!(manager.pointer_settings(device, PointerSettings::default()), pointer);
        assert_eq!(manager.touchpad_settings(device), TouchpadSettings::default());

        assert!(manager.connect(descriptor(13, InputDeviceKind::Tablet)));
        assert_eq!(
            manager.pointer_settings(device, PointerSettings::default()),
            PointerSettings::default()
        );
    }

    #[test]
    fn touchpad_preferences_reject_mouse_devices() {
        let mut manager = PointerManager::new();
        let mouse = InputDeviceId(12);
        assert!(manager.connect(descriptor(12, InputDeviceKind::Mouse)));
        assert!(!manager.set_touchpad_settings(mouse, TouchpadSettings::default()));
    }
}
