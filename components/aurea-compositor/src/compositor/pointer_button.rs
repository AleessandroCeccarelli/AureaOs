// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer button delivery and click-to-focus policy.

use smithay::{backend::input::ButtonState, input::pointer::ButtonEvent, utils::SERIAL_COUNTER};
use aurea_desktop_api::input::InputDeviceId;

use super::AureaCompositor;

const BTN_LEFT: u32 = 0x110;

impl AureaCompositor {
    /// Sends a physical button event and focuses/raises the clicked toplevel.
    pub fn handle_pointer_button(&mut self, device: InputDeviceId, button: u32, state: ButtonState, time: u32) {
        match state {
            ButtonState::Pressed => {
                self.pressed_pointer_buttons.entry(device).or_default().insert(button);
            }
            ButtonState::Released => {
                if let Some(buttons) = self.pressed_pointer_buttons.get_mut(&device) {
                    buttons.remove(&button);
                    if buttons.is_empty() {
                        self.pressed_pointer_buttons.remove(&device);
                    }
                }
            }
        }

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

    /// Synthesizes releases for buttons still held by a device being removed.
    pub(super) fn release_pointer_buttons_for_device(&mut self, device: InputDeviceId, time: u32) {
        let Some(buttons) = self.pressed_pointer_buttons.remove(&device) else {
            return;
        };
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        for button in buttons {
            pointer.button(
                self,
                &ButtonEvent {
                    serial: SERIAL_COUNTER.next_serial(),
                    time,
                    button,
                    state: ButtonState::Released,
                },
            );
        }
        pointer.frame(self);
    }
}
