// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared pointer preferences and semantic pointer controls.

/// Acceleration profile exposed independently from the input backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerAccelerationProfile {
    /// Speed-sensitive acceleration, recommended for general desktop use.
    Adaptive,
    /// Constant device-specific acceleration factor.
    Flat,
}

/// User-configurable pointer preferences shared by Settings and the compositor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerSettings {
    profile: PointerAccelerationProfile,
    speed: f64,
    shake_to_locate: bool,
}

impl Default for PointerSettings {
    fn default() -> Self {
        Self {
            profile: PointerAccelerationProfile::Adaptive,
            speed: 0.0,
            shake_to_locate: true,
        }
    }
}

impl PointerSettings {
    /// Creates validated pointer preferences.
    pub fn new(
        profile: PointerAccelerationProfile,
        speed: f64,
        shake_to_locate: bool,
    ) -> Option<Self> {
        if !speed.is_finite() || !(-1.0..=1.0).contains(&speed) {
            return None;
        }
        Some(Self {
            profile,
            speed,
            shake_to_locate,
        })
    }

    pub fn profile(self) -> PointerAccelerationProfile {
        self.profile
    }

    pub fn speed(self) -> f64 {
        self.speed
    }

    pub fn shake_to_locate(self) -> bool {
        self.shake_to_locate
    }
}

/// Stable semantic pointer button, independent from Linux evdev button codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[cfg(test)]
mod tests {
    use super::{PointerAccelerationProfile, PointerSettings};

    #[test]
    fn defaults_to_adaptive_with_neutral_speed() {
        let settings = PointerSettings::default();
        assert_eq!(settings.profile(), PointerAccelerationProfile::Adaptive);
        assert_eq!(settings.speed(), 0.0);
        assert!(settings.shake_to_locate());
    }

    #[test]
    fn rejects_invalid_speed_values() {
        assert!(PointerSettings::new(PointerAccelerationProfile::Flat, 1.1, true).is_none());
        assert!(PointerSettings::new(PointerAccelerationProfile::Flat, f64::NAN, true).is_none());
    }

    #[test]
    fn accepts_speed_boundaries() {
        assert!(PointerSettings::new(PointerAccelerationProfile::Flat, -1.0, false).is_some());
        assert!(PointerSettings::new(PointerAccelerationProfile::Flat, 1.0, false).is_some());
    }
}
