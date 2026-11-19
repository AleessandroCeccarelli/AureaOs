// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS
// Copyright (C) 2026 Alessandro Ceccarelli

//! Execution coordinator for resolved desktop keyboard shortcuts.

use smithay::input::keyboard::{Keysym, ModifiersState};

use super::AureaCompositor;

impl AureaCompositor {
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
