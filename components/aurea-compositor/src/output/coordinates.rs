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

/// A non-zero extent in global logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalSize {
    pub width: u32,
    pub height: u32,
}

/// Half-open logical rectangle occupied by one output.
///
/// The right and bottom edges are excluded. This makes adjacent output
/// boundaries deterministic and prevents two monitors claiming the same
/// pointer coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputLogicalBounds {
    origin: LogicalPoint,
    size: LogicalSize,
}

impl OutputLogicalBounds {
    /// Builds logical bounds from a physical mode and its output scale.
    pub fn from_physical_mode(
        origin: LogicalPoint,
        physical_width: u32,
        physical_height: u32,
        scale: ScaleFactor,
    ) -> Option<Self> {
        if physical_width == 0 || physical_height == 0 {
            return None;
        }

        let width = scale.physical_to_logical(i64::from(physical_width))?;
        let height = scale.physical_to_logical(i64::from(physical_height))?;
        let size = LogicalSize {
            width: u32::try_from(width).ok().filter(|value| *value > 0)?,
            height: u32::try_from(height).ok().filter(|value| *value > 0)?,
        };

        origin.x.checked_add(i64::from(size.width))?;
        origin.y.checked_add(i64::from(size.height))?;

        Some(Self { origin, size })
    }

    pub const fn origin(self) -> LogicalPoint {
        self.origin
    }

    pub const fn size(self) -> LogicalSize {
        self.size
    }

    /// Returns true when the point belongs to this output's half-open bounds.
    pub fn contains(self, point: LogicalPoint) -> bool {
        let Some(right) = self.origin.x.checked_add(i64::from(self.size.width)) else {
            return false;
        };
        let Some(bottom) = self.origin.y.checked_add(i64::from(self.size.height)) else {
            return false;
        };

        point.x >= self.origin.x && point.y >= self.origin.y && point.x < right && point.y < bottom
    }

    pub fn coordinate_space(self, scale: ScaleFactor) -> OutputCoordinateSpace {
        OutputCoordinateSpace::new(self.origin, scale)
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
    fn mixed_scale_output_bounds_have_deterministic_edges() {
        let left = OutputLogicalBounds::from_physical_mode(
            LogicalPoint { x: 0, y: 0 },
            1920,
            1080,
            ScaleFactor::from_percent(100).unwrap(),
        )
        .unwrap();
        let right = OutputLogicalBounds::from_physical_mode(
            LogicalPoint { x: 1920, y: 0 },
            3840,
            2160,
            ScaleFactor::from_percent(200).unwrap(),
        )
        .unwrap();

        assert!(left.contains(LogicalPoint { x: 1919, y: 1079 }));
        assert!(!left.contains(LogicalPoint { x: 1920, y: 500 }));
        assert!(right.contains(LogicalPoint { x: 1920, y: 0 }));
        assert_eq!(
            right.size(),
            LogicalSize {
                width: 1920,
                height: 1080
            }
        );
    }

    #[test]
    fn rejects_zero_or_overflowing_output_bounds() {
        let scale = ScaleFactor::from_percent(100).unwrap();
        assert!(
            OutputLogicalBounds::from_physical_mode(LogicalPoint { x: 0, y: 0 }, 0, 1080, scale)
                .is_none()
        );
        assert!(
            OutputLogicalBounds::from_physical_mode(
                LogicalPoint { x: i64::MAX, y: 0 },
                1920,
                1080,
                scale
            )
            .is_none()
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
