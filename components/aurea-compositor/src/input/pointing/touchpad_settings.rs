// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Applies per-device touchpad preferences through libinput.

use aurea_desktop_api::input::{
    TouchpadApplyReport, TouchpadClickMethod, TouchpadSettingStatus, TouchpadSettings,
};
use smithay::reexports::input::{
    AccelProfile, ClickMethod, Device, DeviceConfigError, ScrollMethod,
};

use super::touchpad;

fn status(result: Result<(), DeviceConfigError>) -> TouchpadSettingStatus {
    match result {
        Ok(()) => TouchpadSettingStatus::Applied,
        Err(DeviceConfigError::Unsupported) => TouchpadSettingStatus::Unsupported,
        Err(_) => TouchpadSettingStatus::Failed,
    }
}

/// Applies every requested setting supported by this physical touchpad.
///
/// Capabilities are checked independently: a device can support scrolling but
/// not tapping, or pointer speed but not software click methods. Unsupported
/// options are reported instead of being silently treated as successful.
pub(super) fn apply(
    device: &mut Device,
    settings: TouchpadSettings,
) -> Option<TouchpadApplyReport> {
    if !touchpad::supports(device) {
        return None;
    }

    let tap_fingers = device.config_tap_finger_count();
    let tap_to_click = if tap_fingers > 0 {
        status(device.config_tap_set_enabled(settings.tap_to_click()))
    } else {
        TouchpadSettingStatus::Unsupported
    };
    let tap_and_drag = if tap_fingers > 0 {
        status(device.config_tap_set_drag_enabled(settings.tap_and_drag()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let scroll_methods = device.config_scroll_methods();
    let requested_scroll_method = if settings.edge_scrolling() {
        ScrollMethod::Edge
    } else if settings.two_finger_scroll() {
        ScrollMethod::TwoFinger
    } else {
        ScrollMethod::NoScroll
    };
    let selected_scroll_status = if scroll_methods.contains(&requested_scroll_method) {
        status(device.config_scroll_set_method(requested_scroll_method))
    } else {
        TouchpadSettingStatus::Unsupported
    };
    let (two_finger_scroll, edge_scrolling) = match requested_scroll_method {
        ScrollMethod::TwoFinger => (selected_scroll_status, TouchpadSettingStatus::Unsupported),
        ScrollMethod::Edge => (TouchpadSettingStatus::Unsupported, selected_scroll_status),
        ScrollMethod::NoScroll => (selected_scroll_status, TouchpadSettingStatus::Unsupported),
        _ => (TouchpadSettingStatus::Unsupported, TouchpadSettingStatus::Unsupported),
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

    let requested_click_method = match settings.click_method() {
        TouchpadClickMethod::ButtonAreas => ClickMethod::ButtonAreas,
        TouchpadClickMethod::Clickfinger => ClickMethod::Clickfinger,
    };
    let click_methods = device.config_click_methods();
    let click_method = if click_methods.contains(&requested_click_method) {
        status(device.config_click_set_method(requested_click_method))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let left_handed = if device.config_left_handed_is_available() {
        status(device.config_left_handed_set(settings.left_handed()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let middle_button_emulation = if device.config_middle_emulation_is_available() {
        status(device.config_middle_emulation_set_enabled(settings.middle_button_emulation()))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    let pointer_speed = if device.config_accel_is_available() {
        status(device.config_accel_set_speed(
            f64::from(settings.pointer_speed_percent()) / 100.0,
        ))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    // Keep a stable adaptive profile when supported. The user's speed slider
    // remains independent and is applied above.
    if device.config_accel_profiles().contains(&AccelProfile::Adaptive) {
        let _ = device.config_accel_set_profile(AccelProfile::Adaptive);
    }

    Some(TouchpadApplyReport {
        tap_to_click,
        two_finger_scroll,
        edge_scrolling,
        natural_scrolling,
        disable_while_typing,
        tap_and_drag,
        click_method,
        left_handed,
        middle_button_emulation,
        pointer_speed,
    })
}
