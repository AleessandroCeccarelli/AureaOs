// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared display color policy contracts.

/// A technology-independent color adjustment requested for a display output.
///
/// The compositor owns the implementation. A value of zero means that the
/// output must remain unchanged; the range is inclusive from 0 to 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayColorAdjustment {
    warmth_percent: u8,
}

impl DisplayColorAdjustment {
    /// Creates an adjustment, clamping its intensity to the supported range.
    pub const fn new(warmth_percent: u8) -> Self {
        Self {
            warmth_percent: if warmth_percent > 100 { 100 } else { warmth_percent },
        }
    }

    /// Returns the requested warmth intensity in the range 0–100.
    pub const fn warmth_percent(self) -> u8 {
        self.warmth_percent
    }

    /// Returns whether no color adjustment is requested.
    pub const fn is_neutral(self) -> bool {
        self.warmth_percent == 0
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayColorAdjustment;

    #[test]
    fn default_adjustment_is_neutral() {
        let adjustment = DisplayColorAdjustment::default();
        assert!(adjustment.is_neutral());
        assert_eq!(adjustment.warmth_percent(), 0);
    }

    #[test]
    fn intensity_is_clamped_to_the_supported_range() {
        assert_eq!(DisplayColorAdjustment::new(65).warmth_percent(), 65);
        assert_eq!(DisplayColorAdjustment::new(u8::MAX).warmth_percent(), 100);
    }
}
