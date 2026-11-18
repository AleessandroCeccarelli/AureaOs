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

/// A half-open rectangle in global compositor logical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalRect {
    origin: LogicalPoint,
    size: LogicalSize,
}

impl LogicalRect {
    /// Creates a rectangle. Empty extents are valid values but never intersect.
    pub const fn new(origin: LogicalPoint, size: LogicalSize) -> Self {
        Self { origin, size }
    }

    pub const fn origin(self) -> LogicalPoint {
        self.origin
    }

    pub const fn size(self) -> LogicalSize {
        self.size
    }

    /// Returns the exclusive right and bottom edges, or None on overflow/emptiness.
    fn edges(self) -> Option<(i64, i64)> {
        if self.size.width == 0 || self.size.height == 0 {
            return None;
        }

        Some((
            self.origin.x.checked_add(i64::from(self.size.width))?,
            self.origin.y.checked_add(i64::from(self.size.height))?,
        ))
    }
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
    /// Builds bounds from an already converted logical origin and size.
    ///
    /// Zero-sized outputs and coordinates whose exclusive edges overflow are
    /// rejected before the bounds can enter output/surface intersection logic.
    pub fn from_logical_size(origin: LogicalPoint, size: LogicalSize) -> Option<Self> {
        if size.width == 0 || size.height == 0 {
            return None;
        }

        origin.x.checked_add(i64::from(size.width))?;
        origin.y.checked_add(i64::from(size.height))?;

        Some(Self { origin, size })
    }

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

    /// Returns true when a non-empty logical rectangle overlaps this output.
    ///
    /// Both rectangles use half-open edges, so surfaces touching only an output
    /// boundary do not enter that output.
    pub fn intersects(self, rect: LogicalRect) -> bool {
        let Some((output_right, output_bottom)) = LogicalRect::new(self.origin, self.size).edges()
        else {
            return false;
        };
        let Some((rect_right, rect_bottom)) = rect.edges() else {
            return false;
        };

        rect.origin.x < output_right
            && self.origin.x < rect_right
            && rect.origin.y < output_bottom
            && self.origin.y < rect_bottom
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
    fn surface_rect_intersects_output_when_area_overlaps() {
        let output = OutputLogicalBounds::from_physical_mode(
            LogicalPoint { x: 100, y: 50 },
            1920,
            1080,
            ScaleFactor::from_percent(100).unwrap(),
        )
        .unwrap();

        let surface = LogicalRect::new(
            LogicalPoint { x: 90, y: 60 },
            LogicalSize { width: 20, height: 20 },
        );

        assert!(output.intersects(surface));
    }

    #[test]
    fn surface_rect_touching_output_edge_does_not_intersect() {
        let output = OutputLogicalBounds::from_physical_mode(
            LogicalPoint { x: 100, y: 50 },
            1920,
            1080,
            ScaleFactor::from_percent(100).unwrap(),
        )
        .unwrap();

        let adjacent = LogicalRect::new(
            LogicalPoint { x: 2020, y: 50 },
            LogicalSize { width: 100, height: 100 },
        );

        assert!(!output.intersects(adjacent));
    }

    #[test]
    fn empty_or_overflowing_surface_rect_does_not_intersect() {
        let output = OutputLogicalBounds::from_physical_mode(
            LogicalPoint { x: 0, y: 0 },
            1920,
            1080,
            ScaleFactor::from_percent(100).unwrap(),
        )
        .unwrap();

        assert!(!output.intersects(LogicalRect::new(
            LogicalPoint { x: 10, y: 10 },
            LogicalSize { width: 0, height: 20 },
        )));
        assert!(!output.intersects(LogicalRect::new(
            LogicalPoint { x: i64::MAX, y: 0 },
            LogicalSize { width: 10, height: 10 },
        )));
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
