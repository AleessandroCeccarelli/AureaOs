// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Logical monitor placement and per-output scale configuration.

use super::capabilities::DisplayId;

/// Position in the shared desktop logical coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayPosition {
    pub x: i32,
    pub y: i32,
}

impl DisplayPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Logical scale represented as numerator / 120, matching Wayland fractional
/// scale conventions. The supported range is 100% through 400%.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayScale {
    numerator: u32,
}

impl DisplayScale {
    pub const DENOMINATOR: u32 = 120;

    pub const fn new(numerator: u32) -> Option<Self> {
        if numerator < Self::DENOMINATOR || numerator > Self::DENOMINATOR * 4 {
            return None;
        }

        Some(Self { numerator })
    }

    pub const fn from_percent(percent: u32) -> Option<Self> {
        let Some(scaled) = percent.checked_mul(Self::DENOMINATOR) else {
            return None;
        };

        if scaled % 100 != 0 {
            return None;
        }

        Self::new(scaled / 100)
    }

    pub const fn numerator(self) -> u32 {
        self.numerator
    }

    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator) / f64::from(Self::DENOMINATOR)
    }
}

/// Per-display placement and activation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPlacement {
    pub display: DisplayId,
    pub enabled: bool,
    pub position: DisplayPosition,
    pub scale: DisplayScale,
}

/// Complete logical desktop layout. The primary display, when present, must
/// refer to an enabled display in the layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisplayLayout {
    pub placements: Vec<DisplayPlacement>,
    pub primary_display: Option<DisplayId>,
}

#[cfg(test)]
mod tests {
    use super::DisplayScale;

    #[test]
    fn represents_common_scale_percentages_exactly() {
        for percent in [100, 125, 150, 175, 200, 250, 300, 400] {
            assert!(DisplayScale::from_percent(percent).is_some());
        }
        assert!(DisplayScale::from_percent(101).is_none());
    }

    #[test]
    fn rejects_scale_outside_supported_range() {
        assert!(DisplayScale::new(119).is_none());
        assert!(DisplayScale::new(481).is_none());
    }
}
