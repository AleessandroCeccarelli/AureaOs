// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shell-facing compatibility adapter for Aurea Lumen preferences.

use aurea_desktop_api::{
    appearance::lumen::{LumenMode, LumenPreferences, LumenSchedule},
    display::DisplayColorAdjustment,
};

pub use aurea_desktop_api::appearance::lumen::{
    LumenSchedule as NightLightSchedule,
    LumenScheduleError as NightLightScheduleError,
};

/// Legacy shell mode names retained while settings and scheduling migrate to Lumen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NightLightMode {
    /// Apply the selected warmth whenever the feature is enabled.
    #[default]
    Manual,
    /// Follow sunrise and sunset once a location-aware provider exists.
    SunsetToSunrise,
    /// Follow the user-defined local-clock interval.
    CustomSchedule,
}

/// Shell adapter over the shared Desktop API preferences.
///
/// The API owns validation and preference semantics. The shell keeps this
/// adapter temporarily to avoid coupling the current UI/runtime migration to
/// the first Lumen policy-provider implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NightLightState {
    preferences: LumenPreferences,
    mode: NightLightMode,
}

impl Default for NightLightState {
    fn default() -> Self {
        let mut preferences = LumenPreferences::default();
        preferences.set_mode(LumenMode::Manual);
        Self {
            preferences,
            mode: NightLightMode::Manual,
        }
    }
}

impl NightLightState {
    pub const fn is_enabled(self) -> bool {
        self.preferences.is_enabled()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.preferences.set_enabled(enabled);
    }

    pub const fn strength_percent(self) -> u8 {
        self.preferences.warmth_percent()
    }

    pub fn set_strength_percent(&mut self, strength_percent: u8) {
        self.preferences.set_warmth_percent(strength_percent);
    }

    pub const fn mode(self) -> NightLightMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: NightLightMode) {
        self.mode = mode;
        let api_mode = match mode {
            NightLightMode::Manual => LumenMode::Manual,
            NightLightMode::SunsetToSunrise => LumenMode::Solar,
            NightLightMode::CustomSchedule => LumenMode::Schedule,
        };
        self.preferences.set_mode(api_mode);
    }

    pub const fn schedule(self) -> NightLightSchedule {
        self.preferences.schedule()
    }

    pub fn set_schedule(&mut self, schedule: NightLightSchedule) {
        self.preferences.set_schedule(schedule);
    }

    /// Exposes a copy of the shared API preferences for the next migration step.
    pub const fn lumen_preferences(self) -> LumenPreferences {
        self.preferences
    }

    /// Builds the compositor-facing adjustment from an external policy decision.
    ///
    /// Until the Lumen policy coordinator exists, schedule_active is supplied
    /// by the existing shell scheduler. Manual mode does not require a clock.
    pub fn display_adjustment(self, schedule_active: bool) -> DisplayColorAdjustment {
        let active = match self.mode {
            NightLightMode::Manual => true,
            NightLightMode::SunsetToSunrise | NightLightMode::CustomSchedule => schedule_active,
        };

        if self.is_enabled() && active {
            DisplayColorAdjustment::new(self.strength_percent())
        } else {
            DisplayColorAdjustment::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NightLightMode, NightLightSchedule, NightLightScheduleError, NightLightState};

    #[test]
    fn schedule_rejects_invalid_and_empty_intervals() {
        assert_eq!(
            NightLightSchedule::new(1_440, 60),
            Err(NightLightScheduleError::TimeOutOfRange)
        );
        assert_eq!(
            NightLightSchedule::new(600, 600),
            Err(NightLightScheduleError::EmptyInterval)
        );
    }

    #[test]
    fn schedule_supports_intervals_crossing_midnight() {
        let schedule = NightLightSchedule::new(1_200, 360).unwrap();
        assert!(schedule.is_active_at(1_200));
        assert!(schedule.is_active_at(90));
        assert!(!schedule.is_active_at(360));
        assert!(!schedule.is_active_at(800));
        assert!(!schedule.is_active_at(1_440));
    }

    #[test]
    fn disabled_night_light_never_changes_the_output() {
        let mut state = NightLightState::default();
        state.set_strength_percent(80);
        assert!(state.display_adjustment(true).is_neutral());
    }

    #[test]
    fn manual_mode_applies_selected_strength() {
        let mut state = NightLightState::default();
        state.set_enabled(true);
        state.set_strength_percent(72);
        assert_eq!(state.display_adjustment(false).warmth_percent(), 72);
    }

    #[test]
    fn scheduled_modes_respect_scheduler_decision() {
        let mut state = NightLightState::default();
        state.set_enabled(true);
        state.set_mode(NightLightMode::CustomSchedule);
        assert!(state.display_adjustment(false).is_neutral());
        assert_eq!(state.display_adjustment(true).warmth_percent(), 50);
        state.set_mode(NightLightMode::SunsetToSunrise);
        assert!(state.display_adjustment(false).is_neutral());
    }

    #[test]
    fn strength_is_clamped_by_the_shared_api() {
        let mut state = NightLightState::default();
        state.set_strength_percent(u8::MAX);
        assert_eq!(state.strength_percent(), 100);
    }

    #[test]
    fn shell_adapter_uses_the_shared_lumen_schedule_and_preferences() {
        let mut state = NightLightState::default();
        let schedule = NightLightSchedule::new(300, 600).unwrap();
        state.set_schedule(schedule);
        state.set_enabled(true);

        assert_eq!(state.schedule(), schedule);
        assert!(state.lumen_preferences().is_enabled());
    }
}
