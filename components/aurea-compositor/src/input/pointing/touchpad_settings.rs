// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Applies per-device touchpad preferences through libinput.

use aurea_desktop_api::input::{
    TouchpadAccelerationProfile, TouchpadApplyReport, TouchpadClickMethod,
    TouchpadSendEventsMode, TouchpadSettingStatus,
    TouchpadSettings, TouchpadTapButtonMap,
};
use smithay::reexports::input::{
    AccelProfile, ClickMethod, Device, DeviceConfigError, ScrollMethod, SendEventsMode,
    TapButtonMap,
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
    let tap_drag_lock = if tap_fingers > 0 {
        status(device.config_tap_set_drag_lock_enabled(settings.tap_drag_lock()))
    } else {
        TouchpadSettingStatus::Unsupported
    };
    let tap_button_map = if tap_fingers > 0 {
        let map = match settings.tap_button_map() {
            TouchpadTapButtonMap::LeftRightMiddle => TapButtonMap::LeftRightMiddle,
            TouchpadTapButtonMap::LeftMiddleRight => TapButtonMap::LeftMiddleRight,
        };
        status(device.config_tap_set_button_map(map))
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
        ScrollMethod::TwoFinger => (selected_scroll_status, TouchpadSettingStatus::Applied),
        ScrollMethod::Edge => (TouchpadSettingStatus::Applied, selected_scroll_status),
        ScrollMethod::NoScroll => (selected_scroll_status, selected_scroll_status),
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

    #[cfg(feature = "drm")]
    let disable_while_trackpointing = if device.config_dwtp_is_available() {
        status(device.config_dwtp_set_enabled(
            settings.disable_while_trackpointing(),
        ))
    } else {
        TouchpadSettingStatus::Unsupported
    };
    #[cfg(not(feature = "drm"))]
    let disable_while_trackpointing = TouchpadSettingStatus::Unsupported;

    let click_methods = device.config_click_methods();
    let click_method = match settings.click_method() {
        TouchpadClickMethod::DeviceDefault if !click_methods.is_empty() => {
            TouchpadSettingStatus::Applied
        }
        TouchpadClickMethod::DeviceDefault => TouchpadSettingStatus::Unsupported,
        TouchpadClickMethod::ButtonAreas if click_methods.contains(&ClickMethod::ButtonAreas) => {
            status(device.config_click_set_method(ClickMethod::ButtonAreas))
        }
        TouchpadClickMethod::Clickfinger if click_methods.contains(&ClickMethod::Clickfinger) => {
            status(device.config_click_set_method(ClickMethod::Clickfinger))
        }
        _ => TouchpadSettingStatus::Unsupported,
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

    let acceleration_profiles = device.config_accel_profiles();
    let acceleration_profile = match settings.acceleration_profile() {
        TouchpadAccelerationProfile::DeviceDefault if !acceleration_profiles.is_empty() => {
            TouchpadSettingStatus::Applied
        }
        TouchpadAccelerationProfile::DeviceDefault => TouchpadSettingStatus::Unsupported,
        TouchpadAccelerationProfile::Flat if acceleration_profiles.contains(&AccelProfile::Flat) => {
            status(device.config_accel_set_profile(AccelProfile::Flat))
        }
        TouchpadAccelerationProfile::Adaptive
            if acceleration_profiles.contains(&AccelProfile::Adaptive) =>
        {
            status(device.config_accel_set_profile(AccelProfile::Adaptive))
        }
        _ => TouchpadSettingStatus::Unsupported,
    };

    let send_events_mode = match settings.send_events_mode() {
        TouchpadSendEventsMode::Enabled => SendEventsMode::ENABLED,
        TouchpadSendEventsMode::Disabled => SendEventsMode::DISABLED,
        TouchpadSendEventsMode::DisabledOnExternalMouse => {
            SendEventsMode::DISABLED_ON_EXTERNAL_MOUSE
        }
    };
    let available_send_events_modes = device.config_send_events_modes();
    let send_events_supported = match settings.send_events_mode() {
        // ENABLED is the zero-valued placeholder and is valid for every device.
        TouchpadSendEventsMode::Enabled => true,
        TouchpadSendEventsMode::Disabled => {
            available_send_events_modes.contains(SendEventsMode::DISABLED)
        }
        TouchpadSendEventsMode::DisabledOnExternalMouse => {
            available_send_events_modes.contains(SendEventsMode::DISABLED_ON_EXTERNAL_MOUSE)
        }
    };
    let send_events_mode = if send_events_supported {
        status(device.config_send_events_set_mode(send_events_mode))
    } else {
        TouchpadSettingStatus::Unsupported
    };

    Some(TouchpadApplyReport {
        tap_to_click,
        tap_and_drag,
        tap_drag_lock,
        tap_button_map,
        two_finger_scroll,
        edge_scrolling,
        natural_scrolling,
        disable_while_typing,
        disable_while_trackpointing,
        click_method,
        left_handed,
        middle_button_emulation,
        pointer_speed,
        acceleration_profile,
        send_events_mode,
    })
}
