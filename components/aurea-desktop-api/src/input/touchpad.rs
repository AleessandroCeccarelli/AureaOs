// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent touchpad preferences and capability reporting.

/// User-configurable touchpad behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchpadSettings {
    tap_to_click: bool,
    two_finger_scroll: bool,
    natural_scrolling: bool,
    disable_while_typing: bool,
    tap_and_drag: bool,
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
            tap_to_click: true,
            two_finger_scroll: true,
            natural_scrolling: false,
            disable_while_typing: true,
            tap_and_drag: true,
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
            tap_and_drag,
        }
    }

    pub const fn tap_to_click(self) -> bool { self.tap_to_click }
    pub const fn two_finger_scroll(self) -> bool { self.two_finger_scroll }
    pub const fn natural_scrolling(self) -> bool { self.natural_scrolling }
    pub const fn disable_while_typing(self) -> bool { self.disable_while_typing }
    pub const fn tap_and_drag(self) -> bool { self.tap_and_drag }
}

/// Outcome of applying one setting to one physical touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchpadSettingStatus {
    Applied,
    Unsupported,
    Failed,
}

/// Per-setting result. Unsupported hardware is never reported as configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchpadApplyReport {
    pub tap_to_click: TouchpadSettingStatus,
    pub two_finger_scroll: TouchpadSettingStatus,
    pub natural_scrolling: TouchpadSettingStatus,
    pub disable_while_typing: TouchpadSettingStatus,
    pub tap_and_drag: TouchpadSettingStatus,
}

impl TouchpadApplyReport {
    pub const fn all_applied(self) -> bool {
        matches!(self.tap_to_click, TouchpadSettingStatus::Applied)
            && matches!(self.two_finger_scroll, TouchpadSettingStatus::Applied)
            && matches!(self.natural_scrolling, TouchpadSettingStatus::Applied)
            && matches!(self.disable_while_typing, TouchpadSettingStatus::Applied)
            && matches!(self.tap_and_drag, TouchpadSettingStatus::Applied)
    }

    /// Returns true when libinput rejected at least one supported setting.
    ///
    /// Unsupported settings are a hardware capability limitation, not an
    /// application failure, and therefore do not trigger repeated retries.
    pub const fn has_failures(self) -> bool {
        matches!(self.tap_to_click, TouchpadSettingStatus::Failed)
            || matches!(self.two_finger_scroll, TouchpadSettingStatus::Failed)
            || matches!(self.natural_scrolling, TouchpadSettingStatus::Failed)
            || matches!(self.disable_while_typing, TouchpadSettingStatus::Failed)
            || matches!(self.tap_and_drag, TouchpadSettingStatus::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::{TouchpadApplyReport, TouchpadSettingStatus, TouchpadSettings};

    #[test]
    fn defaults_enable_common_touchpad_behaviors() {
        let settings = TouchpadSettings::default();
        assert!(settings.tap_to_click());
        assert!(settings.two_finger_scroll());
        assert!(!settings.natural_scrolling());
        assert!(settings.disable_while_typing());
        assert!(settings.tap_and_drag());
    }

    #[test]
    fn unsupported_settings_are_not_application_failures() {
        let report = TouchpadApplyReport {
            tap_to_click: TouchpadSettingStatus::Applied,
            two_finger_scroll: TouchpadSettingStatus::Unsupported,
            natural_scrolling: TouchpadSettingStatus::Applied,
            disable_while_typing: TouchpadSettingStatus::Applied,
            tap_and_drag: TouchpadSettingStatus::Applied,
        };
        assert!(!report.has_failures());
    }

    #[test]
    fn failed_setting_is_detected_for_retry() {
        let report = TouchpadApplyReport {
            tap_to_click: TouchpadSettingStatus::Failed,
            two_finger_scroll: TouchpadSettingStatus::Unsupported,
            natural_scrolling: TouchpadSettingStatus::Applied,
            disable_while_typing: TouchpadSettingStatus::Applied,
            tap_and_drag: TouchpadSettingStatus::Applied,
        };
        assert!(report.has_failures());
    }

    #[test]
    fn unsupported_setting_prevents_full_success_report() {
        let report = TouchpadApplyReport {
            tap_to_click: TouchpadSettingStatus::Applied,
            two_finger_scroll: TouchpadSettingStatus::Unsupported,
            natural_scrolling: TouchpadSettingStatus::Applied,
            disable_while_typing: TouchpadSettingStatus::Applied,
            tap_and_drag: TouchpadSettingStatus::Applied,
        };
        assert!(!report.all_applied());
    }
}
