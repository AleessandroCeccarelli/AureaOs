// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer axis and scroll frame delivery.

use smithay::{
    backend::input::{Axis, AxisRelativeDirection, AxisSource},
    input::pointer::AxisFrame,
};

use super::AureaCompositor;

fn finite_axis_value(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite())
}

fn axis_v120_value(value: Option<f64>) -> Option<i32> {
    let value = value.filter(|value| value.is_finite())?.round();

    if value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return None;
    }

    Some(value as i32)
}

impl AureaCompositor {
    /// Sends one grouped scroll frame while preserving source, direction and high-resolution wheel data.
    /// Both vertical and horizontal wheel axes are forwarded without desktop-side reinterpretation.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_pointer_axis(
        &mut self,
        horizontal: Option<f64>,
        vertical: Option<f64>,
        horizontal_v120: Option<f64>,
        vertical_v120: Option<f64>,
        source: AxisSource,
        horizontal_direction: AxisRelativeDirection,
        vertical_direction: AxisRelativeDirection,
        time: u32,
    ) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        let mut frame = AxisFrame::new(time)
            .source(source)
            .relative_direction(Axis::Horizontal, horizontal_direction)
            .relative_direction(Axis::Vertical, vertical_direction);

        if let Some(value) = finite_axis_value(horizontal) {
            frame = frame.value(Axis::Horizontal, value);
        }
        if let Some(value) = finite_axis_value(vertical) {
            frame = frame.value(Axis::Vertical, value);
        }
        if let Some(value) = axis_v120_value(horizontal_v120) {
            frame = frame.v120(Axis::Horizontal, value);
        }
        if let Some(value) = axis_v120_value(vertical_v120) {
            frame = frame.v120(Axis::Vertical, value);
        }

        pointer.axis(self, frame);
        pointer.frame(self);
    }
}

#[cfg(test)]
mod tests {
    use super::{axis_v120_value, finite_axis_value};

    #[test]
    fn rejects_non_finite_axis_values() {
        assert_eq!(finite_axis_value(Some(f64::NAN)), None);
        assert_eq!(finite_axis_value(Some(f64::INFINITY)), None);
        assert_eq!(finite_axis_value(Some(12.5)), Some(12.5));
    }

    #[test]
    fn converts_only_representable_v120_values() {
        assert_eq!(axis_v120_value(Some(120.4)), Some(120));
        assert_eq!(axis_v120_value(Some(-120.6)), Some(-121));
        assert_eq!(axis_v120_value(Some(f64::NAN)), None);
        assert_eq!(axis_v120_value(Some(f64::INFINITY)), None);
        assert_eq!(axis_v120_value(Some(f64::MAX)), None);
    }
}
