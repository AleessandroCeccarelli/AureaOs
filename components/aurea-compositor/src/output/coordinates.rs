// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Explicit conversions between global logical and output-local physical space.

use super::scale::ScaleFactor;

/// A point in global compositor logical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalPoint {
    pub x: i64,
    pub y: i64,
}

/// A point in output-local physical pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalPoint {
    pub x: i64,
    pub y: i64,
}

/// Coordinate transform owned by one output.
///
/// Output origins are always expressed in global logical coordinates.
/// Physical coordinates returned by this type are local to the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputCoordinateSpace {
    origin: LogicalPoint,
    scale: ScaleFactor,
}

impl OutputCoordinateSpace {
    pub const fn new(origin: LogicalPoint, scale: ScaleFactor) -> Self {
        Self { origin, scale }
    }

    pub const fn origin(self) -> LogicalPoint {
        self.origin
    }

    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }

    /// Maps a global logical point into this output's physical pixel space.
    pub fn logical_to_physical(self, point: LogicalPoint) -> Option<PhysicalPoint> {
        let local_x = point.x.checked_sub(self.origin.x)?;
        let local_y = point.y.checked_sub(self.origin.y)?;

        Some(PhysicalPoint {
            x: self.scale.logical_to_physical(local_x)?,
            y: self.scale.logical_to_physical(local_y)?,
        })
    }

    /// Maps an output-local physical point into global logical coordinates.
    pub fn physical_to_logical(self, point: PhysicalPoint) -> Option<LogicalPoint> {
        Some(LogicalPoint {
            x: self
                .scale
                .physical_to_logical(point.x)?
                .checked_add(self.origin.x)?,
            y: self
                .scale
                .physical_to_logical(point.y)?
                .checked_add(self.origin.y)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_global_origin_before_scaling() {
        let space = OutputCoordinateSpace::new(
            LogicalPoint { x: 1920, y: 0 },
            ScaleFactor::from_percent(150).unwrap(),
        );

        assert_eq!(
            space.logical_to_physical(LogicalPoint { x: 2020, y: 80 }),
            Some(PhysicalPoint { x: 150, y: 120 })
        );
    }

    #[test]
    fn converts_back_to_global_logical_space() {
        let space = OutputCoordinateSpace::new(
            LogicalPoint { x: 1920, y: 100 },
            ScaleFactor::from_percent(125).unwrap(),
        );

        assert_eq!(
            space.physical_to_logical(PhysicalPoint { x: 125, y: 250 }),
            Some(LogicalPoint { x: 2020, y: 300 })
        );
    }

    #[test]
    fn rejects_coordinate_overflow() {
        let space = OutputCoordinateSpace::new(
            LogicalPoint { x: i64::MIN, y: 0 },
            ScaleFactor::from_percent(100).unwrap(),
        );

        assert_eq!(
            space.logical_to_physical(LogicalPoint { x: i64::MAX, y: 0 }),
            None
        );
    }
}
