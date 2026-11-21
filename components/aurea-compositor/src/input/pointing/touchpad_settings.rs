// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Applies per-device touchpad preferences through libinput.

use aurea_desktop_api::input::{
    TouchpadApplyReport, TouchpadSettingStatus, TouchpadSettings,
};
use smithay::reexports::input::{Device, DeviceCapability, ScrollMethod};

fn status(result: Result<(), smithay::reexports::input::DeviceConfigError>) -> TouchpadSettingStatus {
    match result {
        Ok(()) => TouchpadSettingStatus::Applied,
        Err(smithay::reexports::input::DeviceConfigError::Unsupported) => {
            TouchpadSettingStatus::Unsupported
        }
        Err(_) => TouchpadSettingStatus::Failed,
    }
}

/// Applies supported settings to a connected touchpad and reports each result.
///
/// This function intentionally does not cache settings: libinput owns the
/// active device configuration and every hotplugged device is configured
/// independently.
pub(super) fn apply(
    device: &mut Device,
    settings: TouchpadSettings,
) -> Option<TouchpadApplyReport> {
    // A pen tablet can expose pointer-like capabilities, but it is not a
    // touchpad. Never apply tap/scroll/DWT preferences to tablet tools or pads.
    if !device.has_capability(DeviceCapability::Pointer)
        || device.has_capability(DeviceCapability::Touch)
        || device.has_capability(DeviceCapability::TabletTool)
        || device.has_capability(DeviceCapability::TabletPad)
    {
        return None;
    }

    // A generic mouse also exposes Pointer. Require at least one libinput
    // touchpad-specific capability before attempting touchpad configuration.
    let is_touchpad = device.has_capability(DeviceCapability::Gesture)
        || device.config_tap_finger_count() > 0
        || device.config_dwt_is_available();
    if !is_touchpad {
        return None;
    }

    let tap_to_click = if device.config_tap_finger_count() > 0 {
        status(device.config_tap_set_enabled(settings.tap_to_click()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let scroll_methods = device.config_scroll_methods();
    let requested_scroll_method = if settings.two_finger_scroll() {
        ScrollMethod::TwoFinger
    } else {
        ScrollMethod::NoScroll
    };
    let two_finger_scroll = if scroll_methods.contains(&requested_scroll_method) {
        status(device.config_scroll_set_method(requested_scroll_method))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let natural_scrolling = if device.config_scroll_has_natural_scroll() {
        status(device.config_scroll_set_natural_scroll_enabled(settings.natural_scrolling()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let disable_while_typing = if device.config_dwt_is_available() {
        status(device.config_dwt_set_enabled(settings.disable_while_typing()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let tap_and_drag = if device.config_tap_finger_count() > 0 {
        status(device.config_tap_set_drag_enabled(settings.tap_and_drag()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    Some(TouchpadApplyReport {
        tap_to_click,
        two_finger_scroll,
        natural_scrolling,
        disable_while_typing,
        tap_and_drag,
    })
}
