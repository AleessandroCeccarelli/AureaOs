// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Per-device affine calibration for absolute touchscreen coordinates.

use std::collections::HashMap;

use aurea_desktop_api::input::InputDeviceId;

/// Affine mapping from normalized device coordinates to normalized panel coordinates.
///
/// The coefficients represent:
/// x' = a*x + b*y + c
/// y' = d*x + e*y + f
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchscreenCalibration {
    coefficients: [f64; 6],
}

impl Default for TouchscreenCalibration {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl TouchscreenCalibration {
    /// Identity calibration leaves coordinates unchanged.
    pub const IDENTITY: Self = Self {
        coefficients: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    };

    /// Builds a calibration after validating its coefficients and invertibility.
    pub fn new(coefficients: [f64; 6]) -> Result<Self, TouchscreenCalibrationError> {
        if !coefficients.iter().all(|value| value.is_finite()) {
            return Err(TouchscreenCalibrationError::NonFiniteCoefficient);
        }

        let determinant = coefficients[0] * coefficients[4] - coefficients[1] * coefficients[3];
        if !determinant.is_finite() || determinant.abs() < 1.0e-8 {
            return Err(TouchscreenCalibrationError::SingularTransform);
        }

        Ok(Self { coefficients })
    }

    /// Returns the six affine coefficients in row-major transform order.
    pub fn coefficients(self) -> [f64; 6] {
        self.coefficients
    }

    /// Maps a normalized point and clamps the result to the panel bounds.
    pub fn apply(self, x: f64, y: f64) -> Option<(f64, f64)> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }

        let [a, b, c, d, e, f] = self.coefficients;
        let mapped_x = a * x + b * y + c;
        let mapped_y = d * x + e * y + f;
        if !mapped_x.is_finite() || !mapped_y.is_finite() {
            return None;
        }

        Some((mapped_x.clamp(0.0, 1.0), mapped_y.clamp(0.0, 1.0)))
    }
}

/// Validation failures for touchscreen calibration profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchscreenCalibrationError {
    NonFiniteCoefficient,
    SingularTransform,
    UnknownDevice,
}

/// Keeps calibration profiles isolated by physical touchscreen.
#[derive(Debug, Clone, Default)]
pub struct TouchscreenCalibrationMap {
    profiles: HashMap<InputDeviceId, TouchscreenCalibration>,
}

impl TouchscreenCalibrationMap {
    /// Stores a validated profile for one device.
    pub fn set(&mut self, device: InputDeviceId, calibration: TouchscreenCalibration) {
        self.profiles.insert(device, calibration);
    }

    /// Returns the configured profile, or identity when none is configured.
    pub fn get(&self, device: InputDeviceId) -> TouchscreenCalibration {
        self.profiles
            .get(&device)
            .copied()
            .unwrap_or(TouchscreenCalibration::IDENTITY)
    }

    /// Removes a disconnected device's profile.
    pub fn remove_device(&mut self, device: InputDeviceId) -> bool {
        self.profiles.remove(&device).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: InputDeviceId = InputDeviceId(11);
    const SECOND: InputDeviceId = InputDeviceId(22);

    #[test]
    fn identity_profile_preserves_normalized_coordinates() {
        assert_eq!(
            TouchscreenCalibration::IDENTITY.apply(0.25, 0.75),
            Some((0.25, 0.75))
        );
    }

    #[test]
    fn affine_profile_applies_scale_and_offset() {
        let calibration = TouchscreenCalibration::new([0.5, 0.0, 0.1, 0.0, 0.5, 0.2]).unwrap();

        let (mapped_x, mapped_y) = calibration.apply(0.4, 0.6).unwrap();
        assert!((mapped_x - 0.3).abs() < 1.0e-12);
        assert!((mapped_y - 0.5).abs() < 1.0e-12);
    }

    #[test]
    fn affine_profile_can_correct_axis_inversion() {
        let calibration = TouchscreenCalibration::new([-1.0, 0.0, 1.0, 0.0, 1.0, 0.0]).unwrap();

        assert_eq!(calibration.apply(0.2, 0.7), Some((0.8, 0.7)));
    }

    #[test]
    fn mapped_coordinates_are_clamped_to_panel_bounds() {
        let calibration = TouchscreenCalibration::new([1.0, 0.0, 0.4, 0.0, 1.0, -0.3]).unwrap();

        assert_eq!(calibration.apply(0.9, 0.1), Some((1.0, 0.0)));
    }

    #[test]
    fn rejects_non_finite_coefficients() {
        assert_eq!(
            TouchscreenCalibration::new([1.0, 0.0, f64::NAN, 0.0, 1.0, 0.0]),
            Err(TouchscreenCalibrationError::NonFiniteCoefficient)
        );
    }

    #[test]
    fn rejects_singular_transform() {
        assert_eq!(
            TouchscreenCalibration::new([1.0, 2.0, 0.0, 2.0, 4.0, 0.0]),
            Err(TouchscreenCalibrationError::SingularTransform)
        );
    }

    #[test]
    fn rejects_non_finite_input_and_overflow() {
        let identity = TouchscreenCalibration::IDENTITY;
        assert_eq!(identity.apply(f64::INFINITY, 0.0), None);

        let large = TouchscreenCalibration::new([1.0e308, 0.0, 0.0, 0.0, 1.0, 0.0]).unwrap();
        assert_eq!(large.apply(2.0, 0.5), None);
    }

    #[test]
    fn profiles_are_independent_and_default_to_identity() {
        let mut profiles = TouchscreenCalibrationMap::default();
        let offset = TouchscreenCalibration::new([1.0, 0.0, 0.1, 0.0, 1.0, 0.0]).unwrap();

        profiles.set(FIRST, offset);
        assert_eq!(profiles.get(FIRST), offset);
        assert_eq!(profiles.get(SECOND), TouchscreenCalibration::IDENTITY);
    }

    #[test]
    fn disconnect_removes_only_the_matching_profile() {
        let mut profiles = TouchscreenCalibrationMap::default();
        profiles.set(FIRST, TouchscreenCalibration::IDENTITY);
        profiles.set(
            SECOND,
            TouchscreenCalibration::new([1.0, 0.0, 0.1, 0.0, 1.0, 0.0]).unwrap(),
        );

        assert!(profiles.remove_device(FIRST));
        assert!(!profiles.remove_device(FIRST));
        assert_eq!(
            profiles.get(SECOND).coefficients(),
            [1.0, 0.0, 0.1, 0.0, 1.0, 0.0]
        );
    }
}
