// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Detects deliberate alternating pointer motion to locate the cursor.

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

        let mut reversals = 0_u8;
        let mut distance = 0.0;
        let mut previous_sign = 0_i8;
        let mut previous_time = None;

        for sample in &self.samples {
            let axis = if sample.dx.abs() >= sample.dy.abs() {
                sample.dx
            } else {
                sample.dy
            };
            let sign = if axis > 0.0 { 1 } else { -1 };
            if let Some(last_time) = previous_time {
                if sample.time_ms.saturating_sub(last_time) > MAX_GAP_MS {
                    reversals = 0;
                    distance = 0.0;
                    previous_sign = 0;
                }
            }
            if previous_sign != 0 && sign != previous_sign {
                reversals = reversals.saturating_add(1);
            }
            previous_sign = sign;
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
            .retain(|sample| now_ms.saturating_sub(sample.time_ms) <= WINDOW_MS);
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
    fn ignores_non_finite_motion() {
        let mut detector = PointerShakeDetector::default();
        assert!(!detector.update(delta(f64::NAN, 0.0), 0));
        assert!(!detector.update(delta(70.0, 0.0), 10));
    }
}
