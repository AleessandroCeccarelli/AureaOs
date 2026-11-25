// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Tracks touchpad profiles already applied by the live input backend.

use std::collections::HashMap;

use aurea_desktop_api::input::{InputDeviceId, TouchpadSettings};

/// Records applied values, never retaining libinput device handles.
#[derive(Default)]
pub struct AppliedTouchpadSettings {
    profiles: HashMap<InputDeviceId, TouchpadSettings>,
}

impl AppliedTouchpadSettings {
    /// Returns true when the backend should apply the requested profile.
    pub fn needs_apply(&self, device: InputDeviceId, requested: TouchpadSettings) -> bool {
        self.profiles.get(&device) != Some(&requested)
    }

    /// Records a profile after the backend has attempted to apply it.
    pub fn mark_applied(&mut self, device: InputDeviceId, settings: TouchpadSettings) {
        self.profiles.insert(device, settings);
    }

    /// Drops the runtime record when a physical device disconnects.
    pub fn remove(&mut self, device: InputDeviceId) -> bool {
        self.profiles.remove(&device).is_some()
    }

    /// Invalidates only the selected device after its backend state may have reset.
    pub fn invalidate(&mut self, device: InputDeviceId) -> bool {
        self.profiles.remove(&device).is_some()
    }

    /// Number of devices with a known applied profile.
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidating_one_device_preserves_other_devices_applied_profiles() {
        let mut applied = AppliedTouchpadSettings::default();
        let settings = TouchpadSettings::default();
        let first = InputDeviceId(1);
        let second = InputDeviceId(2);
        applied.mark_applied(first, settings);
        applied.mark_applied(second, settings);

        assert!(applied.invalidate(first));

        assert_eq!(applied.len(), 1);
        assert!(applied.needs_apply(first, settings));
        assert!(!applied.needs_apply(second, settings));
    }

    #[test]
    fn new_or_changed_profile_requires_application() {
        let mut applied = AppliedTouchpadSettings::default();
        let first = TouchpadSettings::default();
        let changed = TouchpadSettings::new(false, true, false, true, false);

        assert!(applied.needs_apply(InputDeviceId(1), first));
        applied.mark_applied(InputDeviceId(1), first);
        assert!(!applied.needs_apply(InputDeviceId(1), first));
        assert!(applied.needs_apply(InputDeviceId(1), changed));
    }

    #[test]
    fn profiles_are_isolated_between_devices() {
        let mut applied = AppliedTouchpadSettings::default();
        let settings = TouchpadSettings::default();

        applied.mark_applied(InputDeviceId(1), settings);

        assert!(!applied.needs_apply(InputDeviceId(1), settings));
        assert!(applied.needs_apply(InputDeviceId(2), settings));
        assert_eq!(applied.len(), 1);
    }

    #[test]
    fn disconnect_clears_profile_and_allows_clean_reconnect() {
        let mut applied = AppliedTouchpadSettings::default();
        let settings = TouchpadSettings::default();
        let id = InputDeviceId(7);

        applied.mark_applied(id, settings);
        assert!(applied.remove(id));
        assert!(!applied.remove(id));
        assert!(applied.needs_apply(id, settings));
        assert!(applied.is_empty());
    }
}
