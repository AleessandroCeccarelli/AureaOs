// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Event-driven pointer placement after desktop output lifecycle changes.

use smithay::{
    input::pointer::MotionEvent,
    utils::{Clock, Monotonic, Point, SERIAL_COUNTER},
};

use super::AureaCompositor;

impl AureaCompositor {
    /// Places the pointer at the center of the first enabled output.
    ///
    /// This is intentionally called only after output startup, session
    /// restoration, or output reconciliation. It does not run in the frame
    /// loop and does not move a pointer while a client owns an active grab.
    pub fn center_pointer_on_primary_output(&mut self) -> bool {
        let Some(pointer) = self.seat.get_pointer() else {
            return false;
        };

        if pointer.is_grabbed() {
            return false;
        }

        let target = self
            .output_manager
            .registry()
            .iter()
            .filter(|output| output.is_enabled())
            .find_map(|output| {
                let bounds = self.output_manager.logical_bounds(output.id()).ok()??;
                let origin = bounds.origin();
                let size = bounds.size();

                Some(Point::from((
                    origin.x as f64 + f64::from(size.width) / 2.0,
                    origin.y as f64 + f64::from(size.height) / 2.0,
                )))
            });

        let Some(location) = target else {
            return false;
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

        let time = Clock::<Monotonic>::new().now().as_millis() as u32;
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
        true
    }
}
