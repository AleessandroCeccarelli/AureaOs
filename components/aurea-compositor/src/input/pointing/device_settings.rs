// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device preferences for physical pointing hardware.

use std::collections::HashMap;

use aurea_desktop_api::input::{
    InputDeviceId, InputDeviceKind, InputDeviceProvider, PointerSettings, TouchpadSettings,
};

/// Owns overrides independently from the compositor's protocol state.
#[derive(Default)]
pub struct PointingDeviceSettings {
    pointers: HashMap<InputDeviceId, PointerSettings>,
    touchpads: HashMap<InputDeviceId, TouchpadSettings>,
}

impl PointingDeviceSettings {
    pub fn pointer(&self, id: InputDeviceId, fallback: PointerSettings) -> PointerSettings {
        self.pointers.get(&id).copied().unwrap_or(fallback)
    }

    pub fn set_pointer(
        &mut self,
        provider: &impl InputDeviceProvider,
        id: InputDeviceId,
        settings: PointerSettings,
    ) -> bool {
        if !provider.device(id).is_some_and(|device| {
            device.kind == InputDeviceKind::Mouse
        }) {
            return false;
        }
        self.pointers.insert(id, settings);
        true
    }

    pub fn touchpad(&self, id: InputDeviceId) -> TouchpadSettings {
        self.touchpads.get(&id).copied().unwrap_or_default()
    }

    pub fn set_touchpad(
        &mut self,
        provider: &impl InputDeviceProvider,
        id: InputDeviceId,
        settings: TouchpadSettings,
    ) -> bool {
        if !provider
            .device(id)
            .is_some_and(|device| device.kind == InputDeviceKind::Touchpad)
        {
            return false;
        }
        self.touchpads.insert(id, settings);
        true
    }

    /// Removes overrides that are no longer valid after a device is reclassified.
    pub fn reconcile_kind(&mut self, id: InputDeviceId, kind: InputDeviceKind) {
        if kind != InputDeviceKind::Mouse {
            self.pointers.remove(&id);
        }
        if kind != InputDeviceKind::Touchpad {
            self.touchpads.remove(&id);
        }
    }

    pub fn remove(&mut self, id: InputDeviceId) {
        self.pointers.remove(&id);
        self.touchpads.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurea_desktop_api::input::{
        InputDeviceCapabilities, InputDeviceDescriptor, InputDeviceEvent, InputDeviceProvider,
        PointerAccelerationProfile,
    };
    use std::sync::mpsc::Receiver;

    struct Devices(Vec<InputDeviceDescriptor>);

    impl InputDeviceProvider for Devices {
        fn devices(&self) -> Vec<InputDeviceDescriptor> {
            self.0.clone()
        }
        fn device(&self, id: InputDeviceId) -> Option<InputDeviceDescriptor> {
            self.0.iter().find(|device| device.id == id).cloned()
        }
        fn subscribe(&self) -> Receiver<InputDeviceEvent> {
            let (_tx, rx) = std::sync::mpsc::channel();
            rx
        }
    }

    fn descriptor(id: u64, kind: InputDeviceKind) -> InputDeviceDescriptor {
        InputDeviceDescriptor {
            id: InputDeviceId(id),
            kind,
            name: format!("Device {id}"),
            vendor_id: None,
            product_id: None,
            persistent_id: None,
            capabilities: InputDeviceCapabilities::default(),
        }
    }

    #[test]
    fn profiles_are_independent_and_fall_back_to_defaults() {
        let devices = Devices(vec![
            descriptor(1, InputDeviceKind::Mouse),
            descriptor(2, InputDeviceKind::Touchpad),
        ]);
        let mut settings = PointingDeviceSettings::default();
        let mouse = PointerSettings::new(PointerAccelerationProfile::Flat, 0.7, false).unwrap();
        assert!(settings.set_pointer(&devices, InputDeviceId(1), mouse));
        assert!(!settings.set_pointer(
            &devices,
            InputDeviceId(2),
            PointerSettings::default()
        ));
        assert_eq!(
            settings.pointer(InputDeviceId(1), PointerSettings::default()),
            mouse
        );
        assert_eq!(
            settings.pointer(InputDeviceId(2), PointerSettings::default()),
            PointerSettings::default()
        );
        assert_eq!(
            settings.touchpad(InputDeviceId(2)),
            TouchpadSettings::default()
        );
    }

    #[test]
    fn touchpad_profile_rejects_non_touchpad_devices() {
        let devices = Devices(vec![descriptor(3, InputDeviceKind::Mouse)]);
        let mut settings = PointingDeviceSettings::default();
        assert!(!settings.set_touchpad(&devices, InputDeviceId(3), TouchpadSettings::default()));
    }

    #[test]
    fn reclassification_clears_only_incompatible_overrides() {
        let devices = Devices(vec![descriptor(5, InputDeviceKind::Mouse)]);
        let mut settings = PointingDeviceSettings::default();
        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.4, false).unwrap();
        assert!(settings.set_pointer(&devices, InputDeviceId(5), pointer));

        settings.reconcile_kind(InputDeviceId(5), InputDeviceKind::Touchpad);
        assert_eq!(settings.pointer(InputDeviceId(5), PointerSettings::default()), PointerSettings::default());
        assert_eq!(settings.touchpad(InputDeviceId(5)), TouchpadSettings::default());

        settings.reconcile_kind(InputDeviceId(5), InputDeviceKind::Tablet);
        assert_eq!(settings.pointer(InputDeviceId(5), PointerSettings::default()), PointerSettings::default());
    }

    #[test]
    fn removal_clears_all_overrides() {
        let devices = Devices(vec![descriptor(4, InputDeviceKind::Mouse)]);
        let mut settings = PointingDeviceSettings::default();
        let pointer = PointerSettings::new(PointerAccelerationProfile::Flat, 0.5, true).unwrap();
        assert!(settings.set_pointer(&devices, InputDeviceId(4), pointer));

        settings.remove(InputDeviceId(4));
        assert_eq!(
            settings.pointer(InputDeviceId(4), PointerSettings::default()),
            PointerSettings::default()
        );

    }
}
