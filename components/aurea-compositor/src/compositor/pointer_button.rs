// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer button delivery and click-to-focus policy.

use aurea_desktop_api::{
    desktop_interaction::{DesktopInteraction, DesktopPosition},
    input::{InputDeviceId, InputDeviceProvider},
};
use smithay::{backend::input::ButtonState, input::pointer::ButtonEvent, utils::SERIAL_COUNTER};

use super::AureaCompositor;

const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;

impl AureaCompositor {
    /// Sends a physical button event and focuses/raises the clicked toplevel.
    pub fn handle_pointer_button(
        &mut self,
        device: InputDeviceId,
        button: u32,
        state: ButtonState,
        time: u32,
    ) {
        let seat_transition = self
            .pointer_manager
            .update_button(device, button, state == ButtonState::Pressed);
        if !seat_transition {
            return;
        }

        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        if state == ButtonState::Pressed && matches!(button, BTN_LEFT | BTN_RIGHT) {
            let location = pointer.current_location();
            let focused_toplevel = {
                let scene = crate::render::RenderScene::from_window_manager(&self.window_manager);
                scene
                    .surface_under(location)
                    .map(|(_, _, toplevel)| toplevel.wl_surface().clone())
            };

            match (button, focused_toplevel) {
                (BTN_LEFT, Some(surface)) => {
                    self.desktop_click_tracker.reset();
                    self.window_manager.raise_toplevel(&surface);
                    self.focus_surface(Some(surface));
                    self.request_all_output_frames();
                }
                (BTN_LEFT, None) => self.desktop_click_tracker.reset(),
                (BTN_RIGHT, Some(_)) => {
                    // A window owns its own context-menu interaction. Do not
                    // replace it with the desktop shell menu.
                    self.desktop_click_tracker.reset();
                }
                (BTN_RIGHT, None) => {
                    self.desktop_click_tracker.reset();
                    let position = DesktopPosition {
                        x: location.x.round() as i32,
                        y: location.y.round() as i32,
                    };
                    if self
                        .desktop_interactions
                        .send(DesktopInteraction::ShowContextMenu { position })
                        .is_err()
                    {
                        tracing::debug!("Shell context-menu receiver is no longer available");
                    }
                },
                _ => {}
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

    /// Releases every button still held by any physical pointer before the
    /// native session is paused. This prevents a client from retaining a
    /// logically pressed button across a VT switch when libinput cannot
    /// deliver the corresponding physical release event.
    pub(crate) fn release_all_pointer_buttons(&mut self, time: u32) {
        let devices = self
            .pointer_manager
            .devices()
            .devices()
            .into_iter()
            .map(|device| device.id)
            .collect::<Vec<_>>();

        for device in devices {
            self.release_pointer_buttons_for_device(device, time);
        }
    }

    /// Synthesizes releases for buttons still held by a device being removed.
    pub(super) fn release_pointer_buttons_for_device(&mut self, device: InputDeviceId, time: u32) {
        let buttons = self.pointer_manager.take_buttons_released_by_device(device);
        if buttons.is_empty() {
            return;
        }
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
