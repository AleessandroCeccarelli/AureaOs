// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Pointer button delivery and click-to-focus policy.

use aurea_desktop_api::{
    desktop::{DesktopContextMenuLayout, DesktopPoint, DesktopRect},
    desktop_interaction::{DesktopInteraction, DesktopPosition},
    input::{InputDeviceId, InputDeviceProvider},
};
use smithay::{backend::input::ButtonState, input::pointer::ButtonEvent, utils::SERIAL_COUNTER};

use super::AureaCompositor;

const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;

// BTN_SIDE/BTN_EXTRA and BTN_FORWARD/BTN_BACK are intentionally not
// intercepted by desktop chrome: they are delivered unchanged to Wayland clients.

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
        if state == ButtonState::Released && self.context_menu_consumed_buttons.remove(&(device, button)) {
            return;
        }
        if !seat_transition {
            return;
        }

        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };

        // Desktop menus own only primary/secondary clicks. Auxiliary mouse
        // buttons (including browser back/forward) must always reach clients.
        if state == ButtonState::Pressed
            && matches!(button, BTN_LEFT | BTN_RIGHT)
            && self.desktop_interactions.context_menu_is_open()
        {
            let location = pointer.current_location();
            let logical = DesktopPoint::new(location.x.round() as i32, location.y.round() as i32);
            let menu_layout = self.desktop_interactions
                .context_menu_position()
                .and_then(|anchor| self.context_menu_layout(anchor));

            if let Some(layout) = menu_layout.filter(|layout| layout.contains(logical)) {
                let interaction = if button == BTN_LEFT {
                    layout.command_at(logical).map(|command| {
                        DesktopInteraction::ContextMenuCommandSelected { command }
                    })
                } else {
                    None
                };
                let interaction = interaction.unwrap_or(DesktopInteraction::DismissContextMenu);
                if self.desktop_interactions.send(interaction).is_err() {
                    tracing::debug!("Shell context-menu receiver is no longer available");
                }
                self.context_menu_consumed_buttons.insert((device, button));
                self.request_all_output_frames();
                return;
            }

            // A click outside the menu dismisses it, then continues through
            // normal Wayland/desktop routing. A secondary click on empty
            // desktop can therefore open a new menu at the new anchor.
            let _ = self.desktop_interactions.send(DesktopInteraction::DismissContextMenu);
        }

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
                (BTN_LEFT, None) => {
                    // The desktop canvas owns selection only when no mapped
                    // Wayland surface is under the pointer.
                    self.select_desktop_item_at(location, true);
                }
                (BTN_RIGHT, Some(_)) => {
                    // A window owns its own context-menu interaction. Do not
                    // replace it with the desktop shell menu.
                    self.desktop_click_tracker.reset();
                }
                (BTN_RIGHT, None) => {
                    // Right-click selects the item under the pointer (or
                    // clears selection on empty space) before opening the
                    // desktop-owned context menu.
                    self.select_desktop_item_at(location, false);
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


    /// Resolves the menu's output from its global logical anchor and builds
    /// the same constrained geometry used by the visual presentation layer.
    pub(super) fn context_menu_layout(
        &self,
        anchor: DesktopPosition,
    ) -> Option<DesktopContextMenuLayout> {
        let anchor_point = DesktopPoint::new(anchor.x, anchor.y);
        let output = self.output_manager.registry().iter()
            .filter(|output| output.is_enabled())
            .find_map(|output| {
                let bounds = self.output_manager.logical_bounds(output.id()).ok().flatten()?;
                let origin = bounds.origin();
                let size = bounds.size();
                let origin = DesktopPoint::new(
                    i32::try_from(origin.x).ok()?,
                    i32::try_from(origin.y).ok()?,
                );
                let rect = DesktopRect::new(origin, size.width, size.height)?;
                rect.contains(anchor_point).then_some(rect)
            })?;
        DesktopContextMenuLayout::at(anchor_point, output)
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
            if self.context_menu_consumed_buttons.remove(&(device, button)) {
                continue;
            }
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
