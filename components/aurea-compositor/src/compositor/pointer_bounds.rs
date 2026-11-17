// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pure pointer-coordinate constraints derived from the enabled output layout.

use smithay::utils::{Logical, Point};

/// One enabled output rectangle in global logical compositor coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct OutputGeometry {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl OutputGeometry {
    /// Creates a logical output rectangle.
    ///
    /// Empty, negative, or non-finite dimensions are rejected.
    pub(super) fn new(x: f64, y: f64, width: f64, height: f64) -> Option<Self> {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width < 1.0
            || height < 1.0
        {
            return None;
        }

        Some(Self {
            x,
            y,
            width,
            height,
        })
    }

    fn clamp(self, point: Point<f64, Logical>) -> Point<f64, Logical> {
        Point::from((
            point.x.clamp(self.x, self.x + self.width - 1.0),
            point.y.clamp(self.y, self.y + self.height - 1.0),
        ))
    }

    fn contains(self, point: Point<f64, Logical>) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }
}

/// Keeps a pointer inside the enabled desktop outputs.
///
/// If the requested location falls into a gap between monitors, the nearest
/// valid point on any output is selected. Returns None when no output is active.
pub(super) fn constrain_to_outputs(
    point: Point<f64, Logical>,
    outputs: impl IntoIterator<Item = OutputGeometry>,
) -> Option<Point<f64, Logical>> {
    let outputs = outputs.into_iter().collect::<Vec<_>>();

    if outputs.is_empty() || !point.x.is_finite() || !point.y.is_finite() {
        return None;
    }

    if outputs.iter().any(|output| output.contains(point)) {
        return Some(point);
    }

    outputs
        .into_iter()
        .map(|output| output.clamp(point))
        .min_by(|left, right| distance_squared(*left, point).total_cmp(&distance_squared(*right, point)))
}

fn distance_squared(left: Point<f64, Logical>, right: Point<f64, Logical>) -> f64 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    dx.mul_add(dx, dy * dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(x: f64, y: f64, width: f64, height: f64) -> OutputGeometry {
        OutputGeometry::new(x, y, width, height).unwrap()
    }

    #[test]
    fn keeps_locations_inside_an_output_unchanged() {
        let point = Point::from((24.5, 18.25));
        assert_eq!(
            constrain_to_outputs(point, [output(0.0, 0.0, 100.0, 80.0)]),
            Some(point)
        );
    }

    #[test]
    fn clamps_locations_beyond_output_edges() {
        assert_eq!(
            constrain_to_outputs(
                Point::from((150.0, 90.0)),
                [output(0.0, 0.0, 100.0, 80.0)]
            ),
            Some(Point::from((99.0, 79.0)))
        );
    }

    #[test]
    fn moves_locations_in_monitor_gaps_to_the_nearest_output() {
        let left = output(0.0, 0.0, 100.0, 100.0);
        let right = output(120.0, 0.0, 100.0, 100.0);

        assert_eq!(
            constrain_to_outputs(Point::from((110.0, 50.0)), [left, right]),
            Some(Point::from((99.0, 50.0)))
        );
    }

    #[test]
    fn supports_outputs_with_negative_global_origins() {
        let point = Point::from((-150.0, 20.0));
        assert_eq!(
            constrain_to_outputs(point, [output(-200.0, 0.0, 100.0, 80.0)]),
            Some(point)
        );
    }

    #[test]
    fn returns_none_when_no_output_is_enabled() {
        assert_eq!(
            constrain_to_outputs(Point::from((0.0, 0.0)), []),
            None
        );
    }

    #[test]
    fn rejects_invalid_output_geometry() {
        assert!(OutputGeometry::new(0.0, 0.0, 0.0, 100.0).is_none());
        assert!(OutputGeometry::new(0.0, 0.0, f64::NAN, 100.0).is_none());
    }
}
