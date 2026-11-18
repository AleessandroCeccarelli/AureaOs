// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Time-based cursor locate animation state, independent from rendering.

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
        Self { peak_scale: 2.4, ramp_up_ms: 110, hold_ms: 650, ramp_down_ms: 220 }
    }
}

/// Deterministic animation driven by compositor monotonic timestamps.
#[derive(Debug, Clone, Copy)]
pub struct CursorLocateAnimation {
    config: CursorAnimationConfig,
    started_at_ms: Option<u32>,
}

impl Default for CursorLocateAnimation {
    fn default() -> Self {
        Self { config: CursorAnimationConfig::default(), started_at_ms: None }
    }
}

impl CursorLocateAnimation {
    pub fn trigger(&mut self, now_ms: u32) {
        self.started_at_ms = Some(now_ms);
    }

    /// Returns the current scale. A value of 1.0 means normal size.
    pub fn scale_at(&mut self, now_ms: u32) -> f64 {
        let Some(started) = self.started_at_ms else { return 1.0 };
        let elapsed = now_ms.saturating_sub(started);
        let up = self.config.ramp_up_ms;
        let hold_end = up.saturating_add(self.config.hold_ms);
        let end = hold_end.saturating_add(self.config.ramp_down_ms);

        if elapsed >= end {
            self.started_at_ms = None;
            return 1.0;
        }
        if elapsed < up {
            let t = elapsed as f64 / up.max(1) as f64;
            return 1.0 + (self.config.peak_scale - 1.0) * smooth_step(t);
        }
        if elapsed < hold_end {
            return self.config.peak_scale;
        }

        let t = (elapsed - hold_end) as f64 / self.config.ramp_down_ms.max(1) as f64;
        1.0 + (self.config.peak_scale - 1.0) * (1.0 - smooth_step(t))
    }

    pub fn is_active(&self) -> bool { self.started_at_ms.is_some() }
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
        animation.trigger(100);
        assert_eq!(animation.scale_at(100), 1.0);
        assert!(animation.scale_at(210) > 2.39);
        assert_eq!(animation.scale_at(500), 2.4);
        assert!(animation.scale_at(900) < 2.4);
        assert_eq!(animation.scale_at(1100), 1.0);
        assert!(!animation.is_active());
    }

    #[test]
    fn retrigger_restarts_animation() {
        let mut animation = CursorLocateAnimation::default();
        animation.trigger(0);
        animation.trigger(50);
        assert_eq!(animation.scale_at(50), 1.0);
    }
}
