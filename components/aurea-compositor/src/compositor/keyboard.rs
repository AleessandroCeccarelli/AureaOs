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
                    && compositor.suppressed_shortcut_releases.remove(&keycode)
                {
                    return FilterResult::Intercept(());
                }

                if state == KeyState::Pressed
                    && compositor.handle_keyboard_shortcut(key.modified_sym(), *modifiers)
                {
                    compositor.suppressed_shortcut_releases.insert(keycode);
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

    /// Applies a resolved global keyboard shortcut to the desktop state.
    ///
    /// Returns `true` when the shortcut was recognized and applied. The input
    /// backend is responsible for calling this only for key press events.
    pub fn handle_keyboard_shortcut(
        &mut self,
        keysym: smithay::input::keyboard::Keysym,
        modifiers: smithay::input::keyboard::ModifiersState,
    ) -> bool {
        use crate::input::shortcuts::{ShortcutAction, resolve_shortcut};

        let Some(action) = resolve_shortcut(keysym, modifiers) else {
            return false;
        };

        let current = self.window_manager.active_workspace();
        let count = crate::window_manager::WindowManager::WORKSPACE_COUNT;

        let next = match action {
            ShortcutAction::PreviousWorkspace => (current + count - 1) % count,
            ShortcutAction::NextWorkspace => (current + 1) % count,
        };

        // Use the compositor-level operation so keyboard focus is updated too.
        self.switch_workspace(next)
    }
}
