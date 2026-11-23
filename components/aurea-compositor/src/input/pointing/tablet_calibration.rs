// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated, per-device absolute-coordinate calibration for pen tablets.

use std::collections::HashMap;

use aurea_desktop_api::input::InputDeviceId;

/// Clockwise rotation applied after selecting the active tablet area.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabletRotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

/// Whether the active tablet area is stretched or cropped to match the output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabletAspectMode {
    #[default]
    Stretch,
    Preserve,
}

/// Normalized active area and coordinate transform for one physical tablet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabletCalibration {
    active_area: [f64; 4],
    rotation: TabletRotation,
    aspect_mode: TabletAspectMode,
    device_aspect_ratio: f64,
}

impl Default for TabletCalibration {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl TabletCalibration {
    pub const IDENTITY: Self = Self {
        active_area: [0.0, 0.0, 1.0, 1.0],
        rotation: TabletRotation::Deg0,
        aspect_mode: TabletAspectMode::Stretch,
        device_aspect_ratio: 1.0,
    };

    /// Creates a profile after validating its normalized area and physical ratio.
    pub fn new(
        active_area: [f64; 4],
        rotation: TabletRotation,
        aspect_mode: TabletAspectMode,
        device_aspect_ratio: f64,
    ) -> Result<Self, TabletCalibrationError> {
        if !active_area.iter().all(|value| value.is_finite()) || !device_aspect_ratio.is_finite() {
            return Err(TabletCalibrationError::NonFiniteValue);
        }

        let [x, y, width, height] = active_area;
        if x < 0.0
            || y < 0.0
            || width <= 0.0
            || height <= 0.0
            || x + width > 1.0
            || y + height > 1.0
        {
            return Err(TabletCalibrationError::ActiveAreaOutOfBounds);
        }
        if device_aspect_ratio <= 0.0 {
            return Err(TabletCalibrationError::InvalidDeviceAspectRatio);
        }

        Ok(Self {
            active_area,
            rotation,
            aspect_mode,
            device_aspect_ratio,
        })
    }

    pub fn active_area(self) -> [f64; 4] {
        self.active_area
    }
    pub fn rotation(self) -> TabletRotation {
        self.rotation
    }
    pub fn aspect_mode(self) -> TabletAspectMode {
        self.aspect_mode
    }
    pub fn device_aspect_ratio(self) -> f64 {
        self.device_aspect_ratio
    }

    /// Converts normalized device coordinates into normalized output coordinates.
    pub fn apply(self, x: f64, y: f64, output_aspect_ratio: f64) -> Option<(f64, f64)> {
        if !x.is_finite()
            || !y.is_finite()
            || !output_aspect_ratio.is_finite()
            || output_aspect_ratio <= 0.0
        {
            return None;
        }

        let [mut area_x, mut area_y, mut area_width, mut area_height] = self.active_area;
        if self.aspect_mode == TabletAspectMode::Preserve {
            let source_ratio = self.device_aspect_ratio * area_width / area_height;
            let target_ratio = match self.rotation {
                TabletRotation::Deg90 | TabletRotation::Deg270 => 1.0 / output_aspect_ratio,
                TabletRotation::Deg0 | TabletRotation::Deg180 => output_aspect_ratio,
            };
            if source_ratio > target_ratio {
                let next_width = area_height * target_ratio / self.device_aspect_ratio;
                area_x += (area_width - next_width) / 2.0;
                area_width = next_width;
            } else {
                let next_height = area_width * self.device_aspect_ratio / target_ratio;
                area_y += (area_height - next_height) / 2.0;
                area_height = next_height;
            }
        }

        let u = ((x.clamp(0.0, 1.0) - area_x) / area_width).clamp(0.0, 1.0);
        let v = ((y.clamp(0.0, 1.0) - area_y) / area_height).clamp(0.0, 1.0);
        Some(match self.rotation {
            TabletRotation::Deg0 => (u, v),
            TabletRotation::Deg90 => (1.0 - v, u),
            TabletRotation::Deg180 => (1.0 - u, 1.0 - v),
            TabletRotation::Deg270 => (v, 1.0 - u),
        })
    }
}

/// Validation errors for tablet calibration profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabletCalibrationError {
    NonFiniteValue,
    ActiveAreaOutOfBounds,
    InvalidDeviceAspectRatio,
    UnknownDevice,
}

/// Stores independent calibration profiles keyed by physical tablet ID.
#[derive(Debug, Clone, Default)]
pub struct TabletCalibrationMap {
    profiles: HashMap<InputDeviceId, TabletCalibration>,
}

impl TabletCalibrationMap {
    pub fn set(&mut self, device: InputDeviceId, profile: TabletCalibration) {
        self.profiles.insert(device, profile);
    }

    pub fn get(&self, device: InputDeviceId) -> TabletCalibration {
        self.profiles.get(&device).copied().unwrap_or_default()
    }

    pub fn remove_device(&mut self, device: InputDeviceId) -> bool {
        self.profiles.remove(&device).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICE: InputDeviceId = InputDeviceId(17);

    #[test]
    fn identity_preserves_coordinates() {
        assert_eq!(
            TabletCalibration::IDENTITY.apply(0.2, 0.7, 16.0 / 9.0),
            Some((0.2, 0.7))
        );
    }

    #[test]
    fn active_area_is_scaled_to_full_output() {
        let profile = TabletCalibration::new(
            [0.25, 0.2, 0.5, 0.6],
            TabletRotation::Deg0,
            TabletAspectMode::Stretch,
            1.0,
        )
        .unwrap();
        assert_eq!(profile.apply(0.5, 0.5, 1.0), Some((0.5, 0.5)));
        assert_eq!(profile.apply(0.25, 0.2, 1.0), Some((0.0, 0.0)));
    }

    #[test]
    fn all_rotations_are_applied_clockwise() {
        let cases = [
            (TabletRotation::Deg0, (0.2, 0.3)),
            (TabletRotation::Deg90, (0.7, 0.2)),
            (TabletRotation::Deg180, (0.8, 0.7)),
            (TabletRotation::Deg270, (0.3, 0.8)),
        ];
        for (rotation, expected) in cases {
            let profile = TabletCalibration::new(
                [0.0, 0.0, 1.0, 1.0],
                rotation,
                TabletAspectMode::Stretch,
                1.0,
            )
            .unwrap();
            assert_eq!(profile.apply(0.2, 0.3, 1.0), Some(expected));
        }
    }

    #[test]
    fn preserve_mode_crops_the_longer_device_axis() {
        let profile = TabletCalibration::new(
            [0.0, 0.0, 1.0, 1.0],
            TabletRotation::Deg0,
            TabletAspectMode::Preserve,
            2.0,
        )
        .unwrap();

        // A 2:1 tablet mapped to a 1:1 output uses the centered middle half.
        assert_eq!(profile.apply(0.0, 0.5, 1.0), Some((0.0, 0.5)));
        assert_eq!(profile.apply(0.25, 0.5, 1.0), Some((0.0, 0.5)));
        assert_eq!(profile.apply(0.5, 0.5, 1.0), Some((0.5, 0.5)));
        assert_eq!(profile.apply(0.75, 0.5, 1.0), Some((1.0, 0.5)));
    }

    #[test]
    fn rejects_invalid_areas_and_ratios() {
        assert_eq!(
            TabletCalibration::new(
                [0.8, 0.0, 0.4, 1.0],
                TabletRotation::Deg0,
                TabletAspectMode::Stretch,
                1.0
            ),
            Err(TabletCalibrationError::ActiveAreaOutOfBounds)
        );
        assert_eq!(
            TabletCalibration::new(
                [0.0, 0.0, 1.0, 1.0],
                TabletRotation::Deg0,
                TabletAspectMode::Stretch,
                0.0
            ),
            Err(TabletCalibrationError::InvalidDeviceAspectRatio)
        );
        assert_eq!(
            TabletCalibration::new(
                [f64::NAN, 0.0, 1.0, 1.0],
                TabletRotation::Deg0,
                TabletAspectMode::Stretch,
                1.0
            ),
            Err(TabletCalibrationError::NonFiniteValue)
        );
    }

    #[test]
    fn profiles_are_independent_and_removed_with_device() {
        let mut map = TabletCalibrationMap::default();
        let profile = TabletCalibration::new(
            [0.1, 0.1, 0.8, 0.8],
            TabletRotation::Deg180,
            TabletAspectMode::Preserve,
            1.5,
        )
        .unwrap();
        map.set(DEVICE, profile);
        assert_eq!(map.get(DEVICE), profile);
        assert!(map.remove_device(DEVICE));
        assert_eq!(map.get(DEVICE), TabletCalibration::IDENTITY);
    }

    #[test]
    fn rejects_non_finite_runtime_coordinates() {
        assert_eq!(TabletCalibration::IDENTITY.apply(f64::NAN, 0.0, 1.0), None);
        assert_eq!(
            TabletCalibration::IDENTITY.apply(0.0, 0.0, f64::INFINITY),
            None
        );
    }
}
