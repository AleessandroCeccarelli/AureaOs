// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Frame scheduling policy for the short cursor locate trail.

use std::time::Instant;

use super::cursor_trail::CursorTrail;

/// Keeps the trail's output frames alive only while impressions remain visible.
#[derive(Debug, Default)]
pub struct CursorTrailAnimation;

impl CursorTrailAnimation {
    pub fn needs_frame(&self, trail: &mut CursorTrail, now: Instant) -> bool {
        trail.is_active(now)
    }
}
