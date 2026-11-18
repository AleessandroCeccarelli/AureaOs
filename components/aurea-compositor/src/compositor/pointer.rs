// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer capability initialization and relative motion routing.

use std::io;

use smithay::{
    input::pointer::{MotionEvent, PointerHandle},
    utils::{Point, SERIAL_COUNTER},
};

use super::{AureaCompositor, pointer_bounds::OutputGeometry};

impl AureaCompositor {
    /// Initializes the seat's Wayland pointer capability.
    pub fn initialize_pointer(&mut self) -> io::Result<()> {
        if self.seat.get_pointer().is_none() {
            let _pointer: PointerHandle<Self> = self.seat.add_pointer();
        }

        Ok(())
    }

    /// Routes relative device motion to the topmost client surface.
    ///
    /// Coordinates are maintained in global logical compositor space. Motion
    /// is constrained to the union of enabled output rectangles so the pointer
    /// cannot become stranded in desktop gaps or beyond a display edge.
    /// Absolute devices are handled separately because they require
    /// output-local mapping.
    pub fn handle_pointer_motion(\n        &mut self,\n        delta: Point<f64, smithay::utils::Logical>,\n        unaccelerated_delta: Point<f64, smithay::utils::Logical>,\n        time: u32,\n    ) {
        if self.pointer_settings.shake_to_locate()\n            && self.pointer_shake.update(unaccelerated_delta, time)\n        {\n            self.cursor_locate_animation.trigger(time);\n        }\n\n        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        let current = pointer.current_location();
        let requested = Point::from((current.x + delta.x, current.y + delta.y));
        let output_geometry = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .filter_map(|output| {
                let position = self.output_manager.layout().get(output.id())?;
                let mode = output.current_mode();
                let scale = f64::from(output.scale());

                OutputGeometry::new(
                    f64::from(position.x()),
                    f64::from(position.y()),
                    f64::from(mode.width()) / scale,
                    f64::from(mode.height()) / scale,
                )
            })
            .collect::<Vec<_>>();

        let Some(location) =
            super::pointer_bounds::constrain_to_outputs(requested, output_geometry.iter().copied())
        else {
            return;
        };

        let focus = {
            let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
            scene.surface_under(location).map(|(surface, origin, _)| {
                (
                    surface,
                    Point::from((f64::from(origin.x), f64::from(origin.y))),
                )
            })
        };

        pointer.motion(
            self,
            focus,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        pointer.frame(self);
        self.request_all_output_frames();
    }
}
