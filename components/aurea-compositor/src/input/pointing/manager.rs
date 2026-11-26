// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Coordinates pointing-device inventory, preferences, and physical button state.

use std::collections::{HashMap, HashSet};

use aurea_desktop_api::input::{
    InputDeviceDescriptor, InputDeviceId, InputDeviceProvider, PointerSettings,
    TouchpadApplyReport, TouchpadSettings,
};

use super::{PointingDeviceService, PointingDeviceSettings};
use super::{pointer_store::PointerProfileStore, touchpad_store::TouchpadProfileStore};

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
    persistent_touchpads: TouchpadProfileStore,
    persistent_pointers: PointerProfileStore,
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
        let persistent_id = descriptor.persistent_id.clone();
        if !self.devices.connect(descriptor) {
            return false;
        }
        self.settings.reconcile_kind(id, kind);
        if let Some(identity) = persistent_id {
            if kind == aurea_desktop_api::input::InputDeviceKind::Mouse {
                if let Some(profile) = self.persistent_pointers.get(&identity) {
                    self.settings.set_pointer(&self.devices, id, profile);
                }
            }
            if kind == aurea_desktop_api::input::InputDeviceKind::Touchpad {
                if let Some(profile) = self.persistent_touchpads.get(&identity) {
                    self.settings.set_touchpad(&self.devices, id, profile);
                }
            }
        }
        true
    }

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

    /// Sets a pointer profile only when the device is a registered mouse.
    pub fn set_pointer_settings(
        &mut self,
        device: InputDeviceId,
        settings: PointerSettings,
    ) -> bool {
        let Some(descriptor) = self.devices.device(device) else {
            return false;
        };
        if descriptor.kind != aurea_desktop_api::input::InputDeviceKind::Mouse {
            return false;
        }

        // Persist first: a failed write must not leave a runtime-only profile
        // that the caller mistakes for a durable preference.
        if let Some(identity) = descriptor.persistent_id {
            if let Err(error) = self.persistent_pointers.set(identity, settings) {
                tracing::warn!(device_id = device.0, %error, "Could not persist pointer preferences; keeping previous settings");
                return false;
            }
        }
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
        let Some(descriptor) = self.devices.device(device) else {
            return false;
        };
        if descriptor.kind != aurea_desktop_api::input::InputDeviceKind::Touchpad {
            return false;
        }

        // Persist first so disk errors cannot silently create a temporary-only
        // profile while reporting the request as accepted.
        if let Some(identity) = descriptor.persistent_id {
            if let Err(error) = self.persistent_touchpads.set(identity, settings) {
                tracing::warn!(device_id = device.0, %error, "Could not persist touchpad preferences; keeping previous settings");
                return false;
            }
        }
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
    /// Updates one physical button and reports whether the shared seat state changed.
    ///
    /// If two devices hold the same button, only the first press and final
    /// release are forwarded to the single Wayland seat pointer.
    pub fn update_button(&mut self, device: InputDeviceId, button: u32, pressed: bool) -> bool {
        let was_pressed = self
            .pressed_buttons
            .values()
            .any(|buttons| buttons.contains(&button));

        if pressed {
            let inserted = self
                .pressed_buttons
                .entry(device)
                .or_default()
                .insert(button);
            inserted && !was_pressed
        } else {
            let removed = if let Some(buttons) = self.pressed_buttons.get_mut(&device) {
                let removed = buttons.remove(&button);
                if buttons.is_empty() {
                    self.pressed_buttons.remove(&device);
                }
                removed
            } else {
                false
            };
            removed
                && !self
                    .pressed_buttons
                    .values()
                    .any(|buttons| buttons.contains(&button))
        }
    }

    /// Removes a device's held buttons and returns only seat buttons no other
    /// connected device still holds.
    pub fn take_buttons_released_by_device(&mut self, device: InputDeviceId) -> Vec<u32> {
        let removed = self.pressed_buttons.remove(&device).unwrap_or_default();
        let mut released = removed
            .into_iter()
            .filter(|button| {
                !self
                    .pressed_buttons
                    .values()
                    .any(|buttons| buttons.contains(button))
            })
            .collect::<Vec<_>>();
        released.sort_unstable();
        released
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
        InputDeviceProvider, PointerAccelerationProfile, PointerSettings, TouchpadSettings,
    };

    fn descriptor(id: u64, kind: InputDeviceKind) -> InputDeviceDescriptor {
        InputDeviceDescriptor {
            id: InputDeviceId(id),
            kind,
            name: format!("Pointing device {id}"),
            vendor_id: None,
            product_id: None,
            persistent_id: None,
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
            manager.take_buttons_released_by_device(first),
            vec![0x111, 0x112]
        );
        assert_eq!(
            manager.take_buttons_released_by_device(first),
            Vec::<u32>::new()
        );
        assert_eq!(manager.take_buttons_released_by_device(second), vec![0x110]);
    }

    #[test]
    fn duplicate_physical_button_events_do_not_duplicate_seat_transitions() {
        let mut manager = PointerManager::new();
        let mouse = InputDeviceId(30);

        assert!(manager.update_button(mouse, 0x110, true));
        assert!(!manager.update_button(mouse, 0x110, true));
        assert!(manager.update_button(mouse, 0x110, false));
        assert!(!manager.update_button(mouse, 0x110, false));
    }

    #[test]
    fn overlapping_button_holds_are_aggregated_across_devices() {
        let mut manager = PointerManager::new();
        let first = InputDeviceId(31);
        let second = InputDeviceId(32);

        assert!(manager.update_button(first, 0x110, true));
        assert!(!manager.update_button(second, 0x110, true));
        assert!(!manager.update_button(first, 0x110, false));
        assert!(manager.update_button(second, 0x110, false));
    }

    #[test]
    fn disconnecting_one_holder_does_not_release_button_still_held_by_another() {
        let mut manager = PointerManager::new();
        let first = InputDeviceId(33);
        let second = InputDeviceId(34);
        manager.update_button(first, 0x110, true);
        manager.update_button(second, 0x110, true);

        assert!(manager.take_buttons_released_by_device(first).is_empty());
        assert!(manager.update_button(second, 0x110, false));
    }

    #[test]
    fn releasing_one_button_preserves_other_pressed_buttons() {
        let mut manager = PointerManager::new();
        let device = InputDeviceId(7);

        manager.update_button(device, 0x110, true);
        manager.update_button(device, 0x111, true);
        manager.update_button(device, 0x110, false);

        assert_eq!(manager.take_buttons_released_by_device(device), vec![0x111]);
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
        assert!(!manager.set_pointer_settings(touchpad, pointer));
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
        assert!(manager.take_buttons_released_by_device(mouse).is_empty());
        assert_eq!(manager.touchpad_settings(touchpad), touchpad_profile);
    }

    #[test]
    fn reclassification_reconciles_per_device_preferences() {
        let mut manager = PointerManager::new();
        let device = InputDeviceId(13);
        assert!(manager.connect(descriptor(13, InputDeviceKind::Mouse)));

        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.55, false)
            .expect("valid pointer profile");
        assert!(manager.set_pointer_settings(device, pointer));

        assert!(manager.connect(descriptor(13, InputDeviceKind::Tablet)));
        assert_eq!(
            manager.pointer_settings(device, PointerSettings::default()),
            PointerSettings::default()
        );
    }

    #[test]
    fn disconnect_removes_inventory_and_all_per_device_state() {
        let mut manager = PointerManager::new();
        let device = InputDeviceId(14);
        assert!(manager.connect(descriptor(14, InputDeviceKind::Mouse)));
        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.45, false)
            .expect("valid pointer profile");
        assert!(manager.set_pointer_settings(device, pointer));
        manager.update_button(device, 0x110, true);

        // The compositor drains this list into Wayland releases before disconnect.
        assert_eq!(manager.take_buttons_released_by_device(device), vec![0x110]);
        assert!(manager.disconnect(device));

        assert!(manager.devices().device(device).is_none());
        assert_eq!(
            manager.pointer_settings(device, PointerSettings::default()),
            PointerSettings::default()
        );
        assert!(manager.take_buttons_released_by_device(device).is_empty());
    }

    #[test]
    fn reconnecting_same_id_after_disconnect_starts_with_clean_runtime_state() {
        let mut manager = PointerManager::new();
        let device = InputDeviceId(15);
        assert!(manager.connect(descriptor(15, InputDeviceKind::Mouse)));

        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.5, false)
            .expect("valid pointer profile");
        assert!(manager.set_pointer_settings(device, pointer));
        manager.update_button(device, 0x110, true);

        // Disconnect consumes the runtime state; a new connection with the
        // same kernel-derived ID must not inherit the previous session's data.
        assert_eq!(manager.take_buttons_released_by_device(device), vec![0x110]);
        assert!(manager.disconnect(device));
        assert!(manager.connect(descriptor(15, InputDeviceKind::Mouse)));

        assert_eq!(
            manager.pointer_settings(device, PointerSettings::default()),
            PointerSettings::default()
        );
        assert_eq!(
            manager.touchpad_settings(device),
            TouchpadSettings::default()
        );
        assert!(manager.take_buttons_released_by_device(device).is_empty());
        assert_eq!(
            manager.devices().device(device).unwrap().kind,
            InputDeviceKind::Mouse
        );
    }

    #[test]
    fn all_pointing_device_classes_can_coexist_without_exclusive_ownership() {
        let mut manager = PointerManager::new();
        let devices = [
            (InputDeviceId(21), InputDeviceKind::Mouse),
            (InputDeviceId(22), InputDeviceKind::Touchpad),
            (InputDeviceId(23), InputDeviceKind::Touchscreen),
            (InputDeviceId(24), InputDeviceKind::Tablet),
            (InputDeviceId(25), InputDeviceKind::TabletPad),
        ];

        for (id, kind) in devices {
            assert!(manager.connect(descriptor(id.0, kind)));
        }
        for (id, kind) in devices {
            assert_eq!(manager.devices().device(id).unwrap().kind, kind);
        }

        // Removing one device must not invalidate any other connected device.
        assert!(manager.disconnect(InputDeviceId(22)));
        assert!(manager.devices().device(InputDeviceId(21)).is_some());
        assert!(manager.devices().device(InputDeviceId(23)).is_some());
        assert!(manager.devices().device(InputDeviceId(24)).is_some());
        assert!(manager.devices().device(InputDeviceId(25)).is_some());
    }

    #[test]
    fn touchpad_preferences_reject_mouse_devices() {
        let mut manager = PointerManager::new();
        let mouse = InputDeviceId(12);
        assert!(manager.connect(descriptor(12, InputDeviceKind::Mouse)));
        assert!(!manager.set_touchpad_settings(mouse, TouchpadSettings::default()));
    }
}
