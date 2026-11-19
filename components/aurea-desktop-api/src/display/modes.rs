// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Validated display modes and refresh-rate values.

/// A fixed display mode. Refresh rate is expressed in millihertz to avoid
/// floating-point comparisons (for example, 59.94 Hz is 59_940 mHz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplayMode {
    width: u32,
    height: u32,
    refresh_millihz: u32,
}

impl DisplayMode {
    pub const fn new(width: u32, height: u32, refresh_millihz: u32) -> Option<Self> {
        if width == 0 || height == 0 || refresh_millihz == 0 {
            return None;
        }

        Some(Self {
            width,
            height,
            refresh_millihz,
        })
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }

    pub const fn refresh_millihz(self) -> u32 {
        self.refresh_millihz
    }

    pub fn refresh_hz(self) -> f64 {
        f64::from(self.refresh_millihz) / 1_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayMode;

    #[test]
    fn stores_fractional_refresh_rates_without_float_rounding() {
        let mode = DisplayMode::new(1920, 1080, 59_940).unwrap();
        assert_eq!(mode.refresh_millihz(), 59_940);
        assert!((mode.refresh_hz() - 59.94).abs() < f64::EPSILON);
    }

    #[test]
    fn rejects_zero_dimensions_and_refresh() {
        assert!(DisplayMode::new(0, 1080, 60_000).is_none());
        assert!(DisplayMode::new(1920, 0, 60_000).is_none());
        assert!(DisplayMode::new(1920, 1080, 0).is_none());
    }
}
