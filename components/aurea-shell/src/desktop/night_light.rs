// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! User preferences and schedule primitives for the system night-light feature.

use aurea_desktop_api::display::DisplayColorAdjustment;

/// Supported ways to activate Night Light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NightLightMode {
    /// Apply the selected warmth whenever Night Light is enabled.
    #[default]
    Manual,
    /// Follow local sunset and sunrise once a location-aware scheduler exists.
    SunsetToSunrise,
    /// Follow the user-defined local clock interval.
    CustomSchedule,
}

/// A local-clock interval expressed as minutes after midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NightLightSchedule {
    start_minute: u16,
    end_minute: u16,
}

impl NightLightSchedule {
    /// Creates a schedule with valid, distinct local-clock times.
    pub fn new(start_minute: u16, end_minute: u16) -> Result<Self, NightLightScheduleError> {
        if start_minute >= 1_440 || end_minute >= 1_440 {
            return Err(NightLightScheduleError::TimeOutOfRange);
        }
        if start_minute == end_minute {
            return Err(NightLightScheduleError::EmptyInterval);
        }
        Ok(Self { start_minute, end_minute })
    }

    pub const fn start_minute(self) -> u16 { self.start_minute }
    pub const fn end_minute(self) -> u16 { self.end_minute }

    /// Start is inclusive and end is exclusive; intervals may cross midnight.
    pub const fn is_active_at(self, minute_of_day: u16) -> bool {
        if minute_of_day >= 1_440 {
            return false;
        }
        if self.start_minute < self.end_minute {
            minute_of_day >= self.start_minute && minute_of_day < self.end_minute
        } else {
            minute_of_day >= self.start_minute || minute_of_day < self.end_minute
        }
    }
}

/// Invalid user-defined local-clock schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NightLightScheduleError {
    TimeOutOfRange,
    EmptyInterval,
}

/// Persistent user preference state for Night Light.
///
/// Schedule evaluation and persistence belong to separate services; this type
/// validates and exposes preferences without pretending those services exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NightLightState {
    enabled: bool,
    strength_percent: u8,
    mode: NightLightMode,
    schedule: NightLightSchedule,
}

impl Default for NightLightState {
    fn default() -> Self {
        Self {
            enabled: false,
            strength_percent: 50,
            mode: NightLightMode::Manual,
            schedule: NightLightSchedule { start_minute: 1_200, end_minute: 360 },
        }
    }
}

impl NightLightState {
    pub const fn is_enabled(self) -> bool { self.enabled }
    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    pub const fn strength_percent(self) -> u8 { self.strength_percent }

    pub fn set_strength_percent(&mut self, strength_percent: u8) {
        self.strength_percent = if strength_percent > 100 { 100 } else { strength_percent };
    }

    pub const fn mode(self) -> NightLightMode { self.mode }
    pub fn set_mode(&mut self, mode: NightLightMode) { self.mode = mode; }
    pub const fn schedule(self) -> NightLightSchedule { self.schedule }
    pub fn set_schedule(&mut self, schedule: NightLightSchedule) { self.schedule = schedule; }

    /// Builds the compositor-facing adjustment from a scheduler decision.
    ///
    /// schedule_active must come from the system scheduling service for
    /// sunset/sunrise and custom modes. Manual mode does not require a clock.
    pub const fn display_adjustment(self, schedule_active: bool) -> DisplayColorAdjustment {
        let active = match self.mode {
            NightLightMode::Manual => true,
            NightLightMode::SunsetToSunrise | NightLightMode::CustomSchedule => schedule_active,
        };
        if self.enabled && active {
            DisplayColorAdjustment::new(self.strength_percent)
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
        assert_eq!(NightLightSchedule::new(1_440, 60), Err(NightLightScheduleError::TimeOutOfRange));
        assert_eq!(NightLightSchedule::new(600, 600), Err(NightLightScheduleError::EmptyInterval));
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
    fn strength_is_clamped() {
        let mut state = NightLightState::default();
        state.set_strength_percent(u8::MAX);
        assert_eq!(state.strength_percent(), 100);
    }
}
