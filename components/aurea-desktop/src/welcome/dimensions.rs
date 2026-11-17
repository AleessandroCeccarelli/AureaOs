// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Aurea Desktop
// Copyright (C) 2026 Alessandro Ceccarelli

//! Bounds and fallback rules for the Welcome surface dimensions.

use super::{HEIGHT, WIDTH};

const MIN_WIDTH: usize = 560;
const MIN_HEIGHT: usize = 360;
const MAX_WIDTH: usize = 1920;
const MAX_HEIGHT: usize = 1200;

/// Resolves a compositor size hint into safe, bounded buffer dimensions.
pub(super) fn resolve_size(width: i32, height: i32) -> (usize, usize) {
    let width = if width <= 0 { WIDTH } else { width as usize };
    let height = if height <= 0 { HEIGHT } else { height as usize };

    (
        width.clamp(MIN_WIDTH, MAX_WIDTH),
        height.clamp(MIN_HEIGHT, MAX_HEIGHT),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_default_dimensions_for_unspecified_size() {
        assert_eq!(resolve_size(0, 0), (WIDTH, HEIGHT));
    }

    #[test]
    fn clamps_dimensions_to_supported_bounds() {
        assert_eq!(resolve_size(1, 1), (MIN_WIDTH, MIN_HEIGHT));
        assert_eq!(resolve_size(i32::MAX, i32::MAX), (MAX_WIDTH, MAX_HEIGHT));
    }

    #[test]
    fn preserves_supported_compositor_size() {
        assert_eq!(resolve_size(1280, 720), (1280, 720));
    }
}
