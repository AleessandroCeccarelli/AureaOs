// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer button delivery and click-to-focus policy.

use smithay::{
    backend::input::ButtonState,
    input::pointer::{ButtonEvent, PointerHandle},
    utils::SERIAL_COUNTER,
};

use super::AureaCompositor;

impl AureaCompositor {
    /// Sends a physical button event and focuses the clicked toplevel on press.
    pub fn handle_pointer_button(&mut self, button: u32, state: ButtonState, time: u32) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        if state == ButtonState::Pressed && button == 0x110 {
            let location = pointer.current_location();
            let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);

            if let Some((_, _, toplevel)) = scene.surface_under(location) {
                self.focus_surface(Some(toplevel.wl_surface().clone()));
            }
        }

        pointer.button(
            self,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time,
                button,
                state,
            },
        );
        pointer.frame(self);
    }
}
