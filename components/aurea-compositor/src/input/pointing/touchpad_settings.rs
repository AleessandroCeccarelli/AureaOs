// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Applies per-device touchpad preferences through libinput.

use aurea_desktop_api::input::{TouchpadApplyReport, TouchpadSettingStatus, TouchpadSettings};
use smithay::reexports::input::{Device, DeviceConfigError, DeviceConfigResult, DeviceCapability, ScrollMethod};

use super::touchpad;

fn status(result: DeviceConfigResult) -> TouchpadSettingStatus {
    match result {
        Ok(()) => TouchpadSettingStatus::Applied,
        Err(DeviceConfigError::Unsupported) => TouchpadSettingStatus::Unsupported,
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
    // Share the exact discovery predicate so a scroll-only touchpad is not
    // classified as a touchpad in one path and rejected in the other.
    if !touchpad::supports(device) {
        return None;
    }

    // A generic mouse also exposes Pointer. Require at least one libinput
    // touchpad-specific configuration capability before attempting settings.
    if !device.has_capability(DeviceCapability::Pointer) {
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
