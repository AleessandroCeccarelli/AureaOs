// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Touchpad-class device detection using libinput's capabilities.

use smithay::reexports::input::{Device, DeviceCapability};

pub(super) fn supports(device: &Device) -> bool {
    let has_pointer = device.has_capability(DeviceCapability::Pointer);
    let is_direct_touch = device.has_capability(DeviceCapability::Touch)
        || device.has_capability(DeviceCapability::TabletTool)
        || device.has_capability(DeviceCapability::TabletPad);

    has_pointer
        && !is_direct_touch
        && exposes_touchpad_configuration(
            device.has_capability(DeviceCapability::Gesture),
            device.config_tap_finger_count() > 0,
            device.config_dwt_is_available(),
            !device.config_scroll_methods().is_empty(),
        )
}

/// libinput exposes different touchpad configuration features depending on
/// hardware and driver support. Scroll-only devices are still touchpads even
/// when tap-to-click, DWT and gesture capabilities are unavailable.
fn exposes_touchpad_configuration(
    has_gesture_events: bool,
    has_tap_configuration: bool,
    has_disable_while_typing: bool,
    has_scroll_methods: bool,
) -> bool {
    has_gesture_events
        || has_tap_configuration
        || has_disable_while_typing
        || has_scroll_methods
}

#[cfg(test)]
mod tests {
    use super::exposes_touchpad_configuration;

    #[test]
    fn recognizes_scroll_only_touchpads() {
        assert!(exposes_touchpad_configuration(false, false, false, true));
    }

    #[test]
    fn recognizes_each_other_touchpad_capability_independently() {
        assert!(exposes_touchpad_configuration(true, false, false, false));
        assert!(exposes_touchpad_configuration(false, true, false, false));
        assert!(exposes_touchpad_configuration(false, false, true, false));
    }

    #[test]
    fn rejects_devices_without_touchpad_specific_configuration() {
        assert!(!exposes_touchpad_configuration(false, false, false, false));
    }
}
