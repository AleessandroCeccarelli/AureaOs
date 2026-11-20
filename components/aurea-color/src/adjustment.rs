// SPDX-License-Identifier: GPL-3.0-or-later
// AureaOS — Color Management

//! Non-destructive user adjustments layered on top of an immutable ICC profile.

/// Three-channel RGB adjustment values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbValues {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl RgbValues {
    pub const NEUTRAL: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    fn is_finite(self) -> bool {
        self.red.is_finite() && self.green.is_finite() && self.blue.is_finite()
    }

    fn within(self, minimum: f32, maximum: f32) -> bool {
        self.red >= minimum
            && self.red <= maximum
            && self.green >= minimum
            && self.green <= maximum
            && self.blue >= minimum
            && self.blue <= maximum
    }
}

/// A 3×3 linear RGB correction matrix, stored in row-major order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbMatrix {
    pub values: [[f32; 3]; 3],
}

impl RgbMatrix {
    pub const IDENTITY: Self = Self {
        values: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
    };

    fn is_finite(self) -> bool {
        self.values.iter().flatten().all(|value| value.is_finite())
    }

    fn determinant(self) -> f32 {
        let m = self.values;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }
}

/// User-editable calibration layered over an ICC profile without changing its bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorAdjustment {
    /// Per-channel linear gain. Neutral value is 1.0.
    pub channel_gain: RgbValues,
    /// Per-channel gamma. Neutral value is 1.0.
    pub gamma: RgbValues,
    /// Linear correction matrix. Neutral value is the identity matrix.
    pub matrix: RgbMatrix,
    /// Exposure compensation in photographic stops.
    pub exposure_stops: f32,
}

impl Default for ColorAdjustment {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl ColorAdjustment {
    pub const NEUTRAL: Self = Self {
        channel_gain: RgbValues::NEUTRAL,
        gamma: RgbValues::NEUTRAL,
        matrix: RgbMatrix::IDENTITY,
        exposure_stops: 0.0,
    };

    /// Checks all values against bounded ranges before the rendering engine uses them.
    pub fn validate(&self) -> Result<(), ColorAdjustmentError> {
        if !self.channel_gain.is_finite()
            || !self.gamma.is_finite()
            || !self.matrix.is_finite()
            || !self.exposure_stops.is_finite()
        {
            return Err(ColorAdjustmentError::NonFiniteValue);
        }

        if !self.channel_gain.within(0.0, 4.0) {
            return Err(ColorAdjustmentError::GainOutOfRange);
        }

        if !self.gamma.within(0.1, 4.0) {
            return Err(ColorAdjustmentError::GammaOutOfRange);
        }

        if self.matrix.values.iter().flatten().any(|value| !(-4.0..=4.0).contains(value)) {
            return Err(ColorAdjustmentError::MatrixValueOutOfRange);
        }

        let determinant = self.matrix.determinant();
        if !determinant.is_finite() || determinant.abs() < 0.000_001 {
            return Err(ColorAdjustmentError::SingularMatrix);
        }

        if !(-8.0..=8.0).contains(&self.exposure_stops) {
            return Err(ColorAdjustmentError::ExposureOutOfRange);
        }

        Ok(())
    }
}

/// Validation errors for user-authored color adjustments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorAdjustmentError {
    NonFiniteValue,
    GainOutOfRange,
    GammaOutOfRange,
    MatrixValueOutOfRange,
    SingularMatrix,
    ExposureOutOfRange,
}

#[cfg(test)]
mod tests {
    use super::{ColorAdjustment, ColorAdjustmentError, RgbValues};

    #[test]
    fn neutral_adjustment_is_valid() {
        assert_eq!(ColorAdjustment::NEUTRAL.validate(), Ok(()));
    }

    #[test]
    fn accepts_bounded_user_adjustments() {
        let adjustment = ColorAdjustment {
            channel_gain: RgbValues { red: 1.1, green: 0.95, blue: 1.05 },
            gamma: RgbValues { red: 1.0, green: 1.02, blue: 0.98 },
            exposure_stops: 0.5,
            ..ColorAdjustment::NEUTRAL
        };
        assert_eq!(adjustment.validate(), Ok(()));
    }

    #[test]
    fn rejects_non_finite_values() {
        let adjustment = ColorAdjustment {
            exposure_stops: f32::NAN,
            ..ColorAdjustment::NEUTRAL
        };
        assert_eq!(adjustment.validate(), Err(ColorAdjustmentError::NonFiniteValue));
    }

    #[test]
    fn rejects_out_of_range_gain() {
        let adjustment = ColorAdjustment {
            channel_gain: RgbValues { red: 5.0, ..RgbValues::NEUTRAL },
            ..ColorAdjustment::NEUTRAL
        };
        assert_eq!(adjustment.validate(), Err(ColorAdjustmentError::GainOutOfRange));
    }

    #[test]
    fn rejects_singular_matrix() {
        let adjustment = ColorAdjustment {
            matrix: super::RgbMatrix { values: [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]] },
            ..ColorAdjustment::NEUTRAL
        };
        assert_eq!(adjustment.validate(), Err(ColorAdjustmentError::SingularMatrix));
    }
}
