// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Backend-independent touchpad preferences and capability reporting.

/// Preferred software clickpad behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TouchpadClickMethod {
    /// Use the physical/software button areas exposed by the device.
    ButtonAreas,
    /// Infer left/right/middle click from the number of fingers.
    #[default]
    Clickfinger,
}

/// User-configurable touchpad behavior.
///
/// The five-argument constructor is kept stable for existing callers. More
/// advanced preferences are opt-in through the builder-style methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchpadSettings {
    tap_to_click: bool,
    two_finger_scroll: bool,
    natural_scrolling: bool,
    disable_while_typing: bool,
    tap_and_drag: bool,
    edge_scrolling: bool,
    click_method: TouchpadClickMethod,
    left_handed: bool,
    middle_button_emulation: bool,
    pointer_speed_percent: i8,
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
            tap_to_click: true,
            two_finger_scroll: true,
            natural_scrolling: false,
            disable_while_typing: true,
            tap_and_drag: true,
            edge_scrolling: false,
            click_method: TouchpadClickMethod::Clickfinger,
            left_handed: false,
            middle_button_emulation: false,
            pointer_speed_percent: 0,
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
            edge_scrolling: false,
            click_method: TouchpadClickMethod::Clickfinger,
            left_handed: false,
            middle_button_emulation: false,
            pointer_speed_percent: 0,
        }
    }

    pub const fn with_edge_scrolling(mut self, enabled: bool) -> Self {
        self.edge_scrolling = enabled;
        self
    }

    pub const fn with_click_method(mut self, method: TouchpadClickMethod) -> Self {
        self.click_method = method;
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

    /// Set pointer speed as a percentage in the inclusive range [-100, 100].
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

    pub const fn tap_to_click(self) -> bool { self.tap_to_click }
    pub const fn two_finger_scroll(self) -> bool { self.two_finger_scroll && !self.edge_scrolling }
    pub const fn natural_scrolling(self) -> bool { self.natural_scrolling }
    pub const fn disable_while_typing(self) -> bool { self.disable_while_typing }
    pub const fn tap_and_drag(self) -> bool { self.tap_and_drag }
    pub const fn edge_scrolling(self) -> bool { self.edge_scrolling }
    pub const fn click_method(self) -> TouchpadClickMethod { self.click_method }
    pub const fn left_handed(self) -> bool { self.left_handed }
    pub const fn middle_button_emulation(self) -> bool { self.middle_button_emulation }
    pub const fn pointer_speed_percent(self) -> i8 { self.pointer_speed_percent }
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
    pub edge_scrolling: TouchpadSettingStatus,
    pub natural_scrolling: TouchpadSettingStatus,
    pub disable_while_typing: TouchpadSettingStatus,
    pub tap_and_drag: TouchpadSettingStatus,
    pub click_method: TouchpadSettingStatus,
    pub left_handed: TouchpadSettingStatus,
    pub middle_button_emulation: TouchpadSettingStatus,
    pub pointer_speed: TouchpadSettingStatus,
}

impl TouchpadApplyReport {
    pub const fn all_applied(self) -> bool {
        matches!(self.tap_to_click, TouchpadSettingStatus::Applied)
            && matches!(self.two_finger_scroll, TouchpadSettingStatus::Applied)
            && matches!(self.edge_scrolling, TouchpadSettingStatus::Applied)
            && matches!(self.natural_scrolling, TouchpadSettingStatus::Applied)
            && matches!(self.disable_while_typing, TouchpadSettingStatus::Applied)
            && matches!(self.tap_and_drag, TouchpadSettingStatus::Applied)
            && matches!(self.click_method, TouchpadSettingStatus::Applied)
            && matches!(self.left_handed, TouchpadSettingStatus::Applied)
            && matches!(self.middle_button_emulation, TouchpadSettingStatus::Applied)
            && matches!(self.pointer_speed, TouchpadSettingStatus::Applied)
    }

    /// Returns true when libinput rejected at least one supported setting.
    /// Unsupported settings are hardware capability limitations, not failures.
    pub const fn has_failures(self) -> bool {
        matches!(self.tap_to_click, TouchpadSettingStatus::Failed)
            || matches!(self.two_finger_scroll, TouchpadSettingStatus::Failed)
            || matches!(self.edge_scrolling, TouchpadSettingStatus::Failed)
            || matches!(self.natural_scrolling, TouchpadSettingStatus::Failed)
            || matches!(self.disable_while_typing, TouchpadSettingStatus::Failed)
            || matches!(self.tap_and_drag, TouchpadSettingStatus::Failed)
            || matches!(self.click_method, TouchpadSettingStatus::Failed)
            || matches!(self.left_handed, TouchpadSettingStatus::Failed)
            || matches!(self.middle_button_emulation, TouchpadSettingStatus::Failed)
            || matches!(self.pointer_speed, TouchpadSettingStatus::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::{TouchpadApplyReport, TouchpadClickMethod, TouchpadSettingStatus, TouchpadSettings};

    #[test]
    fn defaults_enable_common_touchpad_behaviors() {
        let settings = TouchpadSettings::default();
        assert!(settings.tap_to_click());
        assert!(settings.two_finger_scroll());
        assert!(!settings.edge_scrolling());
        assert!(!settings.natural_scrolling());
        assert!(settings.disable_while_typing());
        assert!(settings.tap_and_drag());
        assert_eq!(settings.click_method(), TouchpadClickMethod::Clickfinger);
        assert_eq!(settings.pointer_speed_percent(), 0);
    }

    #[test]
    fn advanced_preferences_are_bounded_and_select_one_scroll_mode() {
        let settings = TouchpadSettings::default()
            .with_edge_scrolling(true)
            .with_left_handed(true)
            .with_middle_button_emulation(true)
            .with_pointer_speed_percent(120)
            .with_click_method(TouchpadClickMethod::ButtonAreas);
        assert!(settings.edge_scrolling());
        assert!(!settings.two_finger_scroll());
        assert!(settings.left_handed());
        assert!(settings.middle_button_emulation());
        assert_eq!(settings.pointer_speed_percent(), 100);
        assert_eq!(settings.click_method(), TouchpadClickMethod::ButtonAreas);
    }

    fn report(status: TouchpadSettingStatus) -> TouchpadApplyReport {
        TouchpadApplyReport {
            tap_to_click: status,
            two_finger_scroll: status,
            edge_scrolling: status,
            natural_scrolling: status,
            disable_while_typing: status,
            tap_and_drag: status,
            click_method: status,
            left_handed: status,
            middle_button_emulation: status,
            pointer_speed: status,
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
