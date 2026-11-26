// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounded history of recent pointer positions for the locate trail.

use std::time::{Duration, Instant};

use smithay::utils::{Logical, Point};

/// Maximum number of historical cursor impressions.
pub const MAX_CURSOR_TRAIL_SAMPLES: usize = 3;
/// Old impressions are discarded quickly to keep the trail short.
pub const CURSOR_TRAIL_LIFETIME: Duration = Duration::from_millis(220);

/// One immutable cursor position and its remaining lifetime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorTrailSample {
    position: Point<f64, Logical>,
    opacity: f32,
}

impl CursorTrailSample {
    pub fn position(self) -> Point<f64, Logical> { self.position }
    pub fn opacity(self) -> f32 { self.opacity }
}

/// Collects recent positions only while the locate animation is active.
#[derive(Debug, Default)]
pub struct CursorTrail {
    samples: Vec<(Point<f64, Logical>, Instant)>,
    last_sample: Option<Instant>,
}

impl CursorTrail {
    /// Records a position, throttled to avoid duplicate impressions per frame.
    pub fn record(&mut self, position: Point<f64, Logical>, now: Instant) {
        if !position.x.is_finite() || !position.y.is_finite() { return; }
        if self.last_sample.is_some_and(|last| now.saturating_duration_since(last) < Duration::from_millis(24)) {
            return;
        }
        self.samples.push((position, now));
        self.samples.truncate(MAX_CURSOR_TRAIL_SAMPLES);
        self.last_sample = Some(now);
    }

    /// Returns newest-first samples with a linear temporal fade.
    pub fn visible_at(&mut self, now: Instant) -> Vec<CursorTrailSample> {
        self.samples.retain(|(_, created)| now.saturating_duration_since(*created) < CURSOR_TRAIL_LIFETIME);
        self.samples.iter().rev().filter_map(|(position, created)| {
            let age = now.saturating_duration_since(*created);
            let remaining = 1.0 - age.as_secs_f32() / CURSOR_TRAIL_LIFETIME.as_secs_f32();
            (remaining > 0.0).then_some(CursorTrailSample { position: *position, opacity: remaining.clamp(0.0, 1.0) })
        }).collect()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.last_sample = None;
    }

    pub fn is_active(&self, now: Instant) -> bool {
        self.samples.iter().any(|(_, created)| now.saturating_duration_since(*created) < CURSOR_TRAIL_LIFETIME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> Point<f64, Logical> { Point::from((x, y)) }

    #[test]
    fn keeps_at_most_three_recent_positions_newest_first() {
        let start = Instant::now();
        let mut trail = CursorTrail::default();
        for index in 0..5 {
            trail.record(point(index as f64, 0.0), start + Duration::from_millis(index * 30));
        }
        let samples = trail.visible_at(start + Duration::from_millis(130));
        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0].position(), point(4.0, 0.0));
    }

    #[test]
    fn fades_and_expires_after_lifetime() {
        let start = Instant::now();
        let mut trail = CursorTrail::default();
        trail.record(point(10.0, 20.0), start);
        let middle = trail.visible_at(start + Duration::from_millis(110));
        assert_eq!(middle.len(), 1);
        assert!((middle[0].opacity() - 0.5).abs() < 0.02);
        assert!(trail.visible_at(start + CURSOR_TRAIL_LIFETIME).is_empty());
    }

    #[test]
    fn rejects_non_finite_positions() {
        let mut trail = CursorTrail::default();
        trail.record(point(f64::NAN, 0.0), Instant::now());
        assert!(trail.visible_at(Instant::now()).is_empty());
    }
}
