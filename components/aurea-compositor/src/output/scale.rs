// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Exact per-output scale factors.
//!
//! The numerator uses Wayland's 120-based fractional scale convention.
//! Keeping the value rational avoids floating-point drift in coordinate
//! conversion and configuration comparisons.

/// Denominator used by Wayland fractional-scale protocol values.
pub const SCALE_DENOMINATOR: u32 = 120;

/// Validated output scale represented as numerator / 120.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScaleFactor {
    numerator: u32,
}

impl ScaleFactor {
    /// Creates a scale factor in the supported 100%-to-400% range.
    pub const fn new(numerator: u32) -> Option<Self> {
        if numerator < SCALE_DENOMINATOR || numerator > SCALE_DENOMINATOR * 4 {
            return None;
        }

        Some(Self { numerator })
    }

    /// Converts an integer percentage when it is exactly representable.
    ///
    /// Common desktop values such as 100%, 125%, 150%, 175%, 200%,
    /// 250%, 300% and 400% are represented without rounding.
    pub const fn from_percent(percent: u32) -> Option<Self> {
        let Some(scaled) = percent.checked_mul(SCALE_DENOMINATOR) else {
            return None;
        };

        if scaled % 100 != 0 {
            return None;
        }

        Self::new(scaled / 100)
    }

    /// Returns the numerator used by the Wayland fractional-scale protocol.
    pub const fn numerator(self) -> u32 {
        self.numerator
    }

    /// Returns the scale as a floating-point value for APIs that require it.
    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator) / f64::from(SCALE_DENOMINATOR)
    }

    /// Converts a signed logical coordinate to physical pixels.
    ///
    /// Rounding is symmetric around zero and uses the nearest pixel.
    pub fn logical_to_physical(self, logical: i64) -> Option<i64> {
        let value = i128::from(logical).checked_mul(i128::from(self.numerator))?;
        let rounded = round_div(value, i128::from(SCALE_DENOMINATOR))?;
        i64::try_from(rounded).ok()
    }

    /// Converts a signed physical coordinate to logical pixels.
    pub fn physical_to_logical(self, physical: i64) -> Option<i64> {
        let value = i128::from(physical).checked_mul(i128::from(SCALE_DENOMINATOR))?;
        let rounded = round_div(value, i128::from(self.numerator))?;
        i64::try_from(rounded).ok()
    }

    /// Converts a non-zero logical extent to its nearest physical extent.
    pub fn logical_extent_to_physical(self, logical: u32) -> Option<u32> {
        if logical == 0 {
            return None;
        }

        let value = u128::from(logical).checked_mul(u128::from(self.numerator))?;
        let denominator = u128::from(SCALE_DENOMINATOR);
        let rounded = value.checked_add(denominator / 2)? / denominator;
        u32::try_from(rounded).ok().filter(|extent| *extent > 0)
    }
}

fn round_div(value: i128, denominator: i128) -> Option<i128> {
    if denominator <= 0 {
        return None;
    }

    let half = denominator / 2;
    if value >= 0 {
        value
            .checked_add(half)
            .map(|adjusted| adjusted / denominator)
    } else {
        value
            .checked_sub(half)
            .map(|adjusted| adjusted / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn represents_common_fractional_scales_exactly() {
        for (percent, numerator) in [
            (100, 120),
            (125, 150),
            (150, 180),
            (175, 210),
            (200, 240),
            (250, 300),
            (300, 360),
            (400, 480),
        ] {
            let scale = ScaleFactor::from_percent(percent).unwrap();
            assert_eq!(scale.numerator(), numerator);
        }
    }

    #[test]
    fn rejects_out_of_range_and_inexact_percentages() {
        assert!(ScaleFactor::new(119).is_none());
        assert!(ScaleFactor::new(481).is_none());
        assert!(ScaleFactor::from_percent(101).is_none());
        assert!(ScaleFactor::from_percent(u32::MAX).is_none());
    }

    #[test]
    fn converts_negative_coordinates_symmetrically() {
        let scale = ScaleFactor::from_percent(125).unwrap();
        assert_eq!(scale.logical_to_physical(10), Some(13));
        assert_eq!(scale.logical_to_physical(-10), Some(-13));
        assert_eq!(scale.physical_to_logical(13), Some(10));
        assert_eq!(scale.physical_to_logical(-13), Some(-10));
    }

    #[test]
    fn converts_extents_without_zero_sized_results() {
        let scale = ScaleFactor::from_percent(150).unwrap();
        assert_eq!(scale.logical_extent_to_physical(100), Some(150));
        assert_eq!(scale.logical_extent_to_physical(0), None);
    }
}
