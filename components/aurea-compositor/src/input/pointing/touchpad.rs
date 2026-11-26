// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Touchpad-class device detection using libinput's capabilities.

use aurea_desktop_api::input::TouchpadCapabilities;
use smithay::reexports::input::{
    AccelProfile, ClickMethod, Device, DeviceCapability, ScrollMethod,
};

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
            !device.config_click_methods().is_empty(),
        )
}

/// Captures the exact configuration surface libinput exposes for this device.
///
/// The result is suitable for a settings UI: it describes the connected
/// hardware/driver combination, not a vendor-wide promise.
pub(super) fn capabilities(device: &Device) -> TouchpadCapabilities {
    let scroll_methods = device.config_scroll_methods();
    let click_methods = device.config_click_methods();
    let acceleration_profiles = device.config_accel_profiles();

    #[cfg(feature = "drm")]
    let disable_while_trackpointing = device.config_dwtp_is_available();
    #[cfg(not(feature = "drm"))]
    let disable_while_trackpointing = false;

    TouchpadCapabilities {
        max_tap_fingers: device.config_tap_finger_count(),
        tap_to_click: device.config_tap_finger_count() > 0,
        tap_and_drag: device.config_tap_finger_count() > 0,
        tap_drag_lock: device.config_tap_finger_count() > 0,
        tap_button_map_configurable: device.config_tap_finger_count() > 0,
        two_finger_scroll: scroll_methods.contains(&ScrollMethod::TwoFinger),
        edge_scrolling: scroll_methods.contains(&ScrollMethod::Edge),
        natural_scrolling: device.config_scroll_has_natural_scroll(),
        button_areas: click_methods.contains(&ClickMethod::ButtonAreas),
        clickfinger: click_methods.contains(&ClickMethod::Clickfinger),
        disable_while_typing: device.config_dwt_is_available(),
        disable_while_trackpointing,
        gestures: device.has_capability(DeviceCapability::Gesture),
        left_handed: device.config_left_handed_is_available(),
        middle_button_emulation: device.config_middle_emulation_is_available(),
        pointer_acceleration: device.config_accel_is_available(),
        flat_acceleration: acceleration_profiles.contains(&AccelProfile::Flat),
        adaptive_acceleration: acceleration_profiles.contains(&AccelProfile::Adaptive),
        disable_events: device
            .config_send_events_modes()
            .contains(smithay::reexports::input::SendEventsMode::DISABLED),
        disable_on_external_mouse: device
            .config_send_events_modes()
            .contains(smithay::reexports::input::SendEventsMode::DISABLED_ON_EXTERNAL_MOUSE),
    }
}

/// libinput exposes different touchpad configuration features depending on
/// hardware and driver support. Scroll-only devices are still touchpads even
/// when tap-to-click, DWT and gesture capabilities are unavailable.
fn exposes_touchpad_configuration(
    has_gesture_events: bool,
    has_tap_configuration: bool,
    has_disable_while_typing: bool,
    has_scroll_methods: bool,
    has_click_methods: bool,
) -> bool {
    has_gesture_events
        || has_tap_configuration
        || has_disable_while_typing
        || has_scroll_methods
        || has_click_methods
}

#[cfg(test)]
mod tests {
    use super::exposes_touchpad_configuration;

    #[test]
    fn recognizes_scroll_only_touchpads() {
        assert!(exposes_touchpad_configuration(
            false, false, false, true, false
        ));
    }

    #[test]
    fn recognizes_each_other_touchpad_capability_independently() {
        assert!(exposes_touchpad_configuration(
            true, false, false, false, false
        ));
        assert!(exposes_touchpad_configuration(
            false, true, false, false, false
        ));
        assert!(exposes_touchpad_configuration(
            false, false, true, false, false
        ));
        assert!(exposes_touchpad_configuration(
            false, false, false, false, true
        ));
    }

    #[test]
    fn rejects_devices_without_touchpad_specific_configuration() {
        assert!(!exposes_touchpad_configuration(
            false, false, false, false, false
        ));
    }
}
