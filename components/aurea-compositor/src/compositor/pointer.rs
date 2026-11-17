// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer capability initialization and relative motion routing.

use std::io;

use smithay::{
    backend::input::PointerMotionEvent as BackendPointerMotionEvent,
    input::pointer::{MotionEvent, PointerHandle},
    utils::{Point, SERIAL_COUNTER},
};

use super::AureaCompositor;

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
    /// Coordinates are maintained in global logical compositor space. Absolute
    /// devices are handled separately because they require output-local mapping.
    pub fn handle_pointer_motion(&mut self, delta: Point<f64, smithay::utils::Logical>, time: u32) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        let current = pointer.current_location();
        let location = Point::from((current.x + delta.x, current.y + delta.y));

        let focus = {
            let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
            scene
                .surface_under(location)
                .map(|(surface, origin, _)| (surface, origin.to_f64()))
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
    }

    /// Resolves a backend relative-motion event without changing its units.
    pub fn handle_backend_pointer_motion<B, E>(&mut self, event: &E)
    where
        B: smithay::backend::input::InputBackend,
        E: BackendPointerMotionEvent<B>,
    {
        self.handle_pointer_motion(event.delta(), event.time() as u32);
    }
}
