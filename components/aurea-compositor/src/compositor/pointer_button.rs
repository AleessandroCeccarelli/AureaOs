// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer button delivery and click-to-focus policy.

use smithay::{backend::input::ButtonState, input::pointer::ButtonEvent, utils::SERIAL_COUNTER};

use super::AureaCompositor;

const BTN_LEFT: u32 = 0x110;

impl AureaCompositor {
    /// Sends a physical button event and focuses/raises the clicked toplevel.
    pub fn handle_pointer_button(&mut self, button: u32, state: ButtonState, time: u32) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        if state == ButtonState::Pressed && button == BTN_LEFT {
            let location = pointer.current_location();
            let focused_toplevel = {
                let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
                scene
                    .surface_under(location)
                    .map(|(_, _, toplevel)| toplevel.wl_surface().clone())
            };

            if let Some(surface) = focused_toplevel {
                self.window_manager.raise_toplevel(&surface);
                self.focus_surface(Some(surface));
                self.request_all_output_frames();
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
