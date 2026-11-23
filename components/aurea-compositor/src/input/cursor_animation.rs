// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Monotonic-time cursor locate animation state, independent from rendering.

use std::time::{Duration, Instant};

/// Cursor locate animation parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorAnimationConfig {
    pub peak_scale: f64,
    pub ramp_up_ms: u32,
    pub hold_ms: u32,
    pub ramp_down_ms: u32,
}

impl Default for CursorAnimationConfig {
    fn default() -> Self {
        Self {
            peak_scale: 2.4,
            ramp_up_ms: 110,
            hold_ms: 650,
            ramp_down_ms: 220,
        }
    }
}

/// Deterministic animation driven by the monotonic system clock.
#[derive(Debug, Clone, Copy)]
pub struct CursorLocateAnimation {
    config: CursorAnimationConfig,
    started_at: Option<Instant>,
}

impl Default for CursorLocateAnimation {
    fn default() -> Self {
        Self {
            config: CursorAnimationConfig::default(),
            started_at: None,
        }
    }
}

impl CursorLocateAnimation {
    /// Starts or restarts the locate effect.
    pub fn trigger(&mut self) {
        self.started_at = Some(Instant::now());
    }

    /// Returns the current scale. A value of 1.0 means normal size.
    pub fn scale_at(&self, now: Instant) -> f64 {
        let Some(started) = self.started_at else {
            return 1.0;
        };
        let elapsed = now.saturating_duration_since(started).as_millis() as u64;
        let ramp_up = u64::from(self.config.ramp_up_ms);
        let hold_end = ramp_up.saturating_add(u64::from(self.config.hold_ms));
        let end = hold_end.saturating_add(u64::from(self.config.ramp_down_ms));

        if elapsed >= end {
            return 1.0;
        }
        if elapsed < ramp_up {
            let t = elapsed as f64 / ramp_up.max(1) as f64;
            return 1.0 + (self.config.peak_scale - 1.0) * smooth_step(t);
        }
        if elapsed < hold_end {
            return self.config.peak_scale;
        }

        let t = (elapsed - hold_end) as f64 / u64::from(self.config.ramp_down_ms).max(1) as f64;
        1.0 + (self.config.peak_scale - 1.0) * (1.0 - smooth_step(t))
    }

    /// Reports whether another animation frame is still useful.
    pub fn is_active(&self, now: Instant) -> bool {
        let Some(started) = self.started_at else {
            return false;
        };
        now.saturating_duration_since(started)
            < Duration::from_millis(
                u64::from(self.config.ramp_up_ms)
                    + u64::from(self.config.hold_ms)
                    + u64::from(self.config.ramp_down_ms),
            )
    }
}

fn smooth_step(value: f64) -> f64 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_grows_holds_and_returns_to_normal() {
        let mut animation = CursorLocateAnimation::default();
        let start = Instant::now();
        animation.started_at = Some(start);

        assert_eq!(animation.scale_at(start), 1.0);
        assert!(animation.scale_at(start + Duration::from_millis(110)) > 2.39);
        assert_eq!(animation.scale_at(start + Duration::from_millis(400)), 2.4);
        assert!(animation.scale_at(start + Duration::from_millis(900)) < 2.4);
        assert_eq!(animation.scale_at(start + Duration::from_millis(1000)), 1.0);
        assert!(!animation.is_active(start + Duration::from_millis(1000)));
    }

    #[test]
    fn retrigger_restarts_animation() {
        let mut animation = CursorLocateAnimation::default();
        let start = Instant::now();
        animation.started_at = Some(start);

        let restarted = start + Duration::from_millis(50);
        animation.started_at = Some(restarted);

        assert_eq!(animation.scale_at(restarted), 1.0);
    }
}
