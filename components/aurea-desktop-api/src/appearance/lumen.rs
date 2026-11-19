// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Aurea Lumen: system-wide environmental display adaptation preferences.
//!
//! This module defines technology-independent policy contracts only. Sensor
//! discovery, solar-time calculation, scheduling, persistence and compositor
//! application belong to separate providers/services.

/// How Aurea Lumen decides when to apply its warm display adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LumenMode {
    /// User explicitly enables or disables the effect.
    Manual,
    /// Follow a validated local-clock interval.
    Schedule,
    /// Follow sunrise and sunset from a trusted location/time service.
    Solar,
    /// Follow a supported ambient-light sensor.
    Ambient,
    /// Prefer ambient sensing, then use an available solar or local schedule.
    #[default]
    Automatic,
}

/// Local-clock interval expressed as minutes after midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LumenSchedule {
    start_minute: u16,
    end_minute: u16,
}

impl LumenSchedule {
    pub fn new(start_minute: u16, end_minute: u16) -> Result<Self, LumenScheduleError> {
        if start_minute >= 1_440 || end_minute >= 1_440 {
            return Err(LumenScheduleError::TimeOutOfRange);
        }
        if start_minute == end_minute {
            return Err(LumenScheduleError::EmptyInterval);
        }
        Ok(Self { start_minute, end_minute })
    }

    pub const fn start_minute(self) -> u16 { self.start_minute }
    pub const fn end_minute(self) -> u16 { self.end_minute }

    /// Start is inclusive and end exclusive; intervals may cross midnight.
    pub const fn is_active_at(self, minute_of_day: u16) -> bool {
        if minute_of_day >= 1_440 { return false; }
        if self.start_minute < self.end_minute {
            minute_of_day >= self.start_minute && minute_of_day < self.end_minute
        } else {
            minute_of_day >= self.start_minute || minute_of_day < self.end_minute
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LumenScheduleError {
    TimeOutOfRange,
    EmptyInterval,
}

/// Lux thresholds with a dead band to prevent rapid switching near a boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmbientLightThresholds {
    darker_than_lux: u32,
    brighter_than_lux: u32,
}

impl AmbientLightThresholds {
    pub const fn new(darker_than_lux: u32, brighter_than_lux: u32) -> Option<Self> {
        if brighter_than_lux <= darker_than_lux { return None; }
        Some(Self { darker_than_lux, brighter_than_lux })
    }

    pub const fn darker_than_lux(self) -> u32 { self.darker_than_lux }
    pub const fn brighter_than_lux(self) -> u32 { self.brighter_than_lux }
}

/// User preferences for Aurea Lumen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LumenPreferences {
    enabled: bool,
    mode: LumenMode,
    /// Warmth strength in the inclusive range 0–100.
    warmth_percent: u8,
    schedule: LumenSchedule,
    ambient_thresholds: AmbientLightThresholds,
}

impl Default for LumenPreferences {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: LumenMode::Automatic,
            warmth_percent: 50,
            schedule: LumenSchedule { start_minute: 1_200, end_minute: 360 },
            ambient_thresholds: AmbientLightThresholds {
                darker_than_lux: 80,
                brighter_than_lux: 160,
            },
        }
    }
}

impl LumenPreferences {
    pub const fn is_enabled(self) -> bool { self.enabled }

    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }

    pub const fn mode(self) -> LumenMode { self.mode }

    pub fn set_mode(&mut self, mode: LumenMode) { self.mode = mode; }

    pub const fn warmth_percent(self) -> u8 { self.warmth_percent }

    pub fn set_warmth_percent(&mut self, value: u8) {
        self.warmth_percent = if value > 100 { 100 } else { value };
    }

    pub const fn schedule(self) -> LumenSchedule { self.schedule }

    pub fn set_schedule(&mut self, schedule: LumenSchedule) { self.schedule = schedule; }

    pub const fn ambient_thresholds(self) -> AmbientLightThresholds { self.ambient_thresholds }

    pub fn set_ambient_thresholds(&mut self, thresholds: AmbientLightThresholds) {
        self.ambient_thresholds = thresholds;
    }
}

#[cfg(test)]
mod tests {
    use super::{AmbientLightThresholds, LumenMode, LumenSchedule, LumenScheduleError, LumenPreferences};

    #[test]
    fn automatic_is_the_default_mode_and_lumen_starts_disabled() {
        let preferences = LumenPreferences::default();
        assert_eq!(preferences.mode(), LumenMode::Automatic);
        assert!(!preferences.is_enabled());
    }

    #[test]
    fn schedule_supports_intervals_crossing_midnight() {
        let schedule = LumenSchedule::new(1_200, 360).unwrap();
        assert!(schedule.is_active_at(1_200));
        assert!(schedule.is_active_at(90));
        assert!(!schedule.is_active_at(360));
        assert!(!schedule.is_active_at(1_440));
    }

    #[test]
    fn schedule_rejects_invalid_and_empty_intervals() {
        assert_eq!(LumenSchedule::new(1_440, 60), Err(LumenScheduleError::TimeOutOfRange));
        assert_eq!(LumenSchedule::new(600, 600), Err(LumenScheduleError::EmptyInterval));
    }

    #[test]
    fn ambient_thresholds_require_a_hysteresis_band() {
        assert!(AmbientLightThresholds::new(80, 160).is_some());
        assert!(AmbientLightThresholds::new(160, 80).is_none());
        assert!(AmbientLightThresholds::new(80, 80).is_none());
    }

    #[test]
    fn preferences_are_changed_through_validated_accessors() {
        let mut preferences = LumenPreferences::default();
        preferences.set_enabled(true);
        preferences.set_mode(LumenMode::Schedule);
        preferences.set_schedule(LumenSchedule::new(300, 600).unwrap());
        preferences.set_ambient_thresholds(AmbientLightThresholds::new(40, 120).unwrap());
        assert!(preferences.is_enabled());
        assert_eq!(preferences.mode(), LumenMode::Schedule);
        assert_eq!(preferences.schedule().start_minute(), 300);
        assert_eq!(preferences.ambient_thresholds().brighter_than_lux(), 120);
    }

    #[test]
    fn warmth_is_clamped() {
        let mut preferences = LumenPreferences::default();
        preferences.set_warmth_percent(u8::MAX);
        assert_eq!(preferences.warmth_percent(), 100);
    }
}
