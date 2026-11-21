// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Touchpad-class device detection using libinput's capabilities.

use smithay::reexports::input::{Device, DeviceCapability};

pub(super) fn supports(device: &Device) -> bool {
    let has_pointer = device.has_capability(DeviceCapability::Pointer);
    let has_gesture_events = device.has_capability(DeviceCapability::Gesture);
    let has_touchpad_settings =
        device.config_tap_finger_count() > 0 || device.config_dwt_is_available();

    has_pointer && (has_gesture_events || has_touchpad_settings)
}
