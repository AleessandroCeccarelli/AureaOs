// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Keyboard initialization, input handling, focus and global shortcuts.

use std::io;

use smithay::{
    backend::input::{KeyState, Keycode},
    input::keyboard::{FilterResult, XkbConfig},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
};

use super::AureaCompositor;

impl AureaCompositor {
    /// Initializes the Wayland keyboard using Smithay's default XKB layout.
    pub fn initialize_keyboard(&mut self) -> io::Result<()> {
        if self.seat.get_keyboard().is_none() {
            self.seat
                .add_keyboard(XkbConfig::default(), 200, 25)
                .map_err(io::Error::other)?;
        }

        Ok(())
    }

    /// Feeds one backend key event through the compositor shortcut filter and
    /// forwards unhandled events to the focused Wayland client.
    pub fn handle_keyboard_input(&mut self, keycode: Keycode, state: KeyState, time_millis: u32) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };

        let serial = SERIAL_COUNTER.next_serial();

        keyboard.input(
            self,
            keycode,
            state,
            serial,
            time_millis,
            |compositor, modifiers, key| {
                if state == KeyState::Released
                    && compositor.consumed_shortcut_keys.remove(&keycode)
                {
                    return FilterResult::Intercept(());
                }

                if state == KeyState::Pressed
                    && compositor.consumed_shortcut_keys.contains(&keycode)
                {
                    // Ignore key-repeat events for a shortcut already consumed.
                    return FilterResult::Intercept(());
                }

                if state == KeyState::Pressed
                    && compositor.handle_keyboard_shortcut(key.modified_sym(), *modifiers)
                {
                    compositor.consumed_shortcut_keys.insert(keycode);
                    return FilterResult::Intercept(());
                }

                FilterResult::Forward
            },
        );
    }

    /// Focuses a mapped surface only when no client currently owns keyboard focus.
    ///
    /// Mapping a new client surface is not, by itself, permission to steal
    /// focus from an already active application. A future activation-token
    /// policy can explicitly authorize focus changes initiated by user input.
    pub(crate) fn focus_surface_if_unset(&mut self, surface: WlSurface) {
        let focus_is_set = self
            .seat
            .get_keyboard()
            .and_then(|keyboard| keyboard.current_focus())
            .is_some();

        if !focus_is_set {
            self.focus_surface(Some(surface));
        }
    }

    /// Changes the focused Wayland surface for the compositor seat.
    pub fn focus_surface(&mut self, surface: Option<WlSurface>) {
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_focus(self, surface, SERIAL_COUNTER.next_serial());
        }
    }
}
