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
    let has_gesture_events = device.has_capability(DeviceCapability::Gesture);
    let has_touchpad_settings =
        device.config_tap_finger_count() > 0 || device.config_dwt_is_available();

    has_pointer && !is_direct_touch && (has_gesture_events || has_touchpad_settings)
}
