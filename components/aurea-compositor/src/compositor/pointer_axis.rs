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

impl AureaCompositor {
    /// Sends one grouped scroll frame while preserving source and direction data.
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

        if let Some(value) = horizontal {
            frame = frame.value(Axis::Horizontal, value);
        }
        if let Some(value) = vertical {
            frame = frame.value(Axis::Vertical, value);
        }
        if let Some(value) = horizontal_v120 {
            frame = frame.v120(Axis::Horizontal, value.round() as i32);
        }
        if let Some(value) = vertical_v120 {
            frame = frame.v120(Axis::Vertical, value.round() as i32);
        }

        pointer.axis(self, frame);
        pointer.frame(self);
    }
}
