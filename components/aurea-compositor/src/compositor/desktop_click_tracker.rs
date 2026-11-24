// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Recognizes desktop-item double-clicks without scheduling timers.

use std::time::{Duration, Instant};

const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);
const DOUBLE_CLICK_DISTANCE: f64 = 4.0;

#[derive(Debug, Clone)]
struct PreviousClick {
    item_id: String,
    x: f64,
    y: f64,
    at: Instant,
}

/// Tracks consecutive clicks on the same desktop item.
#[derive(Debug, Default)]
pub(super) struct DesktopClickTracker {
    previous: Option<PreviousClick>,
}

impl DesktopClickTracker {
    /// Records a click and returns true when it completes a double-click.
    pub(super) fn record(
        &mut self,
        item_id: &str,
        x: f64,
        y: f64,
        now: Instant,
    ) -> bool {
        if !x.is_finite() || !y.is_finite() {
            self.reset();
            return false;
        }

        let is_double_click = self.previous.as_ref().is_some_and(|previous| {
            let dx = x - previous.x;
            let dy = y - previous.y;
            previous.item_id == item_id
                && now.saturating_duration_since(previous.at) <= DOUBLE_CLICK_INTERVAL
                && dx * dx + dy * dy <= DOUBLE_CLICK_DISTANCE * DOUBLE_CLICK_DISTANCE
        });

        if is_double_click {
            self.reset();
        } else {
            self.previous = Some(PreviousClick {
                item_id: item_id.to_owned(),
                x,
                y,
                at: now,
            });
        }

        is_double_click
    }

    /// Cancels a pending first click, for example when a window is clicked.
    pub(super) fn reset(&mut self) {
        self.previous = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_two_clicks_on_the_same_item() {
        let start = Instant::now();
        let mut tracker = DesktopClickTracker::default();

        assert!(!tracker.record("system:trash", 10.0, 20.0, start));
        assert!(tracker.record(
            "system:trash",
            12.0,
            21.0,
            start + Duration::from_millis(250)
        ));
    }

    #[test]
    fn different_items_do_not_form_a_double_click() {
        let start = Instant::now();
        let mut tracker = DesktopClickTracker::default();

        assert!(!tracker.record("system:trash", 10.0, 20.0, start));
        assert!(!tracker.record(
            "home:documents",
            10.0,
            20.0,
            start + Duration::from_millis(100)
        ));
    }

    #[test]
    fn click_after_interval_starts_a_new_sequence() {
        let start = Instant::now();
        let mut tracker = DesktopClickTracker::default();

        assert!(!tracker.record("system:trash", 10.0, 20.0, start));
        assert!(!tracker.record(
            "system:trash",
            10.0,
            20.0,
            start + DOUBLE_CLICK_INTERVAL + Duration::from_millis(1)
        ));
    }

    #[test]
    fn excessive_pointer_distance_does_not_form_a_double_click() {
        let start = Instant::now();
        let mut tracker = DesktopClickTracker::default();

        assert!(!tracker.record("system:trash", 10.0, 20.0, start));
        assert!(!tracker.record(
            "system:trash",
            15.0,
            20.0,
            start + Duration::from_millis(100)
        ));
    }

    #[test]
    fn reset_cancels_the_pending_first_click() {
        let start = Instant::now();
        let mut tracker = DesktopClickTracker::default();

        assert!(!tracker.record("system:trash", 10.0, 20.0, start));
        tracker.reset();
        assert!(!tracker.record(
            "system:trash",
            10.0,
            20.0,
            start + Duration::from_millis(100)
        ));
    }

    #[test]
    fn triple_click_is_one_double_click_followed_by_a_new_first_click() {
        let start = Instant::now();
        let mut tracker = DesktopClickTracker::default();

        assert!(!tracker.record("system:trash", 10.0, 20.0, start));
        assert!(tracker.record(
            "system:trash",
            10.0,
            20.0,
            start + Duration::from_millis(100)
        ));
        assert!(!tracker.record(
            "system:trash",
            10.0,
            20.0,
            start + Duration::from_millis(200)
        ));
    }
}
