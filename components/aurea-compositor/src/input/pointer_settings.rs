// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated pointer preferences, independent from the input backend.

/// Acceleration profile delegated to the device input backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelerationProfile {
    /// Speed-sensitive acceleration, recommended for general desktop use.
    Adaptive,
    /// Constant device-specific acceleration factor.
    Flat,
}

/// User-configurable pointer preferences.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerSettings {
    profile: AccelerationProfile,
    speed: f64,
    shake_to_locate: bool,
}

impl Default for PointerSettings {
    fn default() -> Self {
        Self {
            profile: AccelerationProfile::Adaptive,
            speed: 0.0,
            shake_to_locate: true,
        }
    }
}

impl PointerSettings {
    pub fn new(profile: AccelerationProfile, speed: f64, shake_to_locate: bool) -> Option<Self> {
        if !speed.is_finite() || !(-1.0..=1.0).contains(&speed) {
            return None;
        }
        Some(Self {
            profile,
            speed,
            shake_to_locate,
        })
    }

    pub fn profile(self) -> AccelerationProfile {
        self.profile
    }

    pub fn speed(self) -> f64 {
        self.speed
    }

    pub fn shake_to_locate(self) -> bool {
        self.shake_to_locate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_adaptive_with_neutral_speed() {
        let settings = PointerSettings::default();
        assert_eq!(settings.profile(), AccelerationProfile::Adaptive);
        assert_eq!(settings.speed(), 0.0);
        assert!(settings.shake_to_locate());
    }

    #[test]
    fn rejects_invalid_speed_values() {
        assert!(PointerSettings::new(AccelerationProfile::Flat, 1.1, true).is_none());
        assert!(PointerSettings::new(AccelerationProfile::Flat, f64::NAN, true).is_none());
    }

    #[test]
    fn accepts_speed_boundaries() {
        assert!(PointerSettings::new(AccelerationProfile::Flat, -1.0, false).is_some());
        assert!(PointerSettings::new(AccelerationProfile::Flat, 1.0, false).is_some());
    }
}
