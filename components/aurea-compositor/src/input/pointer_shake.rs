// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Detects deliberate alternating pointer motion to locate the cursor.

use std::collections::HashMap;

use aurea_desktop_api::input::InputDeviceId;
use smithay::utils::{Logical, Point};

const WINDOW_MS: u32 = 650;
const MIN_REVERSALS: u8 = 3;
const MIN_DISTANCE: f64 = 180.0;
const MAX_GAP_MS: u32 = 180;
const MIN_AXIS_DELTA: f64 = 12.0;

#[derive(Debug, Clone, Copy)]
struct MotionSample {
    time_ms: u32,
    dx: f64,
    dy: f64,
}

/// Stateful shake detector. Feed it unaccelerated device deltas.
#[derive(Debug, Default)]
pub struct PointerShakeDetector {
    samples: Vec<MotionSample>,
}

impl PointerShakeDetector {
    /// Returns true once when a sufficiently strong alternating gesture is seen.
    pub fn update(&mut self, delta: Point<f64, Logical>, time_ms: u32) -> bool {
        if !delta.x.is_finite() || !delta.y.is_finite() {
            self.reset();
            return false;
        }

        if delta.x.abs() + delta.y.abs() < MIN_AXIS_DELTA {
            self.prune(time_ms);
            return false;
        }

        self.samples.push(MotionSample {
            time_ms,
            dx: delta.x,
            dy: delta.y,
        });
        self.prune(time_ms);

        let horizontal_activity = self
            .samples
            .iter()
            .map(|sample| sample.dx.abs())
            .sum::<f64>();
        let vertical_activity = self
            .samples
            .iter()
            .map(|sample| sample.dy.abs())
            .sum::<f64>();
        let use_horizontal_axis = horizontal_activity >= vertical_activity;

        let mut reversals = 0_u8;
        let mut distance = 0.0;
        let mut previous_sign = 0_i8;
        let mut previous_time = None;

        for sample in &self.samples {
            let axis_delta = if use_horizontal_axis {
                sample.dx
            } else {
                sample.dy
            };

            if let Some(last_time) = previous_time {
                if sample.time_ms.wrapping_sub(last_time) > MAX_GAP_MS {
                    reversals = 0;
                    distance = 0.0;
                    previous_sign = 0;
                }
            }

            if axis_delta.abs() >= MIN_AXIS_DELTA {
                let sign = if axis_delta > 0.0 { 1 } else { -1 };
                if previous_sign != 0 && sign != previous_sign {
                    reversals = reversals.saturating_add(1);
                }
                previous_sign = sign;
            }

            previous_time = Some(sample.time_ms);
            distance += sample.dx.hypot(sample.dy);
        }

        if reversals >= MIN_REVERSALS && distance >= MIN_DISTANCE {
            self.reset();
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }

    fn prune(&mut self, now_ms: u32) {
        self.samples
            .retain(|sample| now_ms.wrapping_sub(sample.time_ms) <= WINDOW_MS);
    }
}

/// Owns independent shake histories for every connected physical pointer.
#[derive(Debug, Default)]
pub struct PointerShakeManager {
    detectors: HashMap<InputDeviceId, PointerShakeDetector>,
}

impl PointerShakeManager {
    /// Updates one device's history and reports a completed shake.
    pub fn update(
        &mut self,
        device: InputDeviceId,
        delta: Point<f64, Logical>,
        time_ms: u32,
    ) -> bool {
        self.detectors
            .entry(device)
            .or_default()
            .update(delta, time_ms)
    }

    /// Drops motion history when a device disconnects or disables the feature.
    pub fn remove(&mut self, device: InputDeviceId) -> bool {
        self.detectors.remove(&device).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(x: f64, y: f64) -> Point<f64, Logical> {
        Point::from((x, y))
    }

    #[test]
    fn detects_deliberate_alternating_shake_once() {
        let mut detector = PointerShakeDetector::default();
        let mut detected = false;
        for (time, x) in [(0, 70.0), (60, -70.0), (120, 70.0), (180, -70.0)] {
            detected |= detector.update(delta(x, 0.0), time);
        }
        assert!(detected);
        assert!(!detector.update(delta(1.0, 0.0), 190));
    }

    #[test]
    fn does_not_detect_one_direction_motion() {
        let mut detector = PointerShakeDetector::default();
        for (time, x) in [(0, 60.0), (50, 60.0), (100, 60.0), (150, 60.0)] {
            assert!(!detector.update(delta(x, 0.0), time));
        }
    }

    #[test]
    fn does_not_treat_axis_changes_as_a_shake() {
        let mut detector = PointerShakeDetector::default();
        let motions = [
            (0, 70.0, 0.0),
            (60, 0.0, -70.0),
            (120, -70.0, 0.0),
            (180, 0.0, 70.0),
            (240, 70.0, 0.0),
            (300, 0.0, -70.0),
        ];

        for (time, x, y) in motions {
            assert!(!detector.update(delta(x, y), time));
        }
    }

    #[test]
    fn detects_shake_across_millisecond_counter_wraparound() {
        let mut detector = PointerShakeDetector::default();
        let start = u32::MAX - 100;

        for (offset, x) in [(0, 70.0), (60, -70.0), (120, 70.0), (180, -70.0)] {
            assert_eq!(
                detector.update(delta(x, 0.0), start.wrapping_add(offset)),
                offset == 180
            );
        }
    }

    #[test]
    fn interleaved_devices_cannot_combine_motion_into_a_false_shake() {
        let mut manager = PointerShakeManager::default();
        let first = InputDeviceId(1);
        let second = InputDeviceId(2);

        for (time, first_delta, second_delta) in [
            (0, 70.0, -70.0),
            (60, 70.0, -70.0),
            (120, 70.0, -70.0),
            (180, 70.0, -70.0),
        ] {
            assert!(!manager.update(first, delta(first_delta, 0.0), time));
            assert!(!manager.update(second, delta(second_delta, 0.0), time));
        }
    }

    #[test]
    fn removing_a_device_drops_only_its_shake_history() {
        let mut manager = PointerShakeManager::default();
        let first = InputDeviceId(3);
        let second = InputDeviceId(4);

        assert!(!manager.update(first, delta(70.0, 0.0), 0));
        assert!(!manager.update(second, delta(-70.0, 0.0), 0));
        assert!(manager.remove(first));
        assert!(!manager.remove(first));
        assert!(!manager.update(second, delta(-70.0, 0.0), 60));
    }

    #[test]
    fn ignores_non_finite_motion() {
        let mut detector = PointerShakeDetector::default();
        assert!(!detector.update(delta(f64::NAN, 0.0), 0));
        assert!(!detector.update(delta(70.0, 0.0), 10));
    }
}
