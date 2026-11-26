// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent touchpad preferences and capability reporting.

use serde::{Deserialize, Serialize};

/// Preferred software clickpad behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TouchpadClickMethod {
    /// Preserve the hardware/libinput default for this device.
    #[default]
    DeviceDefault,
    /// Use the physical/software button areas exposed by the device.
    ButtonAreas,
    /// Infer left/right/middle click from the number of fingers.
    Clickfinger,
}

/// Preferred pointer acceleration curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TouchpadAccelerationProfile {
    /// Preserve the profile selected by libinput for this device.
    #[default]
    DeviceDefault,
    /// Constant acceleration factor.
    Flat,
    /// Speed-sensitive acceleration.
    Adaptive,
}

/// Finger-tap to mouse-button mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TouchpadTapButtonMap {
    /// One/two/three fingers map to left/right/middle.
    #[default]
    LeftRightMiddle,
    /// One/two/three fingers map to left/middle/right.
    LeftMiddleRight,
}

/// Whether libinput should forward events from the touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TouchpadSendEventsMode {
    /// Normal operation.
    #[default]
    Enabled,
    /// Disable this touchpad until explicitly re-enabled.
    Disabled,
    /// Disable this touchpad while an external pointer is connected.
    DisabledOnExternalMouse,
}

/// User-configurable touchpad behavior.
///
/// The five-argument constructor is kept stable for existing callers. More
/// advanced preferences are opt-in through builder-style methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TouchpadSettings {
    tap_to_click: bool,
    two_finger_scroll: bool,
    natural_scrolling: bool,
    disable_while_typing: bool,
    disable_while_trackpointing: bool,
    tap_and_drag: bool,
    edge_scrolling: bool,
    tap_drag_lock: bool,
    click_method: TouchpadClickMethod,
    tap_button_map: TouchpadTapButtonMap,
    left_handed: bool,
    middle_button_emulation: bool,
    pointer_speed_percent: i8,
    acceleration_profile: TouchpadAccelerationProfile,
    send_events_mode: TouchpadSendEventsMode,
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
            tap_to_click: true,
            two_finger_scroll: true,
            natural_scrolling: false,
            disable_while_typing: true,
            disable_while_trackpointing: false,
            tap_and_drag: true,
            edge_scrolling: false,
            tap_drag_lock: false,
            click_method: TouchpadClickMethod::DeviceDefault,
            tap_button_map: TouchpadTapButtonMap::LeftRightMiddle,
            left_handed: false,
            middle_button_emulation: false,
            pointer_speed_percent: 0,
            acceleration_profile: TouchpadAccelerationProfile::DeviceDefault,
            send_events_mode: TouchpadSendEventsMode::Enabled,
        }
    }
}

impl TouchpadSettings {
    pub const fn new(
        tap_to_click: bool,
        two_finger_scroll: bool,
        natural_scrolling: bool,
        disable_while_typing: bool,
        tap_and_drag: bool,
    ) -> Self {
        Self {
            tap_to_click,
            two_finger_scroll,
            natural_scrolling,
            disable_while_typing,
            disable_while_trackpointing: false,
            tap_and_drag,
            edge_scrolling: false,
            tap_drag_lock: false,
            click_method: TouchpadClickMethod::DeviceDefault,
            tap_button_map: TouchpadTapButtonMap::LeftRightMiddle,
            left_handed: false,
            middle_button_emulation: false,
            pointer_speed_percent: 0,
            acceleration_profile: TouchpadAccelerationProfile::DeviceDefault,
            send_events_mode: TouchpadSendEventsMode::Enabled,
        }
    }

    pub const fn with_edge_scrolling(mut self, enabled: bool) -> Self {
        self.edge_scrolling = enabled;
        self
    }

    pub const fn with_disable_while_trackpointing(mut self, enabled: bool) -> Self {
        self.disable_while_trackpointing = enabled;
        self
    }

    pub const fn with_tap_drag_lock(mut self, enabled: bool) -> Self {
        self.tap_drag_lock = enabled;
        self
    }

    pub const fn with_click_method(mut self, method: TouchpadClickMethod) -> Self {
        self.click_method = method;
        self
    }

    pub const fn with_tap_button_map(mut self, map: TouchpadTapButtonMap) -> Self {
        self.tap_button_map = map;
        self
    }

    pub const fn with_left_handed(mut self, enabled: bool) -> Self {
        self.left_handed = enabled;
        self
    }

    pub const fn with_middle_button_emulation(mut self, enabled: bool) -> Self {
        self.middle_button_emulation = enabled;
        self
    }

    pub const fn with_pointer_speed_percent(mut self, percent: i8) -> Self {
        self.pointer_speed_percent = if percent < -100 {
            -100
        } else if percent > 100 {
            100
        } else {
            percent
        };
        self
    }

    pub const fn with_acceleration_profile(
        mut self,
        profile: TouchpadAccelerationProfile,
    ) -> Self {
        self.acceleration_profile = profile;
        self
    }

    pub const fn with_send_events_mode(mut self, mode: TouchpadSendEventsMode) -> Self {
        self.send_events_mode = mode;
        self
    }

    pub const fn tap_to_click(self) -> bool { self.tap_to_click }
    pub const fn two_finger_scroll(self) -> bool { self.two_finger_scroll && !self.edge_scrolling }
    pub const fn natural_scrolling(self) -> bool { self.natural_scrolling }
    pub const fn disable_while_typing(self) -> bool { self.disable_while_typing }
    pub const fn disable_while_trackpointing(self) -> bool { self.disable_while_trackpointing }
    pub const fn tap_and_drag(self) -> bool { self.tap_and_drag }
    pub const fn edge_scrolling(self) -> bool { self.edge_scrolling }
    pub const fn tap_drag_lock(self) -> bool { self.tap_drag_lock }
    pub const fn click_method(self) -> TouchpadClickMethod { self.click_method }
    pub const fn tap_button_map(self) -> TouchpadTapButtonMap { self.tap_button_map }
    pub const fn left_handed(self) -> bool { self.left_handed }
    pub const fn middle_button_emulation(self) -> bool { self.middle_button_emulation }
    pub const fn pointer_speed_percent(self) -> i8 { self.pointer_speed_percent }
    pub const fn acceleration_profile(self) -> TouchpadAccelerationProfile { self.acceleration_profile }
    pub const fn send_events_mode(self) -> TouchpadSendEventsMode { self.send_events_mode }
}

/// Outcome of applying one setting to one physical touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchpadSettingStatus {
    /// The requested operation was applied successfully.
    Applied,
    /// The device does not expose the requested capability.
    Unsupported,
    /// This option is inactive because another mutually exclusive option was selected.
    NotRequested,
    /// The device exposed the capability but libinput rejected the operation.
    Failed,
}

/// Per-setting result. Unsupported hardware is never reported as configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchpadApplyReport {
    pub tap_to_click: TouchpadSettingStatus,
    pub tap_and_drag: TouchpadSettingStatus,
    pub tap_drag_lock: TouchpadSettingStatus,
    pub tap_button_map: TouchpadSettingStatus,
    pub two_finger_scroll: TouchpadSettingStatus,
    pub edge_scrolling: TouchpadSettingStatus,
    pub natural_scrolling: TouchpadSettingStatus,
    pub disable_while_typing: TouchpadSettingStatus,
    pub disable_while_trackpointing: TouchpadSettingStatus,
    pub click_method: TouchpadSettingStatus,
    pub left_handed: TouchpadSettingStatus,
    pub middle_button_emulation: TouchpadSettingStatus,
    pub pointer_speed: TouchpadSettingStatus,
    pub acceleration_profile: TouchpadSettingStatus,
    pub send_events_mode: TouchpadSettingStatus,
}

impl TouchpadApplyReport {
    pub const fn all_applied(self) -> bool {
        matches!(self.tap_to_click, TouchpadSettingStatus::Applied)
            && matches!(self.tap_and_drag, TouchpadSettingStatus::Applied)
            && matches!(self.tap_drag_lock, TouchpadSettingStatus::Applied)
            && matches!(self.tap_button_map, TouchpadSettingStatus::Applied)
            && matches!(self.two_finger_scroll, TouchpadSettingStatus::Applied | TouchpadSettingStatus::NotRequested)
            && matches!(self.edge_scrolling, TouchpadSettingStatus::Applied | TouchpadSettingStatus::NotRequested)
            && matches!(self.natural_scrolling, TouchpadSettingStatus::Applied)
            && matches!(self.disable_while_typing, TouchpadSettingStatus::Applied)
            && matches!(self.disable_while_trackpointing, TouchpadSettingStatus::Applied)
            && matches!(self.click_method, TouchpadSettingStatus::Applied)
            && matches!(self.left_handed, TouchpadSettingStatus::Applied)
            && matches!(self.middle_button_emulation, TouchpadSettingStatus::Applied)
            && matches!(self.pointer_speed, TouchpadSettingStatus::Applied)
            && matches!(self.acceleration_profile, TouchpadSettingStatus::Applied)
            && matches!(self.send_events_mode, TouchpadSettingStatus::Applied)
    }

    /// Returns true when libinput rejected at least one supported setting.
    /// Unsupported settings are hardware capability limitations, not failures.
    pub const fn has_failures(self) -> bool {
        matches!(self.tap_to_click, TouchpadSettingStatus::Failed)
            || matches!(self.tap_and_drag, TouchpadSettingStatus::Failed)
            || matches!(self.tap_drag_lock, TouchpadSettingStatus::Failed)
            || matches!(self.tap_button_map, TouchpadSettingStatus::Failed)
            || matches!(self.two_finger_scroll, TouchpadSettingStatus::Failed)
            || matches!(self.edge_scrolling, TouchpadSettingStatus::Failed)
            || matches!(self.natural_scrolling, TouchpadSettingStatus::Failed)
            || matches!(self.disable_while_typing, TouchpadSettingStatus::Failed)
            || matches!(self.disable_while_trackpointing, TouchpadSettingStatus::Failed)
            || matches!(self.click_method, TouchpadSettingStatus::Failed)
            || matches!(self.left_handed, TouchpadSettingStatus::Failed)
            || matches!(self.middle_button_emulation, TouchpadSettingStatus::Failed)
            || matches!(self.pointer_speed, TouchpadSettingStatus::Failed)
            || matches!(self.acceleration_profile, TouchpadSettingStatus::Failed)
            || matches!(self.send_events_mode, TouchpadSettingStatus::Failed)
    }

    #[test]
    fn mutually_exclusive_scroll_option_is_reported_as_not_requested() {
        let mut result = report(TouchpadSettingStatus::Applied);
        result.edge_scrolling = TouchpadSettingStatus::NotRequested;
        assert!(result.all_applied());
        assert!(!result.has_failures());

        result.edge_scrolling = TouchpadSettingStatus::Unsupported;
        assert!(!result.all_applied());
    }

}

#[cfg(test)]
mod tests {
    use super::{
        TouchpadAccelerationProfile, TouchpadApplyReport, TouchpadClickMethod, TouchpadSendEventsMode,
        TouchpadSettingStatus, TouchpadSettings, TouchpadTapButtonMap,
    };

    #[test]
    fn defaults_enable_common_touchpad_behaviors() {
        let settings = TouchpadSettings::default();
        assert!(settings.tap_to_click());
        assert!(settings.two_finger_scroll());
        assert!(!settings.edge_scrolling());
        assert!(!settings.natural_scrolling());
        assert!(settings.disable_while_typing());
        assert!(!settings.disable_while_trackpointing());
        assert!(settings.tap_and_drag());
        assert!(!settings.tap_drag_lock());
        assert_eq!(settings.click_method(), TouchpadClickMethod::DeviceDefault);
        assert_eq!(settings.tap_button_map(), TouchpadTapButtonMap::LeftRightMiddle);
        assert_eq!(settings.pointer_speed_percent(), 0);
        assert_eq!(settings.acceleration_profile(), TouchpadAccelerationProfile::DeviceDefault);
        assert_eq!(settings.send_events_mode(), TouchpadSendEventsMode::Enabled);
    }

    #[test]
    fn advanced_preferences_are_bounded_and_select_one_scroll_mode() {
        let settings = TouchpadSettings::default()
            .with_edge_scrolling(true)
            .with_tap_drag_lock(true)
            .with_disable_while_trackpointing(true)
            .with_left_handed(true)
            .with_middle_button_emulation(true)
            .with_pointer_speed_percent(120)
            .with_click_method(TouchpadClickMethod::ButtonAreas)
            .with_tap_button_map(TouchpadTapButtonMap::LeftMiddleRight)
            .with_send_events_mode(TouchpadSendEventsMode::DisabledOnExternalMouse)
            .with_acceleration_profile(TouchpadAccelerationProfile::Flat);
        assert!(settings.edge_scrolling());
        assert!(!settings.two_finger_scroll());
        assert!(settings.tap_drag_lock());
        assert!(settings.disable_while_trackpointing());
        assert!(settings.left_handed());
        assert!(settings.middle_button_emulation());
        assert_eq!(settings.pointer_speed_percent(), 100);
        assert_eq!(settings.click_method(), TouchpadClickMethod::ButtonAreas);
        assert_eq!(settings.tap_button_map(), TouchpadTapButtonMap::LeftMiddleRight);
        assert_eq!(settings.send_events_mode(), TouchpadSendEventsMode::DisabledOnExternalMouse);
        assert_eq!(settings.acceleration_profile(), TouchpadAccelerationProfile::Flat);
    }

    fn report(status: TouchpadSettingStatus) -> TouchpadApplyReport {
        TouchpadApplyReport {
            tap_to_click: status,
            tap_and_drag: status,
            tap_drag_lock: status,
            tap_button_map: status,
            two_finger_scroll: status,
            edge_scrolling: status,
            natural_scrolling: status,
            disable_while_typing: status,
            disable_while_trackpointing: status,
            click_method: status,
            left_handed: status,
            middle_button_emulation: status,
            pointer_speed: status,
            acceleration_profile: status,
            send_events_mode: status,
        }
    }

    #[test]
    fn unsupported_settings_are_not_application_failures() {
        let mut result = report(TouchpadSettingStatus::Applied);
        result.two_finger_scroll = TouchpadSettingStatus::Unsupported;
        assert!(!result.has_failures());
        assert!(!result.all_applied());
    }

    #[test]
    fn failed_setting_is_detected_for_retry() {
        let mut result = report(TouchpadSettingStatus::Applied);
        result.tap_to_click = TouchpadSettingStatus::Failed;
        assert!(result.has_failures());
    }
}
